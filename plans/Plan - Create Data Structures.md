# Plan - Create Data Structures

## Meta
**Goal**: Define the core data structures (`Ticket`, `Queue`, `Board`) for the Slint Kanban application and implement their serialization/deserialization logic, respecting the new file system architecture (symlinks, split directories).

## Changes

### 1. `src/model.rs`
-   **Create File**: New file to hold data models.
-   **Struct `Ticket`**:
    -   `id`: String (matches directory name in `~/Kanban/Tickets`).
    -   `title`: String (from YAML frontmatter).
    -   `created_at`: String (or DateTime, from YAML).
    -   `description`: String (Markdown body).
-   **Struct `Queue`**:
    -   `id`: String (matches directory name in `~/Kanban/Queue`).
    -   `name`: String (display name, same as id for now).
    -   `tickets`: Vec<Ticket>.
    -   `max_tickets`: Option<u32> (for limits: To Do=21, Doing=5).
-   **Struct `Board`**:
    -   `queues`: Vec<Queue>.
    -   `tickets_path`: PathBuf (`~/Kanban/Tickets`).
    -   `queues_path`: PathBuf (`~/Kanban/Queue`).
-   **Serialization**:
    -   Use `serde` for YAML frontmatter parsing.
    -   Define a helper struct `TicketMetadata` for the frontmatter part only.

### 2. `src/lib.rs` (or `src/main.rs`)
-   Declare `mod model;`.

## Verification

### Automated Tests
-   **Test YAML Parsing**: Create a test case with sample `README.md` content (YAML + Body) and verify `Ticket` is correctly parsed.
-   **Test Queue/Ticket Structure**: Ensure relationships hold.

### Manual Verification
-   None for this step (pure logic).
