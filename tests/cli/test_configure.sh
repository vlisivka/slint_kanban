#!/bin/bash
# Tests for the 'configure' CLI command.
# Covers: add user, set active user, toggle show_only_mine, invalid inputs.
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'configure' command..."

# Test 1: Add user (Kanban Config)
log_info "Scenario: Add user"
run_app configure --add-user "Dave"
assert_exit_code 0 "Exit code 0"
# Verify config file
grep -q "\"Dave\"" "$KANBAN_HOME/config.toml"
if [ $? -ne 0 ]; then panic "User not added to config.toml"; fi
log_success "User added"

# Test 2: Set active user (User Config)
log_info "Scenario: Set active user"
run_app configure --active-user "Dave"
assert_exit_code 0 "Exit code 0"
USER_CONFIG="${XDG_CONFIG_HOME}/slint-kanban/user.toml"
grep -q "active_user = \"Dave\"" "$USER_CONFIG"
if [ $? -ne 0 ]; then panic "Active user not set in user.toml (checked $USER_CONFIG)"; fi
log_success "Active user set"

# Test 3: Set show_only_mine (User Config)
log_info "Scenario: Set show_only_mine"
run_app configure --show-only-mine true
assert_exit_code 0 "Exit code 0"
grep -q "show_only_mine = true" "$USER_CONFIG"
if [ $? -ne 0 ]; then panic "show_only_mine not set to true in user.toml"; fi
log_success "show_only_mine set to true"

# Test 4: Unhappy Path - Invalid boolean value
log_info "Scenario: Invalid boolean"
run_app configure --show-only-mine "maybe"
assert_exit_code 2 "Exit code 2 (Clap error)"
assert_contains "$LAST_STDERR" "invalid value" "Error message"

cleanup_env
log_info "Configure tests passed!"
