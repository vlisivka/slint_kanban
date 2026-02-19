# Plan - Fix Double Reload

## Problem
When the user changes the active user or toggles the "Show only my tickets" filter, the application reloads the board twice:
1. Immediately after the change (manual trigger).
2. ~100ms later when the file watcher detects the change to `config.toml`.

This double reload causes UI freezes and wastes resources.

## Solution

### 1. Remove Manual Reload
Since updating the configuration writes to `config.toml`, the file watcher **will** detect this change and trigger a reload automatically. We should remove the manual `reload_board` calls in the following callbacks:
- `on_change_active_user`
- `on_toggle_show_only_mine`
- `on_accept_search` (history update)
- `on_remove_search_item` (history update)
- `on_request_change_limit`

### 2. Verify Watcher Behavior
Create a test case to verify that modifying `config.toml` triggers exactly one reload event from the watcher.

## Implementation Steps
1. Create `tests/watcher_test.rs` to simulate the file watcher logic and ensure it debounces correctly.
2. Modify `src/main.rs`:
   - Remove `reload_board` call in `on_change_active_user`.
   - Remove `reload_board` call in `on_toggle_show_only_mine`.
   - Remove `reload_board` call in `on_accept_search` (history).
   - Remove `reload_board` call in `on_remove_search_item` (history).
   - Remove `reload_board` call in `on_request_change_limit` (already absent? Need to check).
3. Verify the fix manually (since GUI testing is limited).
