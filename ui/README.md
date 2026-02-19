# UI Structure (Slint Kanban)

This directory contains the Slint UI definitions for the Kanban application. The UI is modularized to improve maintainability and readability.

## Directory Layout

```text
ui/
├── app.slint               # Main window and coordination (entry point)
├── common.slint            # Shared structs (TicketStr, etc.) and global config
├── components/             # Core board elements
│   ├── ticket_card.slint   # Individual ticket visualization
│   └── kanban_column.slint # Column layout with header and ticket list
└── dialogs/                # Modal overlays and menus
    ├── ticket_view.slint       # Full ticket details view
    ├── ticket_edit.slint       # Ticket creation and editing interface
    ├── warning_dialog.slint    # Limit warning alert
    ├── queue_limit_edit.slint  # Queue limit configuration modal
    ├── filter_menu.slint       # Queue visibility and date filter dropdown
    └── search_history_menu.slint # Recent search queries dropdown
```

- `app.slint`: The main entry point for the UI. Defines the `App` window, handles global state, and coordinates between components and dialogs.
- `common.slint`: Contains shared data structures (`TicketStr`, `QueueStr`, `RefStr`) and global configuration (`AppConfig`).

### `/components`
Reusable UI elements that form the core of the board.
- `ticket_card.slint`: Individual ticket visualization with drag-and-drop triggers.
- `kanban_column.slint`: Represents a vertical queue, containing a list of `TicketCard`s and headers with limit indicators.

### `/dialogs`
Modal overlays and dropdown menus triggered by user actions.
- `ticket_view.slint`: Full-screen ticket details view.
- `ticket_edit.slint`: Interface for creating or modifying tickets.
- `warning_dialog.slint`: Alert for queue limits or other errors.
- `queue_limit_edit.slint`: Modal for setting/removing per-queue work-in-progress limits.
- `filter_menu.slint`: Dropdown for toggling queue visibility and applying date filters.
- `search_history_menu.slint`: Dropdown displaying recent search queries.

## Development Guidelines

1. **Shared Types**: Always import types from `../common.slint` instead of redefining them.
2. **Callbacks**: Use callbacks to bubble up events to the main `App` component in `app.slint`, where the actual logic integration with Rust happens.
3. **Styling**: Prefer using `Palette` and constants from `AppConfig` to maintain visual consistency.
4. **Layouts**: Use `VerticalBox` and `HorizontalBox` for standard spacing, or built-in `VerticalLayout`/`HorizontalLayout` for more granular control.

To add a new component, create it in the appropriate subdirectory and export it, then import it in `app.slint`.
