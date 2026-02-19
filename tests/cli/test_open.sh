#!/bin/bash
# Tests for the 'open' CLI command.
# Only verifies argument parsing since GUI launch is environment-dependent.
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'open' command..."

# Setup: create a board directory
TEST_PATH="$KANBAN_HOME/TestBoard"
mkdir -p "$TEST_PATH"

# Test 1: Verify argument parsing and initial message
log_info "Scenario: Open specific path"
# Use timeout to prevent blocking if GUI actually launches
timeout 2s "$BIN_PATH" open "$TEST_PATH" > "$KANBAN_HOME/stdout" 2> "$KANBAN_HOME/stderr"

LAST_STDOUT=$(cat "$KANBAN_HOME/stdout")
LAST_STDERR=$(cat "$KANBAN_HOME/stderr")

assert_contains "$LAST_STDOUT" "Opening GUI for path:" "Start message present"
assert_contains "$LAST_STDOUT" "TestBoard" "Path present in message"

cleanup_env
log_info "Open tests passed (argument parsing verified)!"
