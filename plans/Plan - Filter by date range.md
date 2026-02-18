# Plan - Filter by Date Range

## Goal Description
Implement filtering of tickets by a date range (based on the `created_at` date). This allows users to focus on tickets created within a specific timeframe.

## Proposed Changes

### [Component Name] Data Model (model.rs)

#### [MODIFY] [model.rs](file:///home/vlisivka/workspace/slint_kanban/src/model.rs)
- Add `matches_date_range(from: &str, to: &str) -> bool` to `Ticket`.
- Dates in tickets are stored as strings: `%Y-%m-%d %H:%M:%S`.
- The matching logic should support partial dates (e.g., `2024-01-01`) by lexicographical comparison if the format is consistent.

### [Component Name] UI (app.slint)

#### [MODIFY] [app.slint](file:///home/vlisivka/workspace/slint_kanban/ui/app.slint)
- In `FilterMenu`:
    - Add "From Date" and "To Date" `LineEdit` fields.
    - Style them to fit in the compact menu.
- In `App`:
    - Add `in-out property <string> date_from: "";`
    - Add `in-out property <string> date_to: "";`
    - Add `callback date_filter_changed();`

### [Component Name] Sync & Business Logic (main.rs)

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Update `sync_ui_with_board` to accept `from` and `to` date strings.
- Update `on_search_edited` and other callers to use current date filters.
- Implement `on_date_filter_changed` to trigger a re-sync.

## Verification Plan

### Automated Tests
- Add a unit test in `model/tests.rs` for `matches_date_range` with various valid and invalid/empty dates.
- Add a test in `main_tests.rs` to verify that UI filtering works when properties are set.

### Manual Verification
1. Open the Filter Menu (⚙).
2. Enter a "From" date (e.g., `2024-01-01`).
3. Verify that only tickets created after that date are visible.
4. Enter a "To" date.
5. Verify both filters work together.
6. Clear dates and verify all tickets return.
