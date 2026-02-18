# Plan: Queue Limit Configuration Button

## Goal
Implement a way for users to set or change queue limits directly from the Kanban board UI, instead of manually editing `config.toml`.

## Proposed Changes

### UI Changes (`ui/app.slint`)

1. **New Overlay: `QueueLimitEdit`**
   - A dialog that appears when the user want to change a queue limit.
   - Shows the queue name.
   - Contains a `LineEdit` for the limit value (integer).
   - "Save" button to apply the change.
   - "Cancel" button to close without saving.
   - "Remove Limit" button to set no limit (internally -1).

2. **`KanbanColumn` Updates**
   - Change the Limit Counter Badge from a simple `Rectangle` to a `TouchArea` or add a `TouchArea` inside it.
   - On click, trigger a callback to open the `QueueLimitEdit` overlay.

3. **`App` Component Updates**
   - Properties:
     - `is_editing_limit: bool`
     - `editing_limit_queue_id: string`
     - `editing_limit_queue_name: string`
     - `editing_limit_value: int`
   - Callback:
     - `request_change_limit(string, int)`: (queue_id, new_limit)

### Backend Changes (`src/main.rs`)

1. **Register Callback**
   - Register `on_request_change_limit` in `run_gui`.
   - The implementation will:
     - Load the board (to get current config).
     - Call `board.config.set_limit(queue_id, limit)` (if limit >=0) or remove from hashmap (if limit < 0).
     - Call `board.config.write(&root_path)`.
     - The file watcher will automatically trigger a reload of the UI.

## Verification Plan

### Automated Tests
- Add a test case in `src/main_tests.rs` (or `src/main.rs` if tests were there, but they were moved) to verify the new callback logic.
- Verify that `Config::write` is called and the file is updated.

### Manual Verification
1. Start the application.
2. Click on a queue limit badge (or the area where it should be).
3. Change the limit in the dialog.
4. Save and verify that:
   - The UI reflects the new limit.
   - The `config.toml` file is updated.
   - Enforcement logic works with the new limit (e.g., trying to move/create tickets).
