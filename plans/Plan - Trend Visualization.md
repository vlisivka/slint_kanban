# Plan - Trend Visualization

## Meta
The goal is to implement trend visualization in the analytics dashboard to show project progress over time. This involves parsing the activity logs to reconstruct the board state at different points in time and displaying this data as a chart in the GUI.

## Proposed Changes

### 1. Data Model (`src/model/stats.rs`)
- Add a new struct `TrendData` to represent metrics at a specific point in time.
  ```rust
  pub struct TrendPoint {
      pub timestamp: String,
      pub total_tickets: usize,
      pub done_tickets: usize,
      pub total_points: u32,
      pub done_points: u32,
  }
  ```
- Implement a function `get_trend_data(all_logs: &[LogEntry], intervals: usize) -> Vec<TrendPoint>`.
  - This function will:
    1. Determine the time range (from first log to now).
    2. Divide it into `intervals` (e.g., 10-20 points).
    3. For each interval end, simulate the board state by replaying actions from the beginning.
    4. Store the snapshot of ticket/point counts.

### 2. UI Definitions (`ui/common.slint`)
- Export `TrendPoint` struct to Slint.
  ```slint
  export struct TrendPointStr {
      timestamp: string,
      total_tickets: int,
      done_tickets: int,
      total_points: int,
      done_points: int,
  }
  ```
- Add `trend: [TrendPointStr]` to `BoardSummaryStr`.

### 3. Chart Component (`ui/stats_view.slint`)
- Implement a simple `TrendChart` component using `Rectangle` and `for` loops.
- It will show a "stacked" or "side-by-side" bar chart representing total vs done tickets/points.
- Given Slint's limitations, a bar chart is easier to implement than a smooth line chart.

### 4. Integration (`src/main.rs`)
- Update `into_slint_summary` to include trend data.
- Update `handle_show_board_info` or where stats are requested to calculate the trend.

## Tests
- **Unit Tests (`src/model/tests/stats_tests.rs`)**:
  - Test `get_trend_data` with a controlled set of log entries.
  - Verify that counts are correct at each interval.
- **GUI Tests (`src/gui_tests.rs`)**:
  - Verify that the stats view renders without crashing when trend data is present.

## Simulation Script (`scripts/simulate_work.rs`)
To realistically verify trend visualization, we will implement a simulation utility that generates a board with historical data.

### Features:
1. **Board Setup**: Initializes a new Kanban board with default queues and multiple sprints in the past.
2. **History Generation**:
    - Replays actions for the last 30-60 days.
    - Randomly adds tickets with points (0-10).
    - Simulates "work" by moving tickets through queues (To Do -> Doing -> Review -> Done).
    - **Point-Based Velocity**: Heavier tickets (more points) stay in "active" queues longer than smaller ones.
    - **Past Timestamps**: Manually writes log entries and README.md fields with adjusted dates to simulate past activity.
3. **Execution**:
    - Can be run via `cargo run --bin simulate_work -- /tmp/simulated_board`.
    - Outputs summary statistics at the end.
    - Offers to launch the GUI to visualize the generated trends.

## Manual Verification
1. Run the simulator: `cargo run --bin simulate_work -- ./test_board`.
2. Open the application: `cargo run -- ./test_board`.
3. Open **Board Info** -> **Statistics**.
4. Observe the trend chart (it should show a realistic growth of "Total Points" and "Done Points" over time).
