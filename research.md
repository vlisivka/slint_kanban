# Research: Implementing Ticket Points (Estimation)

## Overview
The goal is to add a "Points" field to tickets to represent complexity or effort. The scale is 1-10 with predefined time mappings.

## Data Model Changes
### `src/model/ticket.rs`
1. **`TicketMetadata`**: Add `#[serde(default)] pub points: u32`.
2. **`Ticket`**: Add `pub points: u32`.
3. **`Ticket::from_metadata`**: Copy points from metadata.
4. **`Ticket::save`**: Include `points` in the YAML frontmatter and `write!` call.
5. **`Ticket::load`**: `points` will be automatically parsed by `serde_yaml` since it's in `TicketMetadata`.

## UI Changes
### `ui/common.slint`
1. **`TicketStr`**: Add `points: int`.

### `ui/dialogs/ticket_edit.slint`
1. Add `in-out property <int> points`.
2. Add a `ComboBox` for selecting points (1 to 10).
3. Update the `save` callback signature: `callback save(string, string, string, string, int)`.

### `ui/dialogs/ticket_view.slint`
1. Display points in the header, preferably with the time mapping (e.g., "5 pts (~1 week)").

### `ui/components/ticket_card.slint`
1. Display a small badge or icon with the points count.

### `ui/app.slint`
1. Handle the updated `save` callback.
2. Update the `active_ticket` synchronization.

## Logic Changes
### `src/main.rs`
1. **`into_slint_ticket`**: Map `ticket.points` to `TicketStr.points`.
2. **`handle_command`** (CLI): Add `--points` to `add` and `update` commands.
3. Update `App` callbacks and state handling.

### `src/controller.rs`
1. Update `handle_create_ticket` and `handle_update_ticket` to accept and save points.

### `src/model/stats.rs`
1. Include `total_points` in `BoardSummary`.
2. Calculate total points per user and per sprint.

## Time Mapping (Helper for UI)
| Points | Time Mapping |
|--------|--------------|
| 1      | 1 day or less|
| 2      | 2 days       |
| 3      | 3-4 days     |
| 5      | 1 week       |
| 6      | 2 weeks      |
| 7      | 1 month      |
| 8      | 2-3 months   |
| 9      | 6 months     |
| 10     | 1 year       |

## Plan for Implementation
1. **Step 1**: Update `model.rs` and `ticket.rs` (Data structures & serialization).
2. **Step 2**: Update `common.slint` and `TicketStr` conversion in `main.rs`.
3. **Step 3**: Update `TicketEdit` UI and save logic.
4. **Step 4**: Update `TicketCard` and `TicketView` UI.
5. **Step 5**: Implement CLI support.
6. **Step 6**: Update statistics to include points.
7. **Step 7**: Verify with tests.
