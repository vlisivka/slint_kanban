#!/bin/bash
# Tests for the 'add' CLI command.
# Usage: add --title <TITLE> --queue <QUEUE> [--description <DESC>] [--assign-to <USER>]
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'add' command..."

# Test 1: Add with title and queue (mandatory)
log_info "Scenario: Add ticket with title and queue"
run_app add --title "Buy Milk" --queue "1. Incoming"
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "Adding ticket:" "Output confirms creation"
assert_contains "$LAST_STDOUT" "Buy Milk" "Output contains title"

# Verify file exists
TICKET_ID=$(ls "$KANBAN_HOME/Tickets" | head -n 1)
if [ -z "$TICKET_ID" ]; then
    panic "Ticket file not created"
fi
log_success "Ticket file created at $KANBAN_HOME/Tickets/$TICKET_ID"

# Test 2: Add with description
log_info "Scenario: Add ticket with description"
run_app add --title "Walk Dog" --description "In the park" --queue "1. Incoming"
assert_exit_code 0 "Exit code 0"
# Verify description in file
NEW_TICKET_ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)
grep -q "In the park" "$KANBAN_HOME/Tickets/$NEW_TICKET_ID/README.md"
if [ $? -ne 0 ]; then
    panic "Description not found in ticket file"
fi
log_success "Description persisted correctly"

# Test 3: Add with assignee
log_info "Scenario: Add ticket with assignee"
run_app add --title "Task 3" --queue "1. Incoming" --assign-to "Alice"
assert_exit_code 0 "Exit code 0"
NEW_TICKET_ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)
grep -q "assigned_to: \"Alice\"" "$KANBAN_HOME/Tickets/$NEW_TICKET_ID/README.md"
if [ $? -ne 0 ]; then
    panic "Assignee not found in ticket file"
fi
log_success "Assignee persisted correctly"

# Test 4: Unhappy Path - Missing Queue
log_info "Scenario: Missing queue"
run_app add --title "Missing Queue"
assert_exit_code 2 "Exit code 2 (Clap error)"
assert_contains "$LAST_STDERR" "required arguments" "Error message about arguments"
assert_contains "$LAST_STDERR" "--queue" "Error mentions missing queue"

# Test 5: Unhappy Path - Missing Title
log_info "Scenario: Missing title"
run_app add --queue "1. Incoming"
assert_exit_code 2 "Exit code 2 (Clap error)"
assert_contains "$LAST_STDERR" "required arguments" "Error message about arguments"
assert_contains "$LAST_STDERR" "--title" "Error mentions missing title"

cleanup_env
log_info "Add tests passed!"
