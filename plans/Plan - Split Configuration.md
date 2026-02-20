# Plan - Split Configuration

The goal is to separate board-wide settings (synchronized across users) from local user preferences (machine-specific).

## Goals
- Move `users` and `queue_limits` to `~/Kanban/kanban.toml`.
- Move `active_user`, `show_only_mine`, `hidden_queues`, and `search_history` to `~/.config/slint-kanban/user.toml`.

## Proposed Changes

### 1. Data Models (`src/model/config.rs`)
- Introduce `KanbanConfig` and `UserConfig` structs.
- Retain `Config` as a facade that holds both and manages their persistence separately.
- Implement path resolution logic for `UserConfig` using `dirs` or a similar approach (manual path construction for now: `~/.config/APP_NAME/user.toml`).
- Ensure that board is properly reloaded after update of machine-specific config, which will not triger automatic reload via FS monitor.

### 2. Implementation logic
- **Loading**:
    - Try to find `config.toml` in the board root.
    - Try to find `user.toml` in `~/.config/APP_NAME/`.
- **Saving**:
    - `Config::write()` will now save to two separate files.

### 3. API Updates
- Update `AppController` and UI callbacks to ensure they still work with the unified `Config` facade.

### 4. CLI Updates
- Ensure `--root` still correctly picks up the board-specific `config.toml`.
- `configure` command should accurately update either `kanban.toml` (for `--add-user`) or `~/.config/APP_NAME/user.toml` (for `--active-user`).

## Verification Plan

### Automated Tests
- Add unit tests in `src/model/tests/config_tests.rs`:
    - Test loading from separate files.
    - Test migration from old `config.toml`.
    - Test saving updates to correct files.
- Run existing GUI and CLI tests to ensure no regressions.

### Manual Verification
- Start the app with an existing `config.toml`.
- Verify that `~/.config/slint-kanban/user.toml` is created.
- Verify that `~/Kanban/config.toml` is updated with new data after change.
- Change a user setting (e.g., active user) and verify only `user.toml` changes.
- Change a board setting (e.g., queue limit) and verify only `config.toml` changes.
