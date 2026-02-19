#!/bin/bash
# Tests for the 'update' CLI command.
# Covers: title, description, assign/unassign user, invalid ID.
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'update' command..."

# Setup: Add a ticket
run_app add --title "Original Title" --queue "1. Incoming" --description "Original Desc"
ID=$(ls -t "$KANBAN_HOME/Tickets" | head -n 1)

# Test 1: Update Title
log_info "Scenario: Update title"
run_app update --id "$ID" --title "New Title"
assert_exit_code 0 "Exit code 0"
# Verify file
grep -q "New Title" "$KANBAN_HOME/Tickets/$ID/README.md"
if [ $? -ne 0 ]; then panic "Title not updated"; fi
log_success "Title updated"

# Test 2: Update Description
log_info "Scenario: Update description"
run_app update --id "$ID" --description "New Desc"
assert_exit_code 0 "Exit code 0"
grep -q "New Desc" "$KANBAN_HOME/Tickets/$ID/README.md"
if [ $? -ne 0 ]; then panic "Description not updated"; fi
grep -q "Original Desc" "$KANBAN_HOME/Tickets/$ID/README.md"
if [ $? -eq 0 ]; then panic "Old description still present (should be replaced)"; fi
log_success "Description updated"

# Test 3: Assign User
log_info "Scenario: Assign user"
run_app update --id "$ID" --assign-to "Charlie"
assert_exit_code 0 "Exit code 0"
grep -q "assigned_to: \"Charlie\"" "$KANBAN_HOME/Tickets/$ID/README.md"
if [ $? -ne 0 ]; then panic "User not assigned"; fi
log_success "User assigned"

# Test 4: Unassign User
log_info "Scenario: Unassign user"
run_app update --id "$ID" --unassign
assert_exit_code 0 "Exit code 0"
grep -q 'assigned_to: ""' "$KANBAN_HOME/Tickets/$ID/README.md"
if [ $? -ne 0 ]; then panic "User not unassigned"; fi
log_success "User unassigned"

# Test 5: Unhappy Path - Invalid ID
log_info "Scenario: Invalid ID"
run_app update --id "INVALID" --title "Fail"
assert_exit_code 1 "Exit code 1 (Application Error)"
assert_contains "$LAST_STDERR" "Ticket not found" "Error message present"

cleanup_env
log_info "Update tests passed!"
