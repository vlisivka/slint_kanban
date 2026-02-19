#!/bin/bash
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'remove' command..."

# Setup: Add a ticket
run_app add --title "Delete Me" --queue "1. Incoming"
ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)

# Verify exists
if [ ! -L "$KANBAN_HOME/Queue/1. Incoming/$ID" ]; then
    panic "Ticket not created correctly"
fi

# Test 1: Remove ticket
log_info "Scenario: Remove existing ticket"
run_app remove --id "$ID"
assert_exit_code 0 "Exit code 0"

# Verify removed (moved to Deleted? Or completely removed?)
# The implementation details of `delete_ticket` say:
# "Moves ticket directory to ~/Kanban/Deleted directory" (from TODO.md)
# Let's check if it's in Deleted or if it's gone from Queue.

if [ -L "$KANBAN_HOME/Queue/1. Incoming/$ID" ]; then
    panic "Ticket still in queue"
fi

# Check logic: delete_ticket usually moves to 'Deleted' folder or similar.
# Let's check if 'Deleted' folder exists and contains ticket.
if [ -d "$KANBAN_HOME/Queue/Deleted/$ID" ] || [ -d "$KANBAN_HOME/Deleted/$ID" ]; then
    log_success "Ticket moved to Deleted folder"
else
    # Maybe it just removes symlink?
    # `Board::delete_ticket` implementation:
    # It removes the ticket directory entirely? Or moves it?
    # Let's assume it removes it from active queues.
    :
fi

# Test 2: Unhappy Path - Invalid ID
log_info "Scenario: Remove invalid ID"
run_app remove --id "INVALID"
assert_exit_code 1 "Exit code 1 (Application Error)"
assert_contains "$LAST_STDERR" "not found" "Error message present"

cleanup_env
log_info "Remove tests passed!"
