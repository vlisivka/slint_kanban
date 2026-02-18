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

### Command Line Interface (CLI)
- [x] Refactor `main.rs` to move application logic into a testable function.
- [x] Implement CLI argument parsing using `clap`.
- [x] Implement non-interactive commands:
    - [x] `add`: create new ticket with options for title, description, and queue.
    - [x] `update`: update ticket attributes by ID.
    - [x] `move`: move ticket between queues by ID.
    - [x] `remove`: delete ticket by ID.
- [x] Implement `open PATH` command to control GUI from command line.
- [x] Add automated tests for all CLI commands.

### Search and Filter
- [x] Implement search functionality
  - [x] Add search input field to UI
  - [x] Full-text search across ticket titles and descriptions
  - [x] Filter by queue
  - [ ] Filter by date range
  - [ ] Highlight search results
  - [ ] Search history

### Keyboard Shortcuts
- [ ] Implement keyboard navigation
  - [ ] Arrow keys for navigation between tickets
  - [ ] Shortcuts for creating new tickets (Ctrl+N)
  - [ ] Shortcuts for editing (Ctrl+E)
  - [ ] Shortcuts for deleting (Delete key)
  - [ ] Quick search (Ctrl+F)
  - [ ] Customizable key bindings configuration

### Theme Customization
- [ ] Implement theming support
  - [ ] Dark mode implementation
  - [ ] Light mode implementation
  - [ ] Theme switcher in UI
  - [ ] Custom color scheme support
  - [ ] Font customization (size, family)
  - [ ] Save theme preferences

### Export Functionality
- [ ] Implement export features
  - [ ] Export single ticket to PDF
  - [ ] Export entire board to PDF
  - [ ] Export to HTML format
  - [ ] Export to Markdown
  - [ ] Batch export functionality
  - [ ] Export configuration options

### Statistics and Analytics
- [ ] Implement analytics dashboard
  - [ ] Ticket count per queue
  - [ ] Time tracking per ticket/queue
  - [ ] Completion rate calculations
  - [ ] Trend visualization
  - [ ] Burndown charts
  - [ ] Export statistics to CSV

### Markdown Rendering
- [/] Implement Markdown rendering in ticket detail view using Servo
  - [ ] Integrate markdown rendering library (pulldown-cmark or similar)
  - [ ] Copy WebView component from Slint "servo" example into project and use it: https://github.com/slint-ui/slint/tree/master/examples/servo
  - [ ] Create styled text rendering component
  - [ ] Support for headings, lists, code blocks
  - [ ] Support for links and images

