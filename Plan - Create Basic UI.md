# Plan - Create Basic UI

## Meta
**Goal**: Create a Trello-like user interface using Slint, displaying queues (columns) and tickets (cards) in a horizontal scrolling layout.

## Changes

### 1. `ui/app.slint`
-   **Data Structures**:
    -   `struct TicketStr`: id, title, description.
    -   `struct QueueStr`: id, name, tickets: [TicketStr].
-   **Component `TicketCard`**:
    -   Input: `ticket: TicketStr`.
    -   Styling: White background, rounded corners, slight shadow/border.
    -   Content: Title (bold), ID (small/gray).
-   **Component `KanbanColumn`**:
    -   Input: `queue: QueueStr`.
    -   Styling: Light gray background, rounded corners.
    -   Layout: VerticalBox.
    -   Content: header (queue name), ListView of `TicketCard`.
-   **`MainWindow`**:
    -   Property: `board_queues: [QueueStr]`.
    -   Layout: ScrollView (horizontal) containing a HorizontalLayout of `KanbanColumn`s.

### 2. `src/main.rs`
-   **Adapter Logic**:
    -   Convert Rust `Board` -> `Vec<Queue>` -> Slint `Model` (`Rc<VecModel<QueueStr>>`).
    -   Populate `App` properties on startup.

## Verification

### Automated Tests
-   UI logic is hard to automated test without a full UI test runner.
-   We'll verify compilation and basic startup.

### Manual Verification
-   Run `cargo run`.
-   Ensure columns verify horizontal scrolling.
-   Ensure tickets appear in correct columns.
