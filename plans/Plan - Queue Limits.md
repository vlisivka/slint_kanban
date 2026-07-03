# Plan: Queue Limits Implementation

## Goal
Implement configurable queue limits with visual indicators and enforcement to prevent queues from becoming overloaded.

## Requirements (from SPECIFICATION.md)
- To Do queue: Maximum 21 items (configurable)
- Doing queue: Maximum 5 items (configurable)
- Visual warnings when approaching limits
- Prevent adding tickets when limit reached

## Implementation Steps

### 1. Configuration Support
- Add `toml` crate dependency to `Cargo.toml`
- Create `Config` struct in `model.rs` to hold queue limits
- Implement `Config::load()` to read from `~/Kanban/config.toml`
- Implement `Config::default()` with specification defaults (To Do=21, Doing=5)
- Create default config file if it doesn't exist

### 2. Data Model Updates
- Add `config: Config` field to `Board` struct
- Add `limit: Option<usize>` field to `Queue` struct
- Load configuration in `Board::load()`
- Populate queue limits from configuration

### 3. Limit Enforcement Logic
- Update `Board::create_ticket()`:
  - Check if target queue has reached limit
  - Return error if limit exceeded
- Update `Board::move_ticket()`:
  - Check if target queue has reached limit
  - Return error if limit exceeded

### 4. UI Updates (app.slint)
- Update `QueueStr` struct:
  - Add `limit: int` field (-1 for no limit)
  - Add `ticket_count: int` field
- Update `KanbanColumn` component:
  - Display ticket count and limit in header (e.g., "5/21")
  - Add visual indicators:
    - Normal: < 80% of limit
    - Warning (yellow/orange): >= 80% of limit
    - Error (red): at limit
- Create warning dialog component for limit violations

### 5. Main Application Updates (main.rs)
- Update `reload_board()`:
  - Calculate ticket counts per queue
  - Pass limit and count to Slint models
- Update callbacks:
  - Handle limit errors from `create_ticket` and `move_ticket`
  - Display warning messages to user

### 6. Testing
- Add unit tests for configuration loading
- Add unit tests for limit enforcement on creation
- Add unit tests for limit enforcement on moves
- Manual testing of UI indicators and warnings

## Configuration File Format

```toml
# ~/Kanban/config.toml
[queue_limits]
"2.ToDo" = 21
"3.Doing" = 5
```

## Visual Design
- Counter badge in queue header: `[5/21]` or `[5/∞]`
- Color coding:
  - Normal: default text color
  - Warning (80%+): orange/yellow background
  - Error (100%): red background
