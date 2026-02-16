# Plan: Ticket view

Implement a beautiful, full-window read-only ticket view. This is a foundational step before adding Markdown support.

## Proposed Changes

### [Rust Backend]

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Update how `TicketStr` is populated.
- Pass the **full** description in the `description` field.

### [UI / Slint]

#### [MODIFY] [app.slint](file:///home/vlisivka/workspace/slint_kanban/ui/app.slint)

##### `TicketStr` struct
- Ensure `description` is intended for the full text.
- Ensure that only first line displayed in ticket preview in queue, without overflow (overflow to elipsis).

##### `TicketDetail` component (to be renamed or repurposed as `TicketView`)
- Change the layout of ticket window to be **full-window**:
  - Instead of a fixed-size `Rectangle` in the middle, make it fill the parent width/height.
  - Use a `VerticalBox` for padding and spacing.
  - Render title as header, not as separate field.
- Add a "Close" button.

##### `App` component
- Add a property `is_viewing_ticket` (bool).
- Add a property `active_ticket` (TicketStr) to hold the ticket being viewed.
- Update `TicketCard` clicked handler to set `is_viewing_ticket = true`.

## Verification Plan

### Manual Verification
1. **Launch App**: `cargo run`.
2. **Open Ticket**: Click any ticket card.
3. **Check Visibility**: Confirm text is visible regardless of OS theme (no hardcoded colors).
4. **Check Size**: Confirm the view covers the entire board area.
5. **Close View**: Click "Close" and confirm return to the board.
