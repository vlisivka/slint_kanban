#!/bin/bash
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'configure' command..."
# Config file path: $KANBAN_HOME/config.toml

# Test 1: Add user
log_info "Scenario: Add user"
run_app configure --add-user "Dave"
assert_exit_code 0 "Exit code 0"
# Verify config file
grep -q "\"Dave\"" "$KANBAN_HOME/config.toml"
if [ $? -ne 0 ]; then panic "User not added to config"; fi
log_success "User added"

# Test 2: Set active user
log_info "Scenario: Set active user"
run_app configure --active-user "Dave"
assert_exit_code 0 "Exit code 0"
grep -q "active_user = \"Dave\"" "$KANBAN_HOME/config.toml"
if [ $? -ne 0 ]; then panic "Active user not set"; fi
log_success "Active user set"

# Test 3: Set show_only_mine
log_info "Scenario: Set show_only_mine"
run_app configure --show-only-mine true
assert_exit_code 0 "Exit code 0"
grep -q "show_only_mine = true" "$KANBAN_HOME/config.toml"
if [ $? -ne 0 ]; then panic "show_only_mine not set to true"; fi
log_success "show_only_mine set to true"

# Unhappy path? Configure usually accepts anything that matches arg types.
# Maybe invalid boolean?
log_info "Scenario: Invalid boolean"
run_app configure --show-only-mine "maybe"
assert_exit_code 2 "Exit code 2 (Clap error)" # Should fail argument parsing
assert_contains "$LAST_STDERR" "invalid value" "Error message"

cleanup_env
log_info "Configure tests passed!"
