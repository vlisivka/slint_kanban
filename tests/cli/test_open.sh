#!/bin/bash
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'open' command..."

# Setup: Dummy path
TEST_PATH="$KANBAN_HOME/TestBoard"
mkdir -p "$TEST_PATH"

# Test 1: Open command
log_info "Scenario: Open specific path"
# Since GUI will likely fail in this test environment (headless), or block,
# we need to be careful.
# Slint might panic or return error if no backend.
# However, we can check for "Opening GUI" print before it calls GUI.

# Run with timeout to prevent blocking if it actually works?
#timeout 5s "$BIN_PATH" open "$TEST_PATH" > "$KANBAN_HOME/stdout" 2> "$KANBAN_HOME/stderr"
#Wait, run_app handles exec. We can't use run_app because it waits.
# Slint initialization is fast failure or success.

# Let's try running it. If it fails due to no display, that's fine, as long as it prints the message first.
# "Opening GUI for path: ..." is printed before run_gui.

timeout 2s "$BIN_PATH" open "$TEST_PATH" > "$KANBAN_HOME/stdout" 2> "$KANBAN_HOME/stderr"
RES=$?

LAST_STDOUT=$(cat "$KANBAN_HOME/stdout")
LAST_STDERR=$(cat "$KANBAN_HOME/stderr")

# Check output
assert_contains "$LAST_STDOUT" "Opening GUI for path:" "Start message present"
assert_contains "$LAST_STDOUT" "TestBoard" "Path present in message"

# Exit code?
# If backend fails, likely non-zero.
# If it succeeds (e.g. software renderer), it blocks.
# If it blocks, this test hangs.
# So we MUST use timeout.
# But timeout kills the process.

# Given constrained environment, maybe skip deep verification of GUI launch.
# Just verifying argument parsing is enough.

cleanup_env
log_info "Open tests passed (argument parsing verified)!"
