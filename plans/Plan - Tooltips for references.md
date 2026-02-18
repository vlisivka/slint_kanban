# Plan: Show Tooltip with Title on Hover for References

## Goal
Show the title of a referenced ticket when the user hovers over its ID in the ticket detail view.

## Proposed Changes

### [Component Name] Model and Data Mapping

#### [MODIFY] [app.slint](file:///home/vlisivka/workspace/slint_kanban/ui/app.slint)
- Define `RefStr` struct with `id` (string) and `title` (string).
- Update `TicketStr` to use `[RefStr]` as the type for its `references` property.

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Update the `ticket_to_slint` function to accept a `&Board` argument so it can resolve reference IDs.
- In `ticket_to_slint`, for each extracted reference, look up the corresponding ticket title using `board.find_ticket_by_id`.
- Update `sync_ui_with_board` to pass the `board` to `ticket_to_slint`.

### [Component Name] UI Implementation

#### [MODIFY] [app.slint](file:///home/vlisivka/workspace/slint_kanban/ui/app.slint)
- **State Management**: In the `App` component, add properties to track the tooltip state:
  - `tooltip_title: string`
  - `tooltip_visible: bool`
  - `tooltip_x: length`
  - `tooltip_y: length`
- **Interaction**: In `TicketView`, replace the reference buttons with a more interactive component or add `TouchArea` logic:
  - Add `on-hover` or `TouchArea` events to set `tooltip_title` and `tooltip_visible = true` when hovering over a reference.
  - Update `tooltip_x` and `tooltip_y` with mouse position.
- **Visuals**: Add a floating `Rectangle` (tooltip) at the end of the `App` component's body that displays `tooltip_title` if `tooltip_visible` is true.

## Verification Plan

### Automated Tests
- Update `test_ui` in `src/main_tests.rs` to verify that `TicketStr` now contains `RefStr` with correctly resolved titles.

### Manual Verification
1. Create a ticket named "Dependency Task".
2. Create another ticket and include a reference to the first one (e.g., `#abc123`) in its description.
3. Open the second ticket in the GUI.
4. Hover over the `#abc123` reference.
5. Verify that a tooltip appears with the text "Dependency Task".
