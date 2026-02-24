# Plan - Phase 2: Reactivity (File Watcher)

## Meta
**Goal**: Make the application react to file system changes (created/moved/deleted tickets) in real-time without manual refresh.

## Changes

### 1. `Cargo.toml`
-   **Dependencies**: Add `notify = "6.1.1"`.

### 2. `src/main.rs`
-   **Refactoring**: Extract board loading and UI update logic into a helper function `reload_board(ui: &App, root: &Path)`.
-   **Watcher Setup**:
    -   Spawn a separate thread.
    -   Initialize `notify::RecommendedWatcher`.
    -   Watch `~/Kanban` using inotify or a similar crate.
    -   Event Handling: On any event (Create/Modify/Remove), trigger a board reload.
-   **Thread Communication**:
    -   Use `slint::invoke_from_event_loop` to safely execute code on the main UI thread from the watcher thread.

### 3. `ui/app.slint`
-   No changes required (UI is passive, driven by the model).

## Logic Flow
1.  App starts, loads board (Optimization: Initial load).
2.  Watcher thread starts.
3.  Watcher detects change in `~/Kanban`.
4.  Watcher calls `slint::invoke_from_event_loop`.
5.  Main thread closure executes `reload_board`.
6.  `reload_board` re-scans `~/Kanban` and replaces the Slint model data.

## Verification

### Automated Tests
-   Difficult to test multi-threaded file watching in unit tests reliably without flakiness.
-   We will rely on manual verification.

### Manual Verification
1.  Run the app.
2.  Open a terminal.
3.  `touch ~/Kanban/Queue/To Do/new_ticket_link` (simulated).
4.  Observe the app UI updates automatically.
5.  `rm ~/Kanban/Tickets/T1/README.md`.
6.  Observe update.
