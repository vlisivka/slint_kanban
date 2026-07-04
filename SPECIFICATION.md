# Project Name: Slint Kanban

## Description
A Trello-like Kanban queue application built with Rust and Slint. It manages tasks using a file-system based approach, where queues are directories and tickets are sub-directories containing a `README.md` file.

## Tech Stack
-   **Language**: Rust
-   **GUI Framework**: Slint
-   **CLI Framework**: `clap`
-   **OS**: Linux (specifically Alma Linux 10)

## Communication with developer
-   **Primary Language**: Ukrainian (Українська)
-   **Secondary Language**: English (understand only)
-   **Preference**: The developer prefers communicating in Ukrainian.

## Development Process
-   **Planning**: Before starting implementation, a plan must be created in the `plans` directory.
-   **Format**: The plan should be a Markdown file named `Plan - NAME OF PLAN.md`.
-   **Approval**: The plan must be reviewed and approved by the developer before work begins.
-   **Development Lifecycle**: Before marking a task as complete, you MUST:
    -   Run all automated tests: `cargo test -- --test-threads=1 --nocapture`.
    -   Ensure all tests pass (unit, integration, CLI, and GUI tests).
    -   Format the code using `cargo fmt`.

## Architecture
-   **Data Storage**: File-system based.
    -   **Root Directory**: Default is `~/Kanban`. Can be overridden by the first command-line argument.
    -   **Board Overview**: A `README.md` file in the root directory (`~/Kanban/README.md`) that documents the purpose of the board, definitions for each queue, and the development process/rules.
    -   **Tickets directory**: `~/Kanban/Ticket`
    - **Queues**: Sub-directories within `~/Kanban/Queue`. Sorted alphabetically by directory name. Names should start with a number and a dot (e.g., `1.Incoming`, `2.ToDo`) to ensure predictable ordering.
    -   **Tickets**: Symlinks from a queue directory to `~/Kanban/Tickets`.
    -   **Ticket Content**: A directory containing:
        -   `README.md`: The main ticket file (metadata + description).
        -   `tc<NNN><UID>.md`: Comment files.
        -   `attachment/`: A sub-directory containing all files attached to either the ticket itself or its comments.
-   **Ticket Format**:
    -   **Header**: YAML frontmatter (Pandoc compatible) containing ticket metadata.
    -   **Body**: Markdown content describing the task.

### User Interface Details
- **Ticket Cards**:
    - Show Ticket Title.
    - Show Short ID.
    - Show date of creation.
    - Show date of last modification.
    - Show **only the first line** of the ticket's `README.md` body.
    - Text must not overflow the card boundaries (use ellipsis or clipping).
    - **Copyable ID**: The short ID must be easy to copy to the clipboard (e.g., via click or a copy button) to facilitate cross-referencing.

## Functional Requirements
1.  **Kanban Board UI**: Visualize queues and tickets in columns.
    -   **Board Documentation**: A dedicated button at the top of the window to open and view the root `README.md` (Board Overview) in a Markdown viewer.
2.  **Drag-and-Drop**: Move tickets between queues (directories) using drag-and-drop.
3.  **CRUD Operations**:
    -   Create new tickets.
    -   Read ticket content: Clicking on a ticket opens a full-window read-only view with Markdown rendering.
    -   Update ticket content: Editing is triggered by a dedicated "Edit" button. Allows editing raw text/YAML.
    -   **Delete tickets**: Move tickets from `~/Kanban/Ticket` to `~/Kanban/Deleted` directory (the Recycle Bin). A confirmation dialog must be shown before this action, explicitly stating that the ticket is being moved to the Recycle Bin.
4.  **Ticket Interaction**:
    -   **Click**: Opens read-only details view with Markdown support.
    -   **Edit Button**: Opens the editor for raw `README.md` content.
    -   **ID Interaction**: In both `TicketView` and `TicketEdit`, the ticket ID (prefixed with `#` for convenience) must be easily copyable to the clipboard for linking.
5.  **Cross-Referencing**:
    -   Generate short IDs based on ticket title and creation date.
    -   Support linking between tickets using these IDs (e.g., `#abc123`).
    -   IDs must be short, easy to enter, and consists only of lowercase letters and digits.
6.  **Limits on queues**:
    -  To Do queue - no more than 21 item (configurable).
    -  Doing queue - no more than 5 items (configurable).

7.  **Multi-user Support**:
    -   **Configurable Users**: A list of users is defined in `config.toml`. Defaults to `["<unassigned>", "user"]`.
    -   **Active User Selection**: Users can select their current identity via UI settings or CLI `configure`.
    -   **Ticket Assignment**: Each ticket has an `assigned_to` field. Can be set to a specific user or cleared (unassigned).
    -   **Ticket Author**: Each ticket MUST have an `author` field. The author is automatically set to the currently active user when the ticket is created.
    -   **Filtering**: Support for toggling between viewing all tickets and only those assigned to the current active user.
    -   **Comments**: Support for adding comments to tickets.
        -   Each comment is stored in a separate file within the ticket directory using the `tc<NNN><UID>.md` format.
        -   Comment files contain YAML frontmatter with `author`, `created_at`, `updated_at`, and optionally a list of `attachments`.
        -   Comments are sorted by `created_at` (ascending).
        -   If a comment mentions another ticket (e.g., `#abc123`), those references are displayed in a row below the comment content.
    -   **Attachments**: Support for attaching files to tickets or specific comments.
        -   Files are copied into the `attachment/` sub-directory of the ticket.
        -   Filename collisions are handled by appending a sequence number (e.g., `file (1).png`).
        -   The UI provides an "Attach File" button during editing tickets or creating comments, which opens a file selection dialog.
        -   Once attached, a Markdown link to the file (e.g., `[file (1).png](attachment/file (1).png)`) is generated and inserted into the ticket description or comment text.
        -   The total number of attachments is shown in the ticket view header. Clicking the "📁 Attachments (N)" opens the `attachment/` sub-directory in the host OS's native file manager.
    -   **Collaboration**: Designed for decentralized collaboration where files are synchronized via Git, Dropbox, or similar services.
    -   **Activity Logging**: Support for logging user actions (e.g., creating or editing tickets, commenting, attaching files, assigning users).
        -   To avoid conflicts during synchronization, each user logs their actions into a separate file using their active username and a unique machine ID: `Kanban/logs/log_${USER}_${MACHINE_ID}.md`.
        -   Logs use a Markdown table format (e.g., `| **Date** | **Action** | **Action description** | **JSON** |`) so they are easy to read and parse. 
        -   The `JSON` column stores the machine-readable payload of the action in JSON format (including the action type itself, e.g. `{"action": "...", "id": "..."}`) to simplify parsing the whole event into a single Enum. Placed at the end of the row, it does not clutter the human-readable text.
        -   When creating the file for the first time, a Markdown header and table definition are added. Subsequent entries are simply appended to the file without reading its previous contents.
    -   **Conflict Prevention and Resolution**:
        -   **Prevention**: To minimize race conditions, a "Manage Only My Tickets" (`manage_only_mine`) option is provided.
            -   When enabled (default for all non-admins), the user can only edit or move tickets assigned to them.
            -   This option can be toggled in the user settings.
            -   Administrators have this option disabled by default and can manage any ticket.
        -   **Queue Conflict Resolution**: In a decentralized setup, a ticket might end up in multiple queues or no queues due to synchronization delays.
            -   **Multiple Queues**: If a ticket symlink exists in multiple queue directories, the application resolves the conflict by keeping the symlink in the queue that is "furthest" from the start of the board (e.g., if it's in both "To Do" and "Done", it stays in "Done"). It is automatically removed from all other queues.
            -   **No Queues**: If a ticket exists in the `Tickets/` directory but is not referenced by any symlink in the `Queue/` directory, it is automatically added to the **first visible** queue on the board.

8.  **Command Line Interface (CLI)**:
    -   Non-interactive interface controlled via arguments and options.
    -   **Commands**:
        -   `add`: Create a new ticket (options: `--title`/`-t`, `--description`/`-d`, `--queue`/`-q`, `--assign-to`).
        -   `update`: Update existing ticket (options: `--id`/`-i`, `--title`/`-t`, `--description`/`-d`, `--assign-to`, `--unassign`).
        -   `move`: Move ticket to another queue (options: `--id`/`-i`, `--queue`/`-q`).
        -   `remove`: Delete ticket (options: `--id`/`-i`).
        -   `list`: List tickets with filters (options: `--assigned-to-user`, `--unassigned`, `--search`, `--id`, `--date-from`, `--date-to`).
        - `stats`: Show board statistics (options: `--user`, `--csv`).
        - `show`: Show detailed ticket info (options: `--id`/`-i`).
        - `configure`: Change settings (options: `--active-user`, `--show-only-mine`, `--add-user`).
        - `open PATH`: Open specified directory in the GUI.
        - `admin PATH`: Open the specified board in Administrator Mode.
        - `queue`: Manage queues (subcommands: `list`, `add`, `rename`, `delete`, `settings`, `tickets`).
    - **Administrator Mode**:
        - A specialized mode for board management, triggered via CLI: `slint_kanban admin PATH_TO_BOARD`.
        - Functional capabilities in this mode:
            - **Edit Board Documentation**: Edit the root `README.md` of the board directly within the application.
            - **User Management**: Add, remove, or modify the list of shared users in `config.toml`.
            - **Queue Management**: Add new queues, rename existing ones, or delete empty queues.
            - **Board Configuration**: Manage shared board settings like queue limits.
    - **Recycle Bin Management**:
        -   When tickets are deleted, they are moved directly to the host OS's native Recycle Bin/Trash (using the `trash` crate).
        -   There is no custom `~/Kanban/Deleted` directory anymore. Users manage their deleted items via their standard OS interfaces.
    -   **Testability**: Core logic must be decoupled from the `main` function to allow automated CLI testing.

## Data Models
-   **Ticket ID Generation**: Short hash/ID (up to 6 chars, lowercase latin letters + numbers) derived from Title + Creation Date. Id is the name of ticket directory in `~/Kanban/Tickets` directory.
-   **Ticket metainfo**: is stored in `README.md` file in YAML format. Contains `title`, `created_at`, `updated_at`, `assigned_to`, `author`, and `points`.
-   **Ticket references**: Found in both the ticket description (`README.md`) and comments (`tc*.md`). They are extracted and displayed as clickable links in a row directly below the respective content (the description or the individual comment).
-   **Ticket Comments**: Stored in separate files inside the ticket directory.
    -   **Filename**: `tc<NNN><UID>.md`. `NNN` is a 3-digit sequence (001, 002, etc.), and `UID` is a unique ID to avoid collisions.
    -   **Metadata**: stored in YAML frontmatter. Contains `author`, `created_at` and `updated_at`.
    -   **Content**: Markdown text.
-   **Attachments**: Files stored in the `attachment/` sub-directory.
    -   Associated by Markdown links inserted into the description of `README.md` or comments `tc*.md`.
- **Points (Estimation)**: Each ticket has a `points` field representing the complexity or effort required. The scale is defined as follows:
    - `1`: 1 working day or less.
    - `2`: 2 working days.
    - `3`: 3-4 working days.
    - `5`: 1 week.
    - `6`: 2 weeks.
    - `7`: 1 month.
    - `8`: 2-3 months.
    - `9`: 6 months.
    - `10`: 1 year.
- **Configuration Architecture**: Configuration is split into two files to separate shared board settings from local user preferences:
    - **Kanban Settings (`~/Kanban/config.toml`)**: Shared settings that should be synchronized (e.g., via Git).
        - `users`: List of shared user identities.
        - `queue_limits`: Mapping of queue names to WIP limits.
    - **User Settings (`~/.config/APP_NAME/user.toml`)**: Local preferences unique to each user/machine.
        - `active_user`: Currently selected local user identity.
        - `machine_id`: A unique, randomly generated short ID created upon first run to uniquely identify the machine, used for log files.
        -   `show_only_mine`: Flag to filter by `active_user`.
        -   `manage_only_mine`: Flag to restrict ticket management to assigned tickets only (default: true).
        -   `hidden_queues`: List of queues to hide from the board.
        - `search_history`: List of recent search queries.
        - `date_range`: Last used date filter range (from/to).
## Non-Functional Requirements
- **Performance**: Efficient file system monitoring to reflect external changes.
- **Compatibility**: Optimized for Alma Linux 10.
- **Testing Requirements**:
    - **Unit Tests**: Mandatory for all core business logic in `model.rs`.
    - **CLI Tests**: Mandatory for all CLI commands, testing via automated functions by calling a testable entry point (not `main`).
    - **GUI Tests**: Mandatory for critical UI interactions and state transitions using `slint::testing` and `i-slint-backend-testing`.
    - **Assertions**: All `assert!` and `assert_eq!` calls in tests MUST contain a descriptive message explaining the expected behavior and providing guidance on how to fix the issue if the assertion fails.

- **Initialization**: Automatically create the root directory and sub-directories (`Ticket`, `Queue`). Create default queues (`Incoming`, `To Do`, `Doing`, `Reviewing`, `Testing`, `Done`, `Archive`) **only if no queues already exist** in the `Queue` directory.

## CLI Commands

The application provides a command-line interface via `slint_kanban <command> [OPTIONS]`. All commands support `--help` for usage information.

### Global Options

| Option | Description |
|---|---|
| `-r, --root <PATH>` | Kanban root directory (defaults to `~/Kanban`) |
| `--admin` | Administrator mode (bypasses manage_only_mine restrictions) |

### `add` — Create a new ticket

```bash
slint_kanban add -t TITLE -q QUEUE [OPTIONS]
```

| Option | Short | Description |
|---|---|---|
| `--title <TITLE>` | `-t` | Ticket title (required) |
| `--description <TEXT>` | `-d` | Inline description text |
| `--description-file <PATH>` | `-D` | File to read description from (use `-` for stdin) |
| `--queue <QUEUE>` | `-q` | Target queue ID (e.g., `1.Incoming`, `2.ProductBacklog`) (required) |
| `--assign-to <USER>` | `-a` | Assigned user [default: empty] |
| `--points <N>` | `-p` | Estimation points 1–10 [default: 0] |

**Description resolution priority:**
1. `-D -` (stdin) — reads from stdin, ignores `-d`
2. `-D <file>` — reads from file; if `-d` also provided → concatenates: `[-d]\n[<file>]`
3. `-d <text>` — uses inline text
4. No flags → empty description

**Examples:**
```bash
# Inline description (backward compatible)
slint_kanban add -t "Fix login bug" -q "1.Incoming" -d "User cannot log in"

# Read from file
slint_kanban add -t "Fix login bug" -q "1.Incoming" -D ticket-body.md

# Read from stdin (pipeline)
cat template.md | slint_kanban add -t "Fix login bug" -q "1.Incoming" -D -

# Concatenate inline + file
slint_kanban add -t "Fix login bug" -q "1.Incoming" -d "Context:" -D ticket-body.md
```

### `update` — Update an existing ticket

```bash
slint_kanban update -id TICKET_ID [OPTIONS]
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-id` | Ticket short ID (required) |
| `--title <TITLE>` | `-t` | New title (preserved if omitted) |
| `--description <TEXT>` | `-d` | New description (preserved if omitted) |
| `--assign-to <USER>` | `-a` | Assign to user |
| `--unassign` | | Clear assignment |
| `--points <N>` | `-p` | New points value (preserved if omitted) |

### `move` — Move ticket between queues

```bash
slint_kanban move -id TICKET_ID -q QUEUE
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-id` | Ticket short ID (required) |
| `--queue <QUEUE>` | `-q` | Target queue ID (required) |


### `queue` — Manage queues

```bash
slint_kanban queue <ACTION> [OPTIONS]
```

Actions: `list`, `add`, `rename`, `delete`, `settings`, `tickets`

#### `queue list`

List all queues with their settings.

```bash
slint_kanban queue list
```

Output example:
```
[1.Incoming] Incoming — 3 tickets (limit: 0)
[2.ProductBacklog] Product Backlog — 5 tickets
```

#### `queue add`

Add a new queue (requires `--admin`).

```bash
slint_kanban --admin queue add --name "4.InReview"
```

| Option | Description |
|---|---|
| `--name <NAME>` | New queue name (e.g., `4.InReview`) |

#### `queue rename`

Rename a queue (requires `--admin`).

```bash
slint_kanban --admin queue rename --id "1.Incoming" --name "0.Intake"
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-i` | Current queue ID or name (required) |
| `--name <NAME>` | | New queue name (required) |

#### `queue delete`

Delete an empty queue (requires `--admin`). Fails if queue has tickets.

```bash
slint_kanban --admin queue delete --id "9.Archive"
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-i` | Queue ID to delete (required) |

#### `queue settings`

View or set queue settings.

```bash
# View settings
slint_kanban queue settings -i "2.Incoming"

# Set limit (requires --admin)
slint_kanban --admin queue settings -i "3.SprintBacklog" -l 50
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-i` | Queue ID or name (required) |
| `--limit <N>` | `-l` | Set WIP limit for queue (admin only) |

View output shows: queue name, ID, current limit, visibility, ticket count.

#### `queue tickets`

List tickets in a queue with optional filters.

```bash
# Basic listing
slint_kanban queue tickets -i "2.Incoming"

# Verbose mode
slint_kanban queue tickets -v -i "2.Incoming"

# Filter by date
slint_kanban queue tickets -i "3.SprintBacklog" --after "2026-07-03"
slint_kanban queue tickets -i "3.SprintBacklog" --after "2026-07-03_15:20"

# Filter by time window
slint_kanban queue tickets -i "4.InProgress" --last-hour
slint_kanban queue tickets -i "4.InProgress" --last-day

# Filter by assignee
slint_kanban queue tickets -i "4.InProgress" --assigned-to user
slint_kanban queue tickets -i "4.InProgress" --assigned-to-me
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-i` | Queue ID or name (required) |
| `--verbose` | `-v` | Show detailed ticket info (created/updated at, points, author, assignee) |
| `--after <DATE>` | | Filter by date after (YYYY-MM-DD or YYYY-MM-DD_HH:MM) |
| `--last-hour` | | Show only tickets updated in the last hour |
| `--last-day` | | Show only tickets updated in the last day |
| `--assigned-to <USER>` | | Filter by assigned user |
| `--assigned-to-me` | | Show only tickets assigned to active user |

Basic output: `[ID] Title [N pts]` per ticket.
Verbose output: `[ID] Title (Created: ..., Updated: ..., Points: N, By: ..., Assigned to: ...)`
### `remove` — Delete a ticket

```bash
slint_kanban remove -id TICKET_ID
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-id` | Ticket short ID (required) |

### `list` — List tickets with filters

```bash
slint_kanban list [OPTIONS]
```

| Option | Short | Description |
|---|---|---|
| `--search <QUERY>` | `-s` | Full-text search |
| `--id <ID>` | `-id` | Filter by ticket ID |
| `--assigned-to <USER>` | `-a` | Filter by assigned user |
| `--unassigned` | | Show only unassigned tickets |
| `--date-from <DATE>` | `-f` | Show tickets created after date (YYYY-MM-DD) |
| `--date-to <DATE>` | `-t` | Show tickets created before date (YYYY-MM-DD) |

### `show` — Show detailed ticket information

```bash
slint_kanban show -id TICKET_ID
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-id` | Ticket short ID (required) |

### `configure` — Manage configuration

```bash
slint_kanban configure [ACTION] [OPTIONS]
```

Actions: `add-user`, `remove-user`, `set-active`, `set-queue-limit`, `hide-queue`, `show-queues`

### `stats` — Show board statistics

```bash
slint_kanban stats [OPTIONS]
```

| Option | Short | Description |
|---|---|---|
| `--csv` | | Export to CSV format |

### `sprint` — Manage sprints

```bash
slint_kanban sprint <ACTION> [OPTIONS]
```

Actions: `list`, `current`, `add`, `update`, `remove`

### `comment` — Add a comment to a ticket

```bash
slint_kanban comment -id TICKET_ID -c CONTENT
```

| Option | Short | Description |
|---|---|---|
| `--id <ID>` | `-id` | Ticket short ID (required) |
| `--content <TEXT>` | `-c` | Comment content (required) |

### `attach` — Manage attachments

```bash
slint_kanban attach [ACTION] [OPTIONS]
```

Actions: `add`, `list`, `show`, `open`

| Option | Short | Description |
|---|---|---|
| `--file <PATH>` | `-f` | File to attach (for `add` action) |

### `open` — Open GUI at specific path

```bash
slint_kanban open [PATH]
```
## Future Enhancements

The following features are planned for future releases:

1. **Markdown Rendering**: Full Markdown rendering in ticket detail view with support for formatting, lists, code blocks, and other Markdown features.

2. **Queue Limits**: Configurable limits on the number of tickets per queue with visual indicators and enforcement:
   - To Dooooo queue: Maximum 21 items (configurable)
   - Doing queue: Maximum 5 items (configurable)
   - Visual warnings when approaching limits

3. **Cross-Reference Navigation (Implemented)**: Clickable links between tickets using short IDs (e.g., `#abc123`) with automatic detection and navigation.

4. **Search and Filter**: 
   - Full-text search across all tickets
   - Filter by queue, date range, or tags
   - Quick search with keyboard shortcuts

5. **Keyboard Shortcuts**: 
   - `Ctrl+F`: Focus search input.
   - `Ctrl+N`: Create new ticket in the first visible queue.
   - `Ctrl+M`: Toggle between viewing only assigned tickets and all tickets.
   - `Esc`: Close dialogs or clear and unfocus the search field.
   - `Down Arrow` (in search field): Select from search history.

6. **Theme Customization**:
   - Dark/light/system mode support
   - Custom color schemes
   - Font size and family preferences

7. **Export Functionality**:
   - Export board or individual tickets to PDF
   - HTML export for web viewing
   - Markdown export for documentation

8. **Statistics and Analytics**:
   - Snapshot-metrics (Ticket count per queue and per user) and Agile metrics (Average Lead Time, Average Cycle Time) are implemented and accessible via CLI `stats` command and the `📊 Statistics` button in the GUI.
   - Dashboard with ticket metrics
   - Time tracking per queue is implemented via log parsing.
    - Completion rates and trends
    - Burndown charts are implemented for the current sprint, showing remaining vs ideal points.

9. **Internationalization (i18n)**:
    - **Gettext Support**: The application uses the Gettext `.po` format for translations.
    - **Marking Strings**: Translatable strings are marked using `@tr()` in `.slint` files and `tr!` in `.rs` files.
    - **Extraction Workflow**: A script is used to extract strings from source files into a `.pot` template.
    - **Runtime Localization**: Translations are loaded at runtime from `.mo` files or bundled into the executable.
    - **Supported Languages**: Initially target Ukrainian (uk).
