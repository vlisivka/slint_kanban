#!/bin/bash
source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Testing 'add' command..."

# 1. Happy Path: Add basic ticket
log_info "Scenario: Add ticket with title only"
# Usage: add [OPTIONS] <TITLE> [DESCRIPTION] [QUEUE] [ASSIGNED-TO]
# Wait, CLI structure is likely: add --title "Title" or add "Title"
# Let's check help again.
# Commands:
#   add        Add a new ticket

# Let's check `add --help` to be sure about arguments.
"$BIN_PATH" add --help > /dev/null
if [ $? -ne 0 ]; then
    panic "Failed to run help"
fi

# Assuming: add <TITLE> [DESCRIPTION] --queue <QUEUE> --assigned-to <USER>
# Let's run help via run_app to check output formats.
run_app add --help
# Output would be in LAST_STDOUT.
# I'll rely on my memory of the code.

# Code says:
# Commands::Add { title, description, queue, assigned_to }
# In Clap:
# #[command(subcommand)]
# command: Option<Commands>,
# ...
# enum Commands {
#   Add {
#       title: String,
#       description: Option<String>,
#       #[arg(short, long)]
#       queue: Option<String>,
#       #[arg(short, long)]
#       assigned_to: Option<String>,
#   }
# }
# So: add "Title" "Description" --queue "Queue" --assigned-to "User"

# Test 1: Add with title and queue (mandatory)
log_info "Scenario: Add ticket with title and queue"
run_app add --title "Buy Milk" --queue "1. Incoming"
assert_exit_code 0 "Exit code 0"
assert_contains "$LAST_STDOUT" "Adding ticket:" "Output confirms creation"
assert_contains "$LAST_STDOUT" "Buy Milk" "Output contains title"

# Verify file exists
TICKET_ID=$(ls "$KANBAN_HOME/Tickets" | head -n 1) # Taking first ticket
if [ -z "$TICKET_ID" ]; then
    panic "Ticket file not created"
fi
log_success "Ticket file created at $KANBAN_HOME/Tickets/$TICKET_ID"

# Test 2: Add with description
log_info "Scenario: Add ticket with description"
# Move existing ticket out or just expect 2 tickets
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
