#!/bin/bash
# run_all.sh - Master CLI test runner

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(git rev-parse --show-toplevel)"
BIN_NAME="$(grep -m1 '^name' "$PROJECT_ROOT/Cargo.toml" | sed 's/.*= *"\(.*\)"/\1/')"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

PASS_COUNT=0
FAIL_COUNT=0
FAILED_TESTS=()

run_suite() {
    local script="$1"
    local name="$(basename "$script" .sh)"
    echo ""
    echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}Running: $name${NC}"
    echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    bash "$script"
    local exit_code=$?

    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}✔ $name PASSED${NC}"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        echo -e "${RED}✘ $name FAILED (exit code: $exit_code)${NC}"
        FAIL_COUNT=$((FAIL_COUNT + 1))
        FAILED_TESTS+=("$name")
    fi
}

# Ensure binary is up to date
echo -e "${YELLOW}Building $BIN_NAME...${NC}"
cd "$PROJECT_ROOT" && cargo build --quiet
if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed! Aborting tests.${NC}"
    exit 1
fi
echo -e "${GREEN}Build OK${NC}"

# Run all test scripts in order, except test_open.sh (because it opens GUI window)
for test_script in \
    "$SCRIPT_DIR/test_configure.sh" \
    "$SCRIPT_DIR/test_add.sh" \
    "$SCRIPT_DIR/test_list.sh" \
    "$SCRIPT_DIR/test_show.sh" \
    "$SCRIPT_DIR/test_update.sh" \
    "$SCRIPT_DIR/test_move.sh" \
    "$SCRIPT_DIR/test_remove.sh" \
    "$SCRIPT_DIR/test_attach.sh"
do
    if [ -f "$test_script" ]; then
        run_suite "$test_script"
    else
        echo -e "${YELLOW}Warning: $test_script not found, skipping.${NC}"
    fi
done

# Summary
echo ""
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BOLD}Test Summary${NC}"
echo -e "${BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Passed: $PASS_COUNT${NC}"
echo -e "${RED}Failed: $FAIL_COUNT${NC}"

if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Failed suites:${NC}"
    for t in "${FAILED_TESTS[@]}"; do
        echo -e "  ${RED}• $t${NC}"
    done
    exit 1
else
    echo ""
    echo -e "${GREEN}${BOLD}All tests passed! ✔${NC}"
    exit 0
fi
