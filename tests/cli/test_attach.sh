#!/bin/bash
# test_attach.sh

source "$(dirname "$0")/lib/test_lib.sh"

setup_env

log_info "Running Attach tests..."

# Initialize board and add a ticket
run_app add -t "Test attach" -q "1. Incoming"
assert_exit_code 0 "Failed to add ticket"

# List tickets to extract ID
run_app list
assert_exit_code 0 "Failed to list tickets"
TICKET_ID=$(echo "$LAST_STDOUT" | grep -oP '\[\K[a-zA-Z0-9]+(?=\])' | head -n 1)

if [ -z "$TICKET_ID" ]; then
    panic "Could not find a valid Ticket ID in output: $LAST_STDOUT"
fi

# Create test file to attach
TEST_FILE="$KANBAN_HOME/test_file.txt"
echo "test content" > "$TEST_FILE"

# Attach file
run_app attach -i "$TICKET_ID" -f "$TEST_FILE"
assert_exit_code 0 "Failed to attach file"

# Extract returned markdown link
MARKDOWN_LINK=$(echo "$LAST_STDOUT" | tail -n 1)
assert_contains "$MARKDOWN_LINK" "[test_file.txt](attachment/test_file.txt)" "Output should contain the markdown link"

# Verify file was copied
ATTACHMENT_PATH="$KANBAN_HOME/Tickets/$TICKET_ID/attachment/test_file.txt"
if [ ! -f "$ATTACHMENT_PATH" ]; then
    panic "Attached file was not found at expected path: $ATTACHMENT_PATH"
fi

# Test filename collision
run_app attach -i "$TICKET_ID" -f "$TEST_FILE"
assert_exit_code 0 "Failed to attach file second time"

# Extract returned markdown link
MARKDOWN_LINK=$(echo "$LAST_STDOUT" | tail -n 1)
assert_contains "$MARKDOWN_LINK" "[test_file (1).txt](attachment/test_file (1).txt)" "Output should contain the collision markdown link"

# Verify collision file was copied
ATTACHMENT_PATH="$KANBAN_HOME/Tickets/$TICKET_ID/attachment/test_file (1).txt"
if [ ! -f "$ATTACHMENT_PATH" ]; then
    panic "Attached collision file was not found at expected path: $ATTACHMENT_PATH"
fi

cleanup_env
log_success "Attach tests passed!"
