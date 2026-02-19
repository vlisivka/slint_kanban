#!/bin/bash
# Tests for the 'move' CLI command.
# Covers: valid move, invalid queue, invalid ID.
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'move' command..."

# Setup: Add a ticket
run_app add --title "Move Me" --queue "1. Incoming"
ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)

# Verify source queue
if [ ! -L "$KANBAN_HOME/Queue/1. Incoming/$ID" ]; then
    panic "Ticket not in source queue"
fi

# Test 1: Move to valid queue
log_info "Scenario: Move to valid queue"
run_app move --id "$ID" --queue "2. ToDo"
assert_exit_code 0 "Exit code 0"

# Verify move
if [ -L "$KANBAN_HOME/Queue/1. Incoming/$ID" ]; then
    panic "Ticket still in source queue"
fi
if [ ! -L "$KANBAN_HOME/Queue/2. ToDo/$ID" ]; then
    panic "Ticket not moved to target queue"
fi
log_success "Ticket moved successfully"

# Test 2: Unhappy Path - Invalid Target Queue
log_info "Scenario: Invalid target queue"
run_app move --id "$ID" --queue "invalid_queue"
assert_exit_code 1 "Exit code 1 (Application Error)"
assert_contains "$LAST_STDERR" "queue not found" "Error message present"

# Test 3: Unhappy Path - Invalid ID
run_app move --id "INVALID" --queue "2. ToDo"
assert_exit_code 1 "Exit code 1 (Application Error)"
assert_contains "$LAST_STDERR" "Ticket not found" "Error message present"

cleanup_env
log_info "Move tests passed!"
