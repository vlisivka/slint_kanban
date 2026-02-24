# Slint Kanban Implementation Roadmap

This file tracks the detailed steps to build the Slint Kanban application, following an incremental development approach.

## Phase 1: Viewer (Read-only) ✅
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

## Phase 2: Reactivity (File Watcher) ✅
- [x] Add `notify` crate dependency.
- [x] Implement a file watcher running in a separate thread.
- [x] Send events from watcher to the main thread when file system changes occur.
- [x] Refresh the board model in Slint when changes are detected.
- [x] Make sure that no endless loops are made.
- [x] Make sure that app sleeps properly between filesystem change notifications, instead of hogging CPU. Fixed by filtering `Access` events and updating Slint.

## Phase 3: Moving Tickets (Drag-and-Drop) ✅
- [x] Implement Drag-and-Drop in UI.
- [x] Handle "drop" events in Rust.
- [x] Implement `move_ticket` function to move directories on disk.
- [x] Error handling for failed moves.
- [x] Fix: Correct drag-and-drop target calculation when some queues are hidden.

## Phase 4: Deleting Tickets ✅
- [x] Add a "Delete" button/icon to `TicketCard`.
- [x] Show a confirmation dialog (native or custom).
- [x] Implement `delete_ticket` function. Tickets are moved to the system Recycle Bin via the `trash` crate.

## Phase 5: Creating & Editing Tickets ✅
- [x] Add "New Ticket" button to columns.
- [x] Implement `create_ticket` function (create directory + basic `README.md`).
- [x] Add a click handler to `TicketCard` to open read-only full-window details view.
- [x] Add an "Edit" button to the details view or `TicketCard` to trigger editing.
- [x] Implement text editing for `README.md` content.
- [x] Save changes to disk on button press or auto-save.
- [x] Fix: Save description when creating a new ticket.
- [x] Premium UI Polish (Colors & Contrast).

## Phase 6: Initialization & Bootstrapping ✅
- [x] Implement automatic creation of `~/Kanban` root and sub-directories (`Ticket`, `Queue`).
- [x] Refine: Create default queues with numbered prefixes (e.g., `1. Incoming`) for easy sorting.
- [x] Implement command-line argument handling to allow overriding the root directory (defaults to `~/Kanban`).
- [x] Implement queue sorting by name in `Board::load`.

## Phase 7: Enhancements

### Queue Limits ✅
- [x] Implement configurable queue limits
  - [x] Add configuration file support (TOML)
  - [x] Add limit settings per queue
  - [x] Visual indicators when approaching limits
  - [x] Prevent adding tickets when limit reached
  - [x] Show warning dialog when limit exceeded
  - [x] Add button to set or change queue limits

### Cross-Reference Navigation ✅
- [x] Implement ticket cross-referencing
  - [x] Parse ticket IDs in markdown content (e.g., `#abc123`)
  - [x] Make ticket references clickable
  - [x] Navigate to referenced ticket on click
  - [x] Support copying ticket ID to clipboard from `TicketCard`, `TicketView`, and `TicketEdit`

### Command Line Interface (CLI) ✅
- [x] Refactor `main.rs` to move application logic into a testable function.
- [x] Implement CLI argument parsing using `clap`.
- [x] Implement non-interactive commands:
    - [x] `add`: create new ticket with options for title, description, queue, assignee, and points.
    - [x] `update`: update ticket attributes by ID.
    - [x] `move`: move ticket between queues by ID.
    - [x] `remove`: delete ticket by ID.
    - [x] `list`: list tickets with filtering options (search, date range, assigned user).
    - [x] `show`: show detailed ticket information.
    - [x] `configure`: manage application settings via CLI.
    - [x] `stats`: show board statistics.
    - [x] `sprint`: manage sprints (list, current, add, update, remove).
    - [x] `comment`: add a comment to a ticket.
    - [x] `attach`: attach files, list attachments, show/open attachment directory.
- [x] Implement `open PATH` command to control GUI from command line.
- [x] Add automated tests for all CLI commands.

### Configuration Refinement ✅
- [x] Split configuration into board-wide and user-specific files
  - [x] Define `KanbanConfig` and `UserConfig` structs
  - [x] Update `Config` to manage both files (merging/splitting logic)
  - [x] Implement path resolution (Board root for `config.toml`, `~/.config` for `user.toml`)
  - [x] Update UI and CLI to read/write from appropriate files
  - [x] Migrate existing combined `config.toml` if it exists
  - [x] Add unit tests for split config loading and saving

### Search and Filter ✅
- [x] Implement search functionality
  - [x] Add search input field to UI
  - [x] Full-text search across ticket titles and descriptions
  - [x] Filter by queue (hide/show individual queues)
  - [x] Filter by date range
  - [x] Search history with dropdown
  - [x] Order tickets in queues by time of last update

### Multi-user Support ✅
- [x] Implement multi-user support
  - [x] Update `Config` model with `users`, `active_user`, and `show_only_mine`
  - [x] Add `assigned_to` field to `Ticket` and `TicketMetadata`
  - [x] Implement user selection and filtering toggle in global UI
  - [x] Add user assignment selector in `TicketEdit` and `TicketView`
  - [x] Update CLI commands to handle `--assigned-to`
  - [x] Add unit tests for user-based filtering and config
  - [x] Add `author` field to `Ticket` and `TicketMetadata` (automatically set to active user on creation)

### Keyboard Shortcuts ✅
- [x] Implement essential keyboard shortcuts
  - [x] Quick search (Ctrl+F) focus
  - [x] Create new ticket in first visible queue (Ctrl+N)
  - [x] Toggle "Show only mine" (Ctrl+M)
  - [x] Close dialogs or clear search (Esc)
  - [x] Select from search history with Down Arrow

### Recycle Bin ✅
- [x] Create delete confirmation dialog with specific wording ("ticket is moved to Recycle Bin")
- [x] Move deleted tickets to system Recycle Bin via `trash` crate

### Ticket Comments ✅
- [x] Implement reading comments from `tc<NNN><UID>.md` files in ticket directory
- [x] Support YAML frontmatter in comments (`author`, `created_at`, `updated_at`)
- [x] Extract and display ticket references (`#abc123`) below comments
- [x] Sort comments by `created_at` (older first)
- [x] Update UI to display comments in `TicketView`
- [x] Implement adding new comments via UI

### Attachments ✅
- [x] Create `attachment/` sub-directory logic in ticket models
- [x] Implement file copying with duplicate name handling (`file (1).ext`)
- [x] Generate Markdown links instead of adding `attachments` field to ticket/comment frontmatter
- [x] Implement "Attach..." button and file dialog in UI using `rfd`
- [x] Display attachment count and add button to open attachments directory in `TicketView`
- [x] Implement attach functionality via CLI command (`attach --file`, `--list`, `--show`, `--open`)

### Points (Estimation) ✅
- [x] Implement ticket estimation using "Points"
  - [x] Add `points` field to `Ticket` and `TicketMetadata` model
  - [x] Support scale from 1 to 10 with time mapping (1=1d, 5=1w, 7=1mo, 10=1y)
  - [x] Update `TicketEdit` UI to allow selecting points
  - [x] Update `TicketCard` and `TicketView` to display points with color badges
  - [x] Add CLI support for setting/updating points
  - [x] Include points in `stats` command (total points per user/sprint)

### Statistics and Analytics
- [x] Implement activity logging
  - [x] Log to `Kanban/logs/log_${USER}_${MACHINE_ID}.md` using Markdown table format with JSON payload
  - [x] Generate and store `machine_id` in user config on first run
  - [x] Create `ActionPayload` enum and `append_log_entry` function
  - [x] Log ticket creation, updates, moves, comments, attachments, assignments
- [x] Implement analytics dashboard (GUI + CLI)
  - [x] Ticket count per queue and per user
  - [x] Sprints (CLI CRUD + GUI display)
  - [x] Lead/Cycle time calculations from log parsing
  - [x] Completion rate calculations (overall + per sprint)
  - [x] Trend visualization (bar chart, last 15 days)
  - [x] Points completion rate
- [ ] Remaining analytics
  - [ ] Burndown charts
  - [ ] Export statistics to CSV

### Performance Optimization ✅
- [x] Quick Wins
  - [x] Add `HashMap` index for O(1) ticket lookup (fixes cross-reference lag)
  - [x] Optimize `get_board_summary` to load logs only once
  - [x] Add debounce to search input (300ms)
- [x] I/O Reduction
  - [x] Lazy load comments (only on ticket click)
  - [x] Use header-only loading for board view (YAML + first line snippet)
  - [x] Cache attachment count in ticket metadata
- [x] Scaling Improvements
  - [x] Single-pass log processing for Lead/Cycle time (O(L) complexity)
  - [x] Implement ticket caching (mtime based) via `TICKET_CACHE`
  - [x] Incremental UI updates for Slint models (diff-check before `set_row_data`)
  - [x] Ticket-to-SlintTicketStr cache in `AppController`
- [x] UI Rendering Optimization
  - [x] Remove `drop-shadow-blur` from non-hovered cards (shadows only on hover)
  - [x] Replace `TextEdit`+`Button` in IdCopy with lightweight `Rectangle`+`TouchArea`
  - [x] Implement native clipboard via `arboard` (eliminates hidden TextEdit widgets)
  - [x] Replace `VerticalBox`/`HorizontalBox` with lighter `VerticalLayout`/`HorizontalLayout`
  - [x] Extract point color calculation to cached property
  - [x] Increase file watcher debounce from 100ms to 500ms
  - [x] Add `opt-level = 1` to dev profile for faster runtime
  - [x] Add `debug = 1` (line tables only) to reduce disk/memory usage

---

### Conflicts Handling in Multi-User Multi-Machine Setup ✅
- [x] Implement conflict prevention via "Manage Only My Tickets" option
  - [x] Add `manage_only_mine` to `UserConfig` (default: true)
  - [x] Enforce management restriction in GUI (disable drag/edit for unassigned tickets)
  - [x] Enforce restriction in CLI commands
  - [x] Add setting toggle in GUI and CLI
- [x] Implement automatic queue conflict resolution
  - [x] Detect if ticket exists in multiple queues
  - [x] Keep ticket in the "furthest" queue (highest index) and cleanup others
  - [x] Detect orphaned tickets (not in any queue)
  - [x] Automatically link orphaned tickets to the first visible queue

### Administrator Mode ✅
- [x] Implement administrator mode (top-level `--admin` flag in CLI, `admin` user name bypass)
  - [x] Edit board `README.md`
  - [x] Manage users (add/remove from `config.toml`)
  - [x] Manage queues (add, rename, delete)
  - [x] Manage shared board settings (queue limits)
  - [x] Board initialization and setup logic
  - [x] Internal tools for fixing data corruption or resolving conflicts

### Export Functionality
- [ ] Implement export features
  - [ ] Export single ticket with comments and attachments to PDF (including queue name)
  - [ ] Export entire queue with tickets to PDF
  - [ ] Export entire board with description and all queues to PDF
  - [ ] Export to HTML format
  - [ ] Export to Markdown
  - [ ] Batch export functionality
  - [ ] Export configuration options

### Translate GUI, CLI, and Logs into User Language
  - [ ] Mark all user-facing strings
  - [ ] Extract translatable strings into a `.pot` file
  - [ ] Translate `.pot` file into Ukrainian language
  - [ ] Compile in translation statically or load dynamically

### Markdown Rendering
- [/] Implement Markdown rendering in ticket detail view using Servo
  - [ ] Copy WebView component from Slint "servo" example: https://github.com/slint-ui/slint/tree/master/examples/servo
  - [ ] Integrate markdown rendering library (pulldown-cmark or similar)
  - [ ] Create styled text rendering component
  - [ ] Support for headings, lists, code blocks
  - [ ] Support for links and images

---

## Bugs (Resolved)
- [x] Fix double board reload (and freeze) when changing user settings.
- [x] Fix unassign ticket bug: saving empty user assignment fails.
- [x] Fix scroll jank caused by drop-shadow-blur on every TicketCard.
- [x] Fix deferred rendering of IdCopy (TextEdit+Button widgets in every card).
