# Plan - Administrator Mode Implementation

## Goal
Implement a special Administrator Mode interface (GUI) and CLI bypass for managing board-wide settings, users, queues, and documentation.

## Proposed Changes

### 1. Data Model & Board Logic (`src/model/board.rs`)
- Implement `Board::update_board_readme(&self, content: &str) -> anyhow::Result<()>`: Updates the root `README.md`.
- Implement `Board::add_queue(&self, name: &str) -> anyhow::Result<()>`: Creates a new queue directory.
- Implement `Board::rename_queue(&self, old_id: &str, new_name: &str) -> anyhow::Result<()>`: Renames a queue directory and updates configuration if necessary.
- Implement `Board::delete_queue(&self, id: &str) -> anyhow::Result<()>`: Deletes a queue directory (only if empty).
- Implement `Board::add_user(&mut self, user: &str) -> anyhow::Result<()>`: Proxy to Config.
- Implement `Board::remove_user(&mut self, user: &str) -> anyhow::Result<()>`: Proxy to Config.

### 2. Controller (`src/controller.rs`)
- Add methods to handle admin actions from the UI:
    - `handle_save_board_readme(content: String)`
    - `handle_add_queue(name: String)`
    - `handle_rename_queue(id: String, new_name: String)`
    - `handle_delete_queue(id: String)`
    - `handle_add_user(username: String)`
    - `handle_remove_user(username: String)`

### 3. UI Changes (`ui/app.slint` & new files)
- **`ui/app.slint`**:
    - Add `in-out property <bool> is_admin`.
    - Add "🛠 Admin" button to the header (visible only if `is_admin` is true).
    - Add `AdminSettings` dialog overlay.
- **`ui/dialogs/admin_settings.slint`**:
    - **Section 1: Board Info**: Edit root `README.md`.
    - **Section 2: Users**: List of users, add/remove.
    - **Section 3: Queues**: List of queues, rename/delete/add.
- **`ui/common.slint`**:
    - Add `is_admin` to `UserGlobal` or `AppConfig`? Better in `App` directly for now.

### 4. Main Entry Point (`src/main.rs`)
- Update `run_gui` to accept `admin` parameter and set it in the UI.
- Pass `args.admin` from `run_cli` to `run_gui`.
- Initialize new admin callbacks in `init_callbacks`.

### 5. CLI Enhancements (`src/main.rs`)
- Ensure all admin actions are also available via CLI (some already are in `Configure`).

## Tests
- **Unit Tests**: Test new `Board` methods for queue and user management.
- **Integration Tests**: Verify that `--admin` flag enables admin bypass in CLI and sets property in GUI.
- **GUI Tests**: Verify that admin dialog opens and actions trigger callbacks.

## Manual Verification
- Run with `--admin` flag.
- Open Admin Settings.
- Add/Rename/Delete a queue (check disk).
- Add/Remove a user (check `config.toml`).
- Edit Board Documentation (check `README.md`).
- Verify that without `--admin` flag, the admin button is hidden.
