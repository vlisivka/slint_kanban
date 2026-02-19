#!/bin/bash
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'show' command..."

# Setup: Add a ticket
run_app add --title "Show Me" --queue "1. Incoming" --description "Detailed Description" --assign-to "Dave"
ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)

# Test 1: Show valid ticket
log_info "Scenario: Show ticket"
run_app show --id "$ID"
assert_exit_code 0 "Exit code 0"

# Verify details in output
assert_contains "$LAST_STDOUT" "Title:       Show Me" "Title shown"
assert_contains "$LAST_STDOUT" "Description:" "Description header shown"
assert_contains "$LAST_STDOUT" "Detailed Description" "Description body shown"
assert_contains "$LAST_STDOUT" "Assigned to: Dave" "Assignee shown"
assert_contains "$LAST_STDOUT" "Status:      1. Incoming" "Status/Queue shown"

# Test 2: Unhappy Path - Invalid ID
log_info "Scenario: Invalid ID"
run_app show --id "INVALID"
assert_exit_code 1 "Exit code 1 (Application Error)"
assert_contains "$LAST_STDERR" "Ticket not found" "Error message"

cleanup_env
log_info "Show tests passed!"
