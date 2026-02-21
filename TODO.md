# Slint Kanban Implementation Roadmap

This file tracks the detailed steps to build the Slint Kanban application, following an incremental development approach.

## Phase 1: Viewer (Read-only)
- [x] Initialize Rust project with `slint`, `serde`, `walkdir`, `chrono`.
- [x] Create `Ticket` and `Queue` structs to represent data.
- [x] Implement logic to scan `~/Kanban` and load queues/tickets into memory.
- [x] Create basic Slint UI similar to Trello UI:
    - [x] Main window layout (horizontal scrollable area for columns).
    - [x] `KanbanColumn` component.
    - [x] `TicketCard` component (display title and short ID).
- [x] Connect Rust logic to Slint UI to display the board state.
- [x] Make tickets use fixed amount of vertical space, enough to display title, date, and part of first line of ticket body.
- [x] UI Refinement: Update ticket cards to show only the first line of the ticket body and ensure it doesn't overflow.
- [x] Add "Board Info" button to UI to open root `README.md` in a viewer

## Phase 2: Reactivity (File Watcher)
- [x] Add `notify` crate dependency.
- [x] Implement a file watcher running in a separate thread.
- [x] Send events from watcher to the main thread when file system changes occur.
- [x] Refresh the board model in Slint when changes are detected.
- [x] Make sure that no endless loops are made.
- [x] Make sure that app sleeps properly between filesystem change notifications, instead of hogging CPU. Fixed by filtering `Access` events and updating Slint.

## Phase 3: Moving Tickets (Drag-and-Drop)
- [x] Implement Drag-and-Drop in UI.
- [x] Handle "drop" events in Rust.
- [x] Implement `move_ticket` function to move directories on disk.
- [x] Error handling for failed moves.
- [x] Fix: Correct drag-and-drop target calculation when some queues are hidden.

## Phase 4: Deleting Tickets
- [x] Add a "Delete" button/icon to `TicketCard`.
- [x] Show a confirmation dialog (native or custom).
- [x] Implement `delete_ticket` function to move ticket directory to `~/Kanban/Deleted` directory.

## Phase 5: Creating & Editing Tickets
- [x] Add "New Ticket" button to columns.
- [x] Implement `create_ticket` function (create directory + basic `README.md`).
- [x] Add a click handler to `TicketCard` to open read-only full-window details view.
- [x] Add an "Edit" button to the details view or `TicketCard` to trigger editing.
- [x] Implement text editing for `README.md` content.
- [x] Save changes to disk on button press or auto-save.
- [x] Fix: Save description when creating a new ticket.
- [x] Premium UI Polish (Colors & Contrast).

## Phase 6: Initialization & Bootstrapping
- [x] Implement automatic creation of `~/Kanban` root and sub-directories (`Ticket`, `Queue`).
- [x] Refine: Create default queues with numbered prefixes (e.g., `1. Incoming`) for easy sorting.
- [x] Implement command-line argument handling to allow overriding the root directory (defaults to `~/Kanban`).
- [x] Implement queue sorting by name in `Board::load`.

## Phase 7: Future Enhancements

### Queue Limits
- [x] Implement configurable queue limits
  - [x] Add configuration file support (YAML/TOML)
  - [x] Add limit settings per queue
  - [x] Visual indicators when approaching limits
  - [x] Prevent adding tickets when limit reached
  - [x] Show warning dialog when limit exceeded
  - [x] Add button to set or change queue limits.

### Cross-Reference Navigation
- [x] Implement ticket cross-referencing
  - [x] Parse ticket IDs in markdown content (e.g., `#T-abc123`)
  - [x] Make ticket references clickable
  - [x] Navigate to referenced ticket on click
  - [x] Show tooltip with title on hover
  - [x] Support copying ticket ID to clipboard from `TicketCard`, `TicketView`, and `TicketEdit`

### Command Line Interface (CLI)
- [x] Refactor `main.rs` to move application logic into a testable function.
- [x] Implement CLI argument parsing using `clap`.
- [x] Implement non-interactive commands:
    - [x] `add`: create new ticket with options for title, description, and queue.
    - [x] `update`: update ticket attributes by ID.
    - [x] `move`: move ticket between queues by ID.
    - [x] `remove`: delete ticket by ID.
    - [x] `list`: list tickets with filtering options.
    - [x] `show`: show detailed ticket information.
    - [x] `configure`: manage application settings via CLI.
- [x] Implement `open PATH` command to control GUI from command line.
- [x] Add automated tests for all CLI commands.

### Configuration Refinement
- [x] Split configuration into board-wide and user-specific files
  - [x] Define `KanbanConfig` and `UserConfig` structs
  - [x] Update `Config` to manage both files (merging/splitting logic)
  - [x] Implement path resolution (Board root for `config.toml`, `~/.config` for `user.toml`)
  - [x] Update UI and CLI to read/write from appropriate files
  - [x] Migrate existing combined `config.toml` if it exists
  - [x] Add unit tests for split config loading and saving

### Search and Filter
- [x] Implement search functionality
  - [x] Add search input field to UI
  - [x] Full-text search across ticket titles and descriptions
  - [x] Filter by queue
  - [x] Filter by date range
  - [x] Search history
  - [x] Order tickets in queues by time of last update.

### Multi-user Support
- [x] Implement multi-user support
  - [x] Update `Config` model with `users`, `active_user`, and `show_only_mine`
  - [x] Add `assigned_to` field to `Ticket` and `TicketMetadata`
  - [x] Implement user selection and filtering toggle in global UI
  - [x] Add user assignment selector in `TicketEdit` and `TicketView`
  - [x] Update CLI commands to handle `--assigned-to`
  - [x] Add unit tests for user-based filtering and config
  - [x] Add `author` field to `Ticket` and `TicketMetadata` (automatically set to active user on creation)

### Keyboard Shortcuts
- [x] Implement essential keyboard shortcuts
  - [x] Quick search (Ctrl+F) focus
  - [x] Create new ticket in first visible queue (Ctrl+N)
  - [x] Toggle "Show only mine" (Ctrl+M)
  - [x] Close dialogs or clear search (Esc)
  - [x] Select from search history with Down Arrow

### Recycle Bin Improvements
- [x] Create delete confirmation dialog with specific wording ("ticket is moved to recycle bin")
- [x] Move deleted tickets to system Recycle Bin.

### Ticket Comments
- [ ] Implement reading comments from `tc<NNN><UID>.md` files in ticket directory
- [ ] Support YAML frontmatter in comments (`author`, `created_at`, `updated_at`)
- [ ] Extract and display ticket references (`#abc123`) below comments
- [ ] Sort comments by `created_at` (older first)
- [ ] Update UI to display comments in `TicketView`
- [ ] Implement adding new comments via UI

### Attachments
- [ ] Create `attachment/` sub-directory logic in ticket models
- [ ] Implement file copying with duplicate name handling (`file (1).ext`)
- [ ] Add `attachments` field to ticket and comment frontmatter
- [ ] Implement "Attach..." button and file dialog in UI
- [ ] Display attachments in `TicketView` (under description and comments)
- [ ] Add interaction to open attachments or their parent directory

### Statistics and Analytics
- [ ] Implement analytics dashboard
  - [ ] Ticket count per queue
  - [ ] Time tracking per ticket/queue
  - [ ] Completion rate calculations
  - [ ] Trend visualization
  - [ ] Burndown charts
  - [ ] Export statistics to CSV

### Export Functionality
- [ ] Implement export features
  - [ ] Export single ticket with comments and attachment (names only) to PDF (including it queue name)
  - [ ] Export entire queue with tickets to PDF
  - [ ] Export entire board with board description and all queues and their tickets to PDF
  - [ ] Export to HTML format
  - [ ] Export to Markdown
  - [ ] Batch export functionality
  - [ ] Export configuration options

### Markdown Rendering
- [/] Implement Markdown rendering in ticket detail view using Servo
  - [ ] Integrate markdown rendering library (pulldown-cmark or similar)
  - [ ] Copy WebView component from Slint "servo" example into project and use it: https://github.com/slint-ui/slint/tree/master/examples/servo
  - [ ] Create styled text rendering component
  - [ ] Support for headings, lists, code blocks
  - [ ] Support for links and images

## Bugs
- [x] Fix double board reload (and freeze) when changing user settings.
- [x] Fix unassign ticket bug: saving empty user assignment fails.
