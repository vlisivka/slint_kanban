# CLI Test Suite Design Plan

This document outlines the strategy for creating a comprehensive Bash-based test suite for the Slint Kanban CLI.

## Objective
To ensure the robustness and correctness of the CLI interface through automated, isolated, and comprehensive tests covering both happy and unhappy paths.

## 1. Directory Structure

```
tests/cli/
├── lib/
│   └── test_lib.sh        # Shared library for assertions, logging, and environment management
├── test_add.sh            # Tests for `add` command
├── test_update.sh         # Tests for `update` command
├── test_list.sh           # Tests for `list` command
├── test_configure.sh      # Tests for `configure` command
├── test_move.sh           # Tests for `move` command
├── test_remove.sh         # Tests for `remove` command
├── test_show.sh           # Tests for `show` command
├── test_open.sh           # Tests for `open` command (path verification only)
└── run_all.sh             # Master script to run all tests
```

## 2. Shared Library (`test_lib.sh`)

This library will provide the core functionality for all test scripts.

### Core Functions:
- **Logging**:
  - `log_info(msg)`: Print informational message (e.g., "  [INFO] Running test: $msg").
  - `log_success(msg)`: Print success message (e.g., "  [PASS] $msg").
  - `log_error(msg)`: Print error message to stderr.
  - `panic(msg)`: Print error and exit the script with status 1.

- **Assertions**:
  - `assert_eq(expected, actual, msg)`: Checks if two strings are equal.
  - `assert_contains(haystack, needle, msg)`: Checks if `needle` is present in `haystack`.
  - `assert_not_contains(haystack, needle, msg)`: Checks if `needle` is NOT present in `haystack`.
  - `assert_exit_code(expected, actual, msg)`: Checks if the exit code matches expected value.

- **Environment Management**:
  - `setup_env()`:
    - Creates a temporary directory for `KANBAN_HOME`.
    - Locates the `slint_kanban` binary (builds if necessary or uses pre-built).
    - Sets up necessary environment variables (e.g., `NO_COLOR=1`).
  - `cleanup_env()`:
    - Removes the temporary directory.
  - `run_app(args...)`:
    - Wraps the binary execution, passing `--root $KANBAN_HOME` automatically.
    - Captures stdout, stderr, and exit code.
    - Sets global variables: `LAST_STDOUT`, `LAST_STDERR`, `LAST_EXIT_CODE`.

## 3. Test Strategy

- **Isolation**: Each test file (or even each test case, depending on performance) will use a fresh temporary `KANBAN_HOME` to prevent state pollution.
- **Color Handling**: The application should detect non-TTY environment (pipes) and disable color automatically. `test_lib.sh` will explicitly strip ANSI codes if necessary to ensure plain text verification.
- **Execution**: `run_all.sh` will iterate over all `test_*.sh` scripts and execute them, summarizing the results.

## 4. Detailed Test Cases

### `add` Command
- **Happy Path**:
  - Add ticket with title only.
  - Add ticket with title and description.
  - Add ticket with specific queue.
  - Add ticket with assignee.
- **Unhappy Path**:
  - Missing title argument.
  - Invalid queue name.

### `update` Command
- **Happy Path**:
  - Update title.
  - Update description.
  - Update assignee.
  - Unassign user.
- **Unhappy Path**:
  - Non-existent Ticket ID.

### `move` Command
- **Happy Path**:
  - Move ticket to valid queue.
- **Unhappy Path**:
  - Non-existent Ticket ID.
  - Invalid target queue.
  - Moving to the same queue (should handle gracefully or error).

### `remove` Command
- **Happy Path**:
  - Remove existing ticket (should move to Deleted or be removed).
- **Unhappy Path**:
  - Non-existent Ticket ID.

### `list` Command
- **Happy Path**:
  - List all tickets.
  - Filter by queue name.
  - Filter by user (assigned).
  - Filter by unassigned.
  - Filter by search query.
  - Filter by date range.
- **Unhappy Path**:
  - (Few unhappy paths for list, mostly just empty results).

### `show` Command
- **Happy Path**:
  - Show details of existing ticket.
- **Unhappy Path**:
  - Non-existent Ticket ID.

### `configure` Command
- **Happy Path**:
  - Add user.
  - Set active user.
  - Toggle `show_only_mine`.
- **Unhappy Path**:
  - Invalid subcommands/arguments.

## 5. Implementation Steps

1.  **Create Directory Structure**: Set up `tests/cli/lib`.
2.  **Develop `test_lib.sh`**: Implement the shared functions.
3.  **Implement `test_add.sh`**: Create the first test script as a proof of concept.
4.  **Implement Remaining Tests**: Iteratively add scripts for other commands.
5.  **Create `run_all.sh`**: Tie everything together.
6.  **Verify**: Run the suite and fix any issues in the app or tests.
