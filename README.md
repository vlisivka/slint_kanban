# Slint Kanban

A modern, file-system-based Kanban board application built with Rust and Slint UI framework. Manage your tasks with a beautiful, native desktop interface while keeping all your data in plain text files that you own and control.

![Version](https://img.shields.io/badge/version-0.2.0-blue)
![Rust](https://img.shields.io/badge/rust-1.70+-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## ✨ Features

### 🎯 Core Functionality
- **Visual Kanban Board**: Clean, intuitive column-based task management interface
- **Drag & Drop**: Seamlessly move tickets between queues with mouse interactions
- **Real-time Synchronization**: File watcher auto-reloads the board when files change on disk
- **File-System Storage**: All data stored as plain text files — you own your data
- **YAML Frontmatter**: Pandoc-compatible metadata in Markdown files
- **Cross-Referencing**: Link tickets using short IDs (e.g., `#tick12`) with clickable navigation
- **Full CLI**: Manage your board entirely from the command line

### 📝 Ticket Management
- **Create**: Quick ticket creation with title, description, assignee, and estimation points
- **View**: Read-only detail view with full ticket content, comments, references, and attachments
- **Edit**: Edit ticket title, description, assignee, and points
- **Delete**: Safe deletion via the OS Recycle Bin (using the `trash` crate)
- **Timestamps**: Automatic creation and modification date tracking
- **Comments**: Add threaded comments to any ticket (stored as separate `tc*.md` files)
- **Attachments**: Attach files to tickets or comments, opened via native file manager
- **Points (Estimation)**: Estimate effort on a 1–10 scale (1 = 1 day, 5 = 1 week, 10 = 1 year)
- **Copyable ID**: Click the 📋 button to copy a ticket's short ID to the clipboard

### 📊 Statistics & Analytics
- **Board Summary**: Total tickets, points, and per-queue/per-user breakdown
- **Agile Metrics**: Average Lead Time and Cycle Time calculated from activity logs
- **Completion Rate**: Percentage of tickets completed overall and per sprint
- **Trend Visualization**: Bar chart of ticket/point trends over the last 15 days
- **Sprints**: Define sprints with name, start/end dates; track sprint completion rate
- **Activity Logging**: All user actions (create, move, comment, attach, assign) are logged per-user to `logs/log_${USER}_${MACHINE_ID}.md`

### 🔍 Search & Filter
- **Full-text Search**: Search across ticket titles and descriptions with debounced input
- **Date Range Filter**: Filter tickets by creation/modification date
- **User Filter**: Toggle between viewing all tickets and only those assigned to the current user
- **Search History**: Recent queries saved and accessible via dropdown

### ⌨️ Keyboard Shortcuts
| Shortcut | Action |
|----------|--------|
| `Ctrl+F` | Focus search input |
| `Ctrl+N` | Create new ticket in the first visible queue |
| `Ctrl+M` | Toggle "Show only mine" filter |
| `Esc`    | Close dialog / clear search |
| `↓`      | Open search history (when search is focused) |

### 👥 Multi-User Support
- **Configurable Users**: User list defined in `config.toml`
- **Active User Selection**: Switch identity via UI or CLI
- **Ticket Assignment**: Assign tickets to specific users or leave unassigned
- **Ticket Author**: Automatically set to the active user on creation
- **Collaboration**: Designed for decentralized collaboration via Git, Dropbox, or similar

### 🎨 User Interface
- **Native Performance**: Built with Slint for smooth, responsive UI
- **System Theme**: Follows system dark/light mode
- **Queue Limits (WIP)**: Visual indicators when queues approach or exceed configured limits
- **Customizable Queues**: Queues are filesystem directories — add as many as you need
- **Board Info**: View the root `README.md` (board documentation) from within the app

## 🏗️ Architecture

### File System Structure

```
~/Kanban/
├── Queue/                            # Kanban queues (columns)
│   ├── 1. Incoming/                  # Symlinks to ticket directories
│   ├── 2. To Do/
│   ├── 3. Doing/
│   ├── 4. Reviewing/
│   ├── 5. Testing/
│   ├── 6. Done/
│   └── 7. Archive/
├── Tickets/                          # Actual ticket storage
│   ├── abc123/                       # Ticket directory (short ID)
│   │   ├── README.md                 # Ticket content with YAML frontmatter
│   │   ├── tc001xyz.md               # Comment file
│   │   └── attachment/               # Attached files
│   └── def456/
│       └── README.md
├── logs/                             # Activity logs (per user per machine)
│   └── log_user_a1b2c3.md
├── sprints.toml                      # Sprint definitions
├── config.toml                       # Shared board settings (users, WIP limits)
└── README.md                         # Board documentation

~/.config/slint-kanban/
└── user.toml                         # Local user settings (active user, search history, hidden queues)
```

### Configuration Architecture

Configuration is split into two files to separate shared board settings from local user preferences:

| File | Scope | Contains |
|------|-------|----------|
| `~/Kanban/config.toml` | Shared (sync via Git) | `users`, `queue_limits` |
| `~/.config/slint-kanban/user.toml` | Local (per machine) | `active_user`, `machine_id`, `show_only_mine`, `hidden_queues`, `search_history`, `date_range` |

### Ticket Format

Each ticket is a directory containing a `README.md` with YAML frontmatter:

```markdown
---
title: Implement user authentication
created_at: "2026-02-17 10:30:00"
updated_at: "2026-02-17 12:15:00"
assigned_to: user
author: user
points: 3
---

Detailed description of the task in Markdown format.
References to other tickets like #abc123 are auto-detected.
```

### Data Models

- **Ticket ID**: Up to 6 characters (lowercase letters + digits), derived from title + creation date. The ID is the name of the ticket directory.
- **Comments**: Stored as `tc<NNN><UID>.md` files with YAML frontmatter (`author`, `created_at`, `updated_at`).
- **Attachments**: Files stored in `attachment/` sub-directory, referenced via Markdown links.
- **Points Scale**: 1 = 1 day, 2 = 2 days, 3 = 3–4 days, 5 = 1 week, 6 = 2 weeks, 7 = 1 month, 8 = 2–3 months, 9 = 6 months, 10 = 1 year.

## 🚀 Installation

### Prerequisites

- **Rust**: 1.70 or later
- **Linux**: Optimized for Alma Linux 10 (works on other distributions)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/user/slint_kanban.git
cd slint_kanban

# Build and install
cargo install --path=.

# Run the application (uses ~/Kanban by default)
slint_kanban
```

## 📖 Usage

### GUI Mode

```bash
# Use default location (~/Kanban)
slint_kanban

# Specify custom Kanban directory
slint_kanban --root /path/to/your/kanban
```

On first launch, the application automatically creates the default directory structure and seven standard queues.

### Working with Tickets (GUI)

1. **Create a Ticket**: Click the "+" button in any queue column
2. **View Details**: Click on a ticket card to open the detail view
3. **Edit**: Click "Edit" in the detail view to modify title, description, assignee, and points
4. **Move**: Drag and drop tickets between queue columns
5. **Delete**: Click the "×" button on a card (ticket moves to the OS Recycle Bin)
6. **Comment**: Add comments from within the ticket detail view
7. **Attach File**: Click "Attach" to add files via file dialog
8. **Copy ID**: Click the 📋 button next to the ticket ID to copy it to the clipboard
9. **Navigate References**: Click on `#abc123` links to jump to referenced tickets

### CLI Mode

```bash
# Add a ticket
slint_kanban add -t "Fix login bug" -d "Users can't log in" -q "2. To Do" --assign-to user -p 3

# List all tickets
slint_kanban list

# List tickets assigned to a user
slint_kanban list --assigned-to-user user

# Search tickets
slint_kanban list --search "login"

# Show ticket details
slint_kanban show -i abc123

# Move a ticket to another queue
slint_kanban move -i abc123 -q "3. Doing"

# Update a ticket
slint_kanban update -i abc123 -t "New title" --assign-to admin -p 5

# Delete a ticket
slint_kanban remove -i abc123

# Add a comment
slint_kanban comment -i abc123 -c "This is fixed now"

# Attach a file
slint_kanban attach -i abc123 -f /path/to/file.png

# List attachments
slint_kanban attach -i abc123 --list

# View board statistics
slint_kanban stats

# Manage sprints
slint_kanban sprint list
slint_kanban sprint current
slint_kanban sprint add --name "Sprint 1" --start 2026-02-17 --end 2026-03-02

# Configure settings
slint_kanban configure --active-user admin
slint_kanban configure --add-user newuser
slint_kanban configure --show-only-mine true
```

### External Editing

You can edit tickets using any text editor:

```bash
# Edit a ticket directly
vim ~/Kanban/Tickets/abc123/README.md

# The UI will automatically refresh when you save
```

### Custom Queues

Create custom queues by adding directories:

```bash
mkdir ~/Kanban/Queue/"6. Review"
mkdir ~/Kanban/Queue/"7. Testing"
```

The application automatically detects and displays new queues, sorted alphabetically.

## 🛠️ Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Language | Rust 2021 Edition | Core application |
| UI Framework | [Slint](https://slint.dev/) 1.15 | Desktop GUI |
| CLI | clap 4.5 | Command-line interface |
| Serialization | serde + serde_yaml + serde_json + toml | Data parsing |
| File Watching | notify 8.2 | Real-time sync |
| Date/Time | chrono 0.4 | Timestamps |
| Error Handling | anyhow 1.0 | Error propagation |
| Clipboard | arboard 3.4 | System clipboard |
| File Dialogs | rfd 0.17 | Native file picker |
| Recycle Bin | trash 5.2 | Safe deletion |
| File Manager | open 5.3 | Open folders in OS |

## 🎯 Roadmap

### Implemented ✅
- Kanban board with drag & drop
- CRUD operations for tickets
- File watcher with debounced reloading
- Queue limits (WIP) with visual indicators
- Cross-reference navigation between tickets
- Full CLI with all commands
- Multi-user support with filtering
- Ticket comments and attachments
- Points (estimation) system
- Search with history and date range filter
- Keyboard shortcuts
- Statistics and analytics dashboard
- Activity logging
- Sprints management
- Incremental UI updates and caching
- System Recycle Bin integration
- Split configuration (shared + local)
- Clipboard integration

### Planned 🔜
- [ ] Burndown charts
- [ ] Export statistics to CSV
- [ ] Export tickets/board to PDF, HTML, Markdown
- [ ] Markdown rendering in ticket detail view (via Servo WebView)
- [ ] Conflict handling for multi-user/multi-machine setups
- [ ] Separate admin GUI/CLI for board management
- [ ] Internationalization (i18n) — Ukrainian language support

## 🛠️ Development & Testing

### Running Tests

```bash
# Run all tests (must use single thread due to shared test state)
cargo test -- --test-threads=1 --nocapture
```

### Code Style

Always run `cargo fmt` before committing:

```bash
cargo fmt
```

### Project Structure

```
src/
├── main.rs          # Entry point, GUI setup, callbacks, CLI dispatch
├── lib.rs           # Slint type conversions (TicketStr, BoardSummaryStr)
├── cli.rs           # CLI argument definitions (clap)
├── controller.rs    # AppController: bridge between UI and Board model
└── model/
    ├── mod.rs       # Module re-exports
    ├── board.rs     # Board: load, save, move, create, delete operations
    ├── ticket.rs    # Ticket and TicketMetadata structs, parsing
    ├── queue.rs     # Queue struct
    ├── comment.rs   # Comment parsing and creation
    ├── config.rs    # Split config (KanbanConfig + UserConfig)
    ├── stats.rs     # Statistics, trends, lead/cycle time calculations
    ├── action.rs    # Activity logging (ActionPayload enum)
    └── tests/       # Unit and integration tests

ui/
├── app.slint            # Main application window
├── common.slint         # Shared structs, globals, theme
├── stats_view.slint     # Statistics dashboard
├── sprints_view.slint   # Sprints list view
├── components/
│   ├── ticket_card.slint    # Individual ticket card
│   └── kanban_column.slint  # Queue column with scrollable ticket list
└── dialogs/
    ├── ticket_view.slint        # Read-only ticket detail view
    ├── ticket_edit.slint        # Ticket editor
    ├── delete_confirm_dialog.slint
    ├── warning_dialog.slint
    ├── queue_limit_edit.slint
    ├── filter_menu.slint
    └── search_history_menu.slint
```

### Build Profiles

The project includes optimized build profiles for development on resource-constrained hardware:

- **`dev`**: `opt-level = 1` (faster runtime), `debug = 1` (line tables only — works with `dbg!()` and backtraces), `incremental = false` (saves disk space)
- **`release`**: `opt-level = "z"` (size-optimized), `strip = true`, `panic = "abort"`, `lto = true`, `codegen-units = 1`

## 📄 License

This project is licensed under the MIT License — see the LICENSE file for details.

## 🙏 Acknowledgments

- Built with [Slint UI](https://slint.dev/) — a declarative GUI toolkit for Rust
- Inspired by Trello and other Kanban board applications
- Thanks to the Rust community for excellent tooling and libraries

---

**Made with ❤️ using Rust and Slint**
