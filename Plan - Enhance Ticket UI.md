# Plan - Enhance Ticket UI

## Meta
**Goal**: Update `TicketCard` to use fixed vertical space and display Title, Date, and truncated Body.

## Changes

### 1. `ui/app.slint`
-   **Struct `TicketStr`**: Add `created_at: string`.
-   **Component `TicketCard`**:
    -   Set fixed `height` (e.g., `120px`) or `min-height`/`max-height`.
    -   Layout:
        -   Top: Title.
        -   Middle: Description (first few lines, `overflow: elide`).
        -   Bottom: Row with `created_at` (left) and `id` (right).

### 2. `src/main.rs`
-   **Helper `reload_board`**:
    -   Map `ticket.created_at` to `TicketStr.created_at`.

## Verification

### Manual Verification
-   Run `cargo run`.
-   Check if cards look uniform.
-   Check if long descriptions are truncated.
-   Check if date is visible.
