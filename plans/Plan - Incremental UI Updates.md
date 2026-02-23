# Plan - Incremental UI Updates

## Goal
Optimize the synchronization between the Rust backend and the Slint UI to handle 1000+ tickets smoothly. Instead of replacing the entire board model on every change, we will update individual rows and nested models only when necessary.

## Proposed Changes

### 1. `TicketStr` Caching
*   Implement a cache for `TicketStr` (the Slint-compatible ticket struct) in `AppController`.
*   Key: `(ticket_id, ticket_updated_at_timestamp)`.
*   Value: `TicketStr`.
*   **Rationale**: Converting a domain `Ticket` to `TicketStr` involves creating `SharedString`s and nested models for references/comments. Doing this 1000 times on every file change is wasteful.

### 2. Persistent Models in `AppController`
*   Add `board_queues_model: Rc<VecModel<QueueStr>>` to `AppController`.
*   Keep a mapping of `queue_id -> Rc<VecModel<TicketStr>>`.
*   **Rationale**: By keeping the same `VecModel` instances and modifying them, Slint's UI can perform more efficient incremental re-rendering.

### 3. Incremental Synchronization Logic
*   Refactor `sync_board_to_ui` to:
    1.  Update the top-level `board_queues_model` row-by-row.
    2.  For each queue, update its `tickets` `VecModel` row-by-row.
    3.  If a row is identical (using ID and `updated_at` as proxies for identity), skip it.
    4.  If the number of items changed, perform `push`/`pop` or `insert` as needed.
    5.  For simplicity in this step, we can use a "diff and patch" approach or just a "sync counts then update same length" approach.

### 4. Technical Details
*   `AppController` will need internal mutability for the model storage. Since it's used in `Arc`, we'll use `Mutex` or `parking_lot::Mutex`.
*   The `sync_board_to_ui` function in `lib.rs` will be updated to take these persistent models as arguments.

## Automated Tests
1.  **Correctness**: Ensure that after a ticket is moved or updated, the UI reflects the correct state.
2.  **Performance**: Run `simulate_large_board` and observe UI responsiveness during ticket moves.
3.  **No Flicker**: Verify that incremental updates don't cause the UI components (like list selection) to jump or flicker unnecessarily.

## Manual Verification
1.  Open a board with 1000 tickets.
2.  Perform a search - should be instantaneous.
3.  Move a ticket - should be smooth.
4.  Edit a ticket - properties should update in the list without the whole list resetting.
