# Plan - Performance Optimization

## Objective
Significant performance improvement for boards with 1000+ tickets.
Reduce I/O operations by 50-80% and optimize algorithmic complexity from O(T*L) to O(L).

---

## Phase 1: Quick Wins (Structural & Algorithmic O(1))
1.  **Ticket Indexing**:
    *   Modify `Board` struct to include a `ticket_index: HashMap<String, (usize, usize)>` (mapping ticket ID to queue/ticket indices).
    *   Update `Board::load` to populate this index.
    *   Replace linear search in `find_ticket_by_id` with O(1) hash map lookup.
    *   **Impact**: Fixes sluggish UI synchronization when tickets have many cross-references.

2.  **Log Loading Optimization**:
    *   Refactor `get_board_summary` in `stats.rs` to load all logs into a local variable once.
    *   Pass the pre-loaded logs to Lead Time, Cycle Time, Sprint Rate, and Trend calculation functions.
    *   **Impact**: Reduces log file I/O by 50% during statistics display.

3.  **Search Debounce**:
    *   Add a small delay (e.g., 300ms) before triggering a full board reload when the user types in the search field.
    *   **Impact**: Prevents UI freezes during rapid typing.

---

## Phase 2: I/O Reduction (Lazy Loading)
4.  **Lazy Comment Loading**:
    *   Modify `Ticket::load` to NOT load comments from disk.
    *   Create `Ticket::load_comments` to fetch comments only when a ticket is opened for viewing.
    *   **Impact**: Dramatically reduces the number of files read during initial board load (1000 tickets * 3 comments = 3000 fewer files).

5.  **Header-Only Ticket Loading**:
    *   Split `Ticket::load` into `load_header` and `load_full`.
    *   `load_header` will only parse the YAML metadata and the first line of the body (for the snippet).
    *   `load_full` will be called only when the user clicks on a card.
    *   **Impact**: Reduces CPU and Memory usage during board sync.

6.  **Attachment Count Caching**:
    *   Avoid re-scanning the `attachment/` directory for every ticket during every UI sync.
    *   Optionally store the count in ticket metadata or cache it in memory.

---

## Phase 3: Scaling Improvements (Advanced)
7.  **O(L) Metric Calculation**:
    *   Refactor `calculate_lead_time` and `calculate_cycle_time` to avoid nested loops (O(T*L)).
    *   Implement a single-pass processing function that iterates over logs once and updates state for all tickets simultaneously.
    *   **Impact**: Crucial for large logs (10,000+ entries).

8.  **Stable Queue Caching**:
    *   Implement a check for directory `mtime` in `Done` and `Archive` queues.
    *   If the directory has not changed, reuse cached ticket data instead of re-reading every file.
    *   **Impact**: 70-90% faster reload for mature boards.

9.  **Incremental UI Updates**:
    *   In `sync_board_to_ui`, only update rows in Slint's `VecModel` that have actually changed.
    *   **Impact**: Smoother UI transitions and less work for the Slint renderer.

---

## Verification Plan
1.  **Large Board Simulation**: Generate a test board with 2000 tickets and 10,000 log entries.
2.  **Timing**: Measure "Board Load" and "Stats Generation" time before and after optimizations.
3.  **Profiling**: Use `cargo flamegraph` to ensure P2/P3/P4 bottlenecks are eliminated.
4.  **Tests**: Ensure all existing tests pass, verifying that optimizations didn't break data integrity.
