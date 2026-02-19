# Plan - Multi-user Support

This plan outlines the implementation of multi-user support in the Slint Kanban application. This allows multiple people to collaborate using a shared file storage (e.g., Git/Dropbox) by assigning tickets to themselves or others.

## Current State
- Monolithic user model (implicitly "local user").
- `Config` contains queue limits, hidden queues, and search history.
- `TicketMetadata` contains title, creation date, and update date.

## Proposed Changes

### 1. Data Model Updates (Backend)

#### `src/model/config.rs`
- Add `users: Vec<String>` (default: `["user"]`).
- Add `active_user: String` (default: first from `users`).
- Add `show_only_mine: bool` (default: `false`).
- Update `Default` implementation and loading logic.

#### `src/model/ticket.rs`
- Add `assigned_to: String` to `TicketMetadata` and `Ticket`.
- Support optional `assigned_to` in YAML deserialization (default to empty string).

#### `src/model/board.rs`
- Ensure `assigned_to` is preserved during loading/saving/updating.

### 2. UI Updates (Slint)

#### `ui/common.slint`
- Update `AppConfig` struct to include `users: [string]`, `active_user: string`, and `show_only_mine: bool`.
- Add `assigned_to` field to `TicketStr`.

#### `ui/app.slint`
- Add a user selection dropdown in the top bar or a "Settings" menu.
- Add a toggle switch for "Mine Only" filter.
- Update ticket filtering logic to respect `show_only_mine` and `active_user`.

#### `ui/dialogs/ticket_view.slint` & `ui/dialogs/ticket_edit.slint`
- Add a field to display/edit the assigned user.
- For editing, use a `ComboBox` or similar selection from the `users` list.

### 3. Application Logic (Rust)

#### `src/main.rs`
- Update `ticket_to_slint` to populate the `assigned_to` field.
- Update `update_board` to filter tickets if `show_only_mine` is true.
- Implement callbacks:
    - `change_active_user(username)`
    - `toggle_show_only_mine(enabled)`
    - `assign_ticket(ticket_id, username)`
- Update CLI commands in `cli.rs` and `main.rs` to support `--assigned-to`.

### 4. Testing
- Unit tests for `Config` updates.
- Unit tests for user-based ticket filtering in `Board` or UI conversion logic.
- Integration tests for UI toggles.

## Implementation Steps

### Phase 1: Backend & Models
1. Modify `src/model/config.rs` to add user fields.
2. Modify `src/model/ticket.rs` to add `assigned_to`.
3. Update `src/model/board.rs` methods (especially `create_ticket` and `update_ticket`).
4. Fix tests in `src/model/tests/`.

### Phase 2: UI Definitions
1. Update `ui/common.slint` with new fields.
2. Update `ui/dialogs/ticket_view.slint` and `ui/dialogs/ticket_edit.slint`.
3. Update `ui/app.slint` to add user controls.

### Phase 3: Integration & Logic
1. Update `src/main.rs` to handle user selection and filtering.
2. Update CLI handlers for assignment.
3. Verify file-system persistence (config.toml and ticket READMEs).

### Phase 4: Verification
1. Run all automated tests.
2. Manually verify multi-user switching and filtering.
