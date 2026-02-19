#!/bin/bash
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'list' command..."

# Setup: Add some tickets
run_app add --title "Task A" --queue "1. Incoming"
run_app add --title "Task B" --queue "1. Incoming" --assign-to "Alice"
run_app add --title "Task C" --queue "2. ToDo" --assign-to "Bob"
run_app add --title "UniqueKeyword" --queue "3. Doing"
run_app add --title "Task D" --queue "1. Incoming" --description "Findme"

# Test 1: List all
log_info "Scenario: List all tickets"
run_app list
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "Task A" "Task A listed"
assert_contains "$LAST_STDOUT" "Task B" "Task B listed"
assert_contains "$LAST_STDOUT" "Task C" "Task C listed"
assert_contains "$LAST_STDOUT" "UniqueKeyword" "UniqueKeyword listed"

# Test 2: Filter by assigned user
log_info "Scenario: Filter by assigned user (Alice)"
run_app list --assigned-to-user "Alice"
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "Task B" "Task B found"
assert_not_contains "$LAST_STDOUT" "Task A" "Task A (unassigned) hidden"
assert_not_contains "$LAST_STDOUT" "Task C" "Task C (Bob) hidden"

# Test 3: Filter by unassigned
log_info "Scenario: Filter by unassigned"
run_app list --unassigned
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "Task A" "Task A (unassigned) found"
assert_not_contains "$LAST_STDOUT" "Task B" "Task B (assigned) hidden"

# Test 4: Search
log_info "Scenario: Search by title keyword"
run_app list --search "UniqueKeyword"
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "UniqueKeyword" "Keyword match found"
assert_not_contains "$LAST_STDOUT" "Task A" "Other task hidden"

log_info "Scenario: Search by description keyword"
run_app list --search "Findme"
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "Task D" "Description match found"

# Test 5: ID filter (if valid)
# Need to get ID first
ID_A=$(ls -t "$KANBAN_HOME/Tickets" | grep -v "README" | head -n 1) # Wait, ls lists folders.
# Actually, tickets are random IDs.
# Let's re-add a ticket and catch ID? No.
# Just grep stdout of list all.
run_app list
ID_B=$(echo "$LAST_STDOUT" | grep "Task B" | awk '{print $1}' | tr -d '[]')
# Output format: [id] Title ...

log_info "Scenario: Filter by ID"
if [ -n "$ID_B" ]; then
    run_app list --id "$ID_B"
    assert_exit_code 0 "Exit code 0"
    assert_contains "$LAST_STDOUT" "Task B" "Task B found by ID"
    assert_not_contains "$LAST_STDOUT" "Task A" "Task A hidden"
else
    log_error "Could not parse ID for Task B"
fi

cleanup_env
log_info "List tests passed!"
