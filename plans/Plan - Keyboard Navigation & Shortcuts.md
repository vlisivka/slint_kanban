# Plan - Keyboard Navigation & Shortcuts

Implementing essential keyboard shortcuts to improve productivity without requiring users to learn complex navigation patterns.

## Goals
- **Search & Filter**:
    - `Ctrl+F`: Focus the search input field.
    - `Esc`: Clear the search field or close any open dialog.
    - `Down Arrow`: (When search is focused) Open search history/suggestions.
- **Ticket Management**:
    - `Ctrl+N`: Create a new ticket in the first visible queue.
- **View Toggles**:
    - `Ctrl+M`: Toggle between "Show only my tickets" and "Show all tickets".

## Proposed Changes

### 1. `ui/app.slint`
- Add a root-level `FocusScope` or handle `key-pressed` in the main `Window`.
- Implement global shortcut logic:
    - `Ctrl+F` -> `search-input.focus()`.
    - `Ctrl+N` -> Trigger `create_ticket` for the first queue in `board_queues` that is not hidden.
    - `Ctrl+M` -> Toggle `show_only_mine` property.
    - `Esc` -> 
        - If a dialog is open (e.g., `is_viewing_ticket` or `is_editing_ticket`): Close it.
        - If search field is focused/not empty: Clear search and unfocus.
- Enhance `search-input`:
    - Add `key-pressed` handler for `Down Arrow` to trigger showing search history dropdown/logic.

### 2. `src/main.rs` (if processing shortcuts in Rust)
- Ensure the Rust backend can handle the `Ctrl+M` toggle and update the user configuration accordingly.
- (Optional) If history selection requires Rust-side logic, ensure it's exposed to Slint.

## Verification Plan

### Automated Tests
- **GUI Tests (`src/gui_tests.rs`)**:
    - Verify `Ctrl+F` focuses the search input.
    - Verify `Ctrl+N` opens the creation dialog for the first queue.
    - Verify `Ctrl+M` changes the "show only mine" state.
    - Verify `Esc` closes open dialogs.

### Manual Verification
- Open the app.
- Press `Ctrl+F` and type a search.
- Press `Esc` to clear search.
- Press `Ctrl+N` and verify the new ticket dialog appears for the first queue.
- Press `Ctrl+M` and see the ticket list filter toggling.
- Focus search and press `Down Arrow` to ensure history interaction works (once implemented).
