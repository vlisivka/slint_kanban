# Project Name: Slint Kanban

## Description
A Trello-like Kanban queue application built with Rust and Slint. It manages tasks using a file-system based approach, where queues are directories and tickets are sub-directories containing a `README.md` file.

## Tech Stack
-   **Language**: Rust
-   **GUI Framework**: Slint
-   **OS**: Linux (specifically Alma Linux 10)

## Communication with developer
-   **Primary Language**: Ukrainian (Українська)
-   **Secondary Language**: English (understand only)
-   **Preference**: The developer prefers communicating in Ukrainian.

## Architecture
-   **Data Storage**: File-system based.
    -   **Root Directory**: Default is `~/Kanban`. Can be overridden by the first command-line argument.
    -   **Tickets directory**: `~/Kanban/Ticket`
    - **Queues**: Sub-directories within `~/Kanban/Queue`. Sorted alphabetically by directory name. Names should start with a number and a dot (e.g., `1. Incoming`, `2. ToDo`) to ensure predictable ordering.
    -   **Tickets**: Symlinks from a queue directory to `~/Kanban/Tickets`.
    -   **Ticket Content**: `README.md` file inside the ticket directory.
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

## Functional Requirements
1.  **Kanban Board UI**: Visualize queues and tickets in columns.
2.  **Drag-and-Drop**: Move tickets between queues (directories) using drag-and-drop.
3.  **CRUD Operations**:
    -   Create new tickets.
    -   Read ticket content: Clicking on a ticket opens a full-window read-only view with Markdown rendering.
    -   Update ticket content: Editing is triggered by a dedicated "Edit" button. Allows editing raw text/YAML.
    -   Delete tickets (move them from `~/Kanban/Ticket` to `~/Kanban/Deleted` directory).
4.  **Ticket Interaction**:
    -   **Click**: Opens read-only details view with Markdown support.
    -   **Edit Button**: Opens the editor for raw `README.md` content.
5.  **Cross-Referencing**:
    -   Generate short IDs based on ticket title and creation date.
    -   Support linking between tickets using these IDs ad #tick12.
6.  **Limits on queues**:
    -  ToDo queue - no more than 21 item (configurable).
    -  Doing queue - no more than 5 items (configurable).

## Data Models
-   **Ticket ID Generation**: Short hash/ID (up to 6 chars, lowercase latin letters + numbers) derived from Title + Creation Date. Id is the name of ticket directory in `~/Kanban/Tickets` directory.
-   **Ticket metainfo**: is stored in README.md file in YAML format. 

## Non-Functional Requirements
- **Performance**: Efficient file system monitoring to reflect external changes.
- **Compatibility**: Optimized for Alma Linux 10.
- **Testing Requirements**:
    - **Unit Tests**: Mandatory for all core business logic in `model.rs`.
    - **GUI Tests**: Mandatory for critical UI interactions and state transitions using `slint::testing` and `i-slint-backend-testing`.
    - **Assertions**: All `assert!` and `assert_eq!` calls in tests MUST contain a descriptive message explaining the expected behavior and providing guidance on how to fix the issue if the assertion fails.

- **Initialization**: Automatically create the root directory and sub-directories (`Ticket`, `Queue`). Create default queues (`Incoming`, `ToDo`, `Doing`, `Reviewing`, `Testing`, `Done`, `Archive`) **only if no queues already exist** in the `Queue` directory.

## Future Enhancements

The following features are planned for future releases:

1. **Markdown Rendering**: Full Markdown rendering in ticket detail view with support for formatting, lists, code blocks, and other Markdown features.

2. **Queue Limits**: Configurable limits on the number of tickets per queue with visual indicators and enforcement:
   - ToDo queue: Maximum 21 items (configurable)
   - Doing queue: Maximum 5 items (configurable)
   - Visual warnings when approaching limits

3. **Cross-Reference Navigation (Implemented)**: Clickable links between tickets using short IDs (e.g., `#T-abc123`) with automatic detection and navigation.

4. **Search and Filter**: 
   - Full-text search across all tickets
   - Filter by queue, date range, or tags
   - Quick search with keyboard shortcuts

5. **Keyboard Shortcuts**: 
   - Navigation between queues and tickets
   - Quick ticket creation and editing
   - Customizable key bindings

6. **Theme Customization**:
   - Dark/light/system mode support
   - Custom color schemes
   - Font size and family preferences

7. **Export Functionality**:
   - Export board or individual tickets to PDF
   - HTML export for web viewing
   - Markdown export for documentation

8. **Statistics and Analytics**:
   - Dashboard with ticket metrics
   - Time tracking per queue
   - Completion rates and trends
   - Burndown charts for project tracking

