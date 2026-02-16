# Plan - Scan Kanban Board

## Meta
**Goal**: Implement the logic to scan the file system at `~/Kanban` and populate the `Board` struct with `Queue`s and `Ticket`s, respecting the symlink architecture.

## Changes

### 1. `src/model.rs`
-   **Helper Function `initialize_board`**:
    -   Accepts `root_path`.
    -   Checks if `root_path` exists. If not, maybe create it or return empty.
    -   Reads `root_path/Queue` entries to find queues.
    -   For each queue directory:
        -   Reads entries.
        -   If entry is a symlink, resolves it to `root_path/Tickets`.
        -   Reads the target directory's `README.md` to get metadata.
        -   Constructs `Ticket` object.
    -   Returns `Board`.

### 2. `src/lib.rs` (or `src/main.rs`)
-   Call `initialize_board` in `main`.
-   Print the board structure to console (temporary, for verification).

## Verification

### Automated Tests
-   **Test Board Scanning**:
    -   Create a temporary directory structure:
        -   `Root/Tickets/T1/README.md`
        -   `Root/Queue/Q1/symlink_to_T1`
    -   Run `initialize_board`.
    -   Assert `Board` contains 1 queue "Q1" with 1 ticket "T1".
-   **Test Board Scanning**:
    -   Create a temporary directory structure:
        -   `Root/Tickets/ttt123/README.md`
        -   `Root/Tickets/ttt456/README.md`
        -   `Root/Queue/q1/symlink_to_ttt123`
        -   `Root/Queue/q2/symlink_to_ttt456`
    -   Run `initialize_board`.
    -   Assert `Board` contains 2 queues, "q1" with 1 ticket "ttt123", "q2" with 1 ticket "ttt456".

### Manual Verification
-   Create `~/Kanban` structure manually.
-   Run `cargo run` and check console output.

## Implementation Status
-   [x] Implemented `Ticket`, `Queue`, `Board` structs.
-   [x] Implemented `Board::load` with symlink resolution.
-   [x] Verified with `test_board_scanning` and `test_board_scanning_multiple_queues`.

