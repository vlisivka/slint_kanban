# Project Code Structure

This document provides an overview of the Slint Kanban project's directory structure and the purpose of each file.

## Root Directory

- `src/`: Rust backend source code.
- `ui/`: Slint frontend definitions.
- `plans/`: Implementation plans and architectural documentation.
- `TODO.md`: Project roadmap and task list.
- `Cargo.toml`: Rust project dependencies and configuration.

## Backend Structure (`src/`)

```text
src/
├── main.rs                 # Entry point, UI orchestration, and event handlers.
├── main_tests.rs           # Integration tests for UI and CLI flows.
├── cli.rs                  # Command-line argument parsing and CLI mode logic.
└── model/                  # Data model and core business logic.
    ├── mod.rs              # Module entry point and public type re-exports.
    ├── board.rs            # Board orchestration (loading, moving, creating tickets).
    ├── config.rs           # App configuration (limits, visibility, search history).
    ├── queue.rs            # Queue (column) data structure.
    ├── ticket.rs           # Ticket and Metadata structures with parsing logic.
    └── tests/              # Unit tests for the model.
        ├── mod.rs          # Test module entry.
        ├── board_tests.rs  # Tests for Board operations.
        ├── config_tests.rs # Tests for Config and history.
        └── ticket_tests.rs # Tests for Ticket parsing and matching.
```

## Frontend Structure (`ui/`)

```text
ui/
├── app.slint               # Main application window and UI component assembly.
├── common.slint            # Shared Slint structs and global UI configuration.
├── README.md               # Detailed documentation of the UI components.
├── components/             # Reusable board elements.
│   ├── kanban_column.slint # Column visualization with headers.
│   └── ticket_card.slint   # Individual ticket visualization and drag-and-drop triggers.
└── dialogs/                # Modal windows and dropdown menus.
    ├── filter_menu.slint       # Queue visibility and date filter dropdown.
    ├── queue_limit_edit.slint  # Modal for configuring WIP limits.
    ├── search_history_menu.slint # dropdown for recent search queries.
    ├── ticket_edit.slint       # Interface for creating and editing tickets.
    ├── ticket_view.slint       # Detailed ticket information view.
    └── warning_dialog.slint    # Alert dialog for limit violations.
```

## Deployment & Documentation

- `build.rs`: Compiles Slint files into Rust code during the build process.
- `.agent/`: Internal agent configurations and workflows.
- `plans/`: Detailed step-by-step plans for major feature implementations.
