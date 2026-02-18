# Plan - Phase 3: Moving Tickets (Drag-and-Drop)

## Meta
**Goal**: Allow users to move tickets between columns by dragging them. This translates to moving a symlink from one queue directory to another.

## Proposed Changes

### 1. `src/model.rs` [MODIFY]
- Implement `Board::move_ticket(ticket_id: &str, source_queue_id: &str, target_queue_id: &str) -> anyhow::Result<()>`.
- This function will:
    - Find the symlink in `~/Kanban/Queue/<source_queue_id>/`.
    - Move (rename) it to `~/Kanban/Queue/<target_queue_id>/`.
    - It does NOT touch `~/Kanban/Tickets/`.

### 2. `ui/app.slint` [MODIFY]
- Add `callback move_ticket(string, string, string)` (ticket_id, from_queue, to_queue).
- Update `TicketCard`:
    - Add a `TouchArea` to handle drag gestures.
    - Since Slint lacks a built-in cross-component DND, we'll implement a simplified version:
        - When a card is "dropped" over a column, trigger the callback.
        - Alternatively, for a MVP, we can add "Move Left/Right" buttons if DND proves too complex for Slint's current state, but we'll try DND first using global mouse position or similar.
        - *Correction*: Slint has `pointer-event` and `moved` events. We can track the drag and check which column is under the cursor on release.

### 3. `src/main.rs` [MODIFY]
- Bind `ui.on_move_ticket` to `Board::move_ticket`.
- The watcher will automatically refresh the UI after the move.

## Verification Plan

### Automated Tests
- Unit tests in `model.rs` to verify that `move_ticket` correctly moves symlinks and handles non-existent sources/targets.

### Manual Verification
1. Run `cargo run`.
2. Drag a ticket from "To Do" to "Doing".
3. Verify the ticket appears in the new column and disappears from the old one.
4. Verify the symlink on disk changed location.
5. Check logs for errors.
