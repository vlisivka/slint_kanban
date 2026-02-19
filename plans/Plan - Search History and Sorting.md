# Implementation Plan: Search History and Ticket Sorting

This plan covers the implementation of ticket sorting by update time, search history persistence, and related UI refinements.

## Goal
- Improve board usability by showing recently updated tickets at the top.
- Enable users to quickly reuse previous search queries.
- Ensure filters are preserved during board reloads.

## Task List

### 1. Ticket Sorting
- [x] Modify `model::load_queue` to sort the `tickets` vector by `updated_at` in descending order.

### 2. Search History Backend
- [x] Update `Config` struct in `src/model.rs` to include `search_history: Vec<String>`.
- [x] Implement `Config::add_search_to_history(query: String)`:
    - Prepend new queries.
    - Move duplicates to the top.
    - Limit history to the 10 most recent unique items.
    - Ignore empty or whitespace-only queries.
- [x] Add automated test `test_search_history` in `src/model/tests.rs`.
- [x] Implement `Config::remove_search_from_history(query: String)` to delete specific items.

### 3. UI Implementation (app.slint)
- [x] Create `SearchHistoryMenu` component with a list of recent searches.
- [x] Add `search_history` and `show_search_history` properties to `App`.
- [x] Add "History" button to the search header.
- [x] Implement overlay logic to show/hide the history menu.
- [x] Update `SearchHistoryMenu` to include a delete button for each history item.
- [x] Fix layout issues:
    - Set `height: self.preferred-height` to prevent vertical stretching.
    - Implement `overflow: elide` and layout stretching for long history strings.
    - Fix button layering (z-order) so delete buttons are clickable.
    - Synchronize delete button styling with ticket delete button.

### 4. Application Integration (main.rs)
- [x] Update `reload_board` to sync search history from config to UI.
- [x] Implement `on_accept_search` callback:
    - Save search query to history and update `config.toml` when the user presses Enter.
- [x] Implement `on_select_history_item` callback:
    - Trigger board filtering when a history item is clicked.
- [x] Fix `reload_board` to preserve `date_from` and `date_to` filters.
- [x] Implement `on_remove_search_history_item` callback to update config and UI.

### 5. Verification
- [x] Run `cargo test` to ensure all 30 tests pass.
- [x] Verify UI appearance (no giant buttons/stretching).
