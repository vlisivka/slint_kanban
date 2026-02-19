#!/bin/bash
# Tests for the 'remove' CLI command.
# Tickets are moved to the Deleted folder upon removal.
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'remove' command..."

# Setup: Add a ticket
run_app add --title "Delete Me" --queue "1. Incoming"
ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)

if [ ! -L "$KANBAN_HOME/Queue/1. Incoming/$ID" ]; then
    panic "Ticket not created correctly"
fi

# Test 1: Remove ticket
log_info "Scenario: Remove existing ticket"
run_app remove --id "$ID"
assert_exit_code 0 "Exit code 0"

# Verify ticket removed from queue
if [ -L "$KANBAN_HOME/Queue/1. Incoming/$ID" ]; then
    panic "Ticket still in queue"
fi

# Verify ticket moved to Deleted folder
if [ -d "$KANBAN_HOME/Queue/Deleted/$ID" ] || [ -d "$KANBAN_HOME/Deleted/$ID" ]; then
    log_success "Ticket moved to Deleted folder"
fi

# Test 2: Unhappy Path - Invalid ID
log_info "Scenario: Remove invalid ID"
run_app remove --id "INVALID"
assert_exit_code 1 "Exit code 1 (Application Error)"
assert_contains "$LAST_STDERR" "not found" "Error message present"

cleanup_env
log_info "Remove tests passed!"
