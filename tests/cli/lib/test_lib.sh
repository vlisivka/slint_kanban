#!/bin/bash

# Configuration: derive binary name from Cargo.toml at project root
PROJECT_ROOT="$(git rev-parse --show-toplevel)"
BIN_NAME="$(grep -m1 '^name' "$PROJECT_ROOT/Cargo.toml" | sed 's/.*= *"\(.*\)"/\1/')"
BIN_PATH="$PROJECT_ROOT/target/debug/$BIN_NAME"

# Globals
KANBAN_HOME=""
LAST_STDOUT=""
LAST_STDERR=""
LAST_EXIT_CODE=0

# ANSI colors for test runner output (app output uses NO_COLOR)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

panic() {
    log_error "$1"
    cleanup_env
    exit 1
}

setup_env() {
    # Create isolated temp directory for each test run
    KANBAN_HOME=$(mktemp -d -t slint_kanban_test.XXXXXX)
    export KANBAN_HOME
    export NO_COLOR=1 # Standard way to request no color
    
    # Ensure binary exists
    if [ ! -f "$BIN_PATH" ]; then
        log_info "Building binary..."
        cd "$(git rev-parse --show-toplevel)" && cargo build --quiet || panic "Build failed"
    fi
}

cleanup_env() {
    if [ -d "$KANBAN_HOME" ]; then
        rm -rf "$KANBAN_HOME"
    fi
}

run_app() {
    local out_file="${KANBAN_HOME}/stdout"
    local err_file="${KANBAN_HOME}/stderr"
    
    # NO_COLOR is already exported; also strip ANSI codes defensively
    "$BIN_PATH" "$@" > "$out_file" 2> "$err_file"
    LAST_EXIT_CODE=$?
    # Strip any ANSI escape codes so assertions always see plain text
    sed -i 's/\x1b\[[0-9;]*m//g' "$out_file" "$err_file"
    
    LAST_STDOUT=$(cat "$out_file")
    LAST_STDERR=$(cat "$err_file")
    
    # Cleanup temp files
    rm "$out_file" "$err_file"
}

assert_eq() {
    local expected="$1"
    local actual="$2"
    local msg="$3"
    
    if [ "$expected" != "$actual" ]; then
        log_error "Assertion failed: $msg"
        log_error "Expected: '$expected'"
        log_error "Actual:   '$actual'"
        panic "Test failed"
    else
        log_success "$msg"
    fi
}

assert_contains() {
    local haystack="$1"
    local needle="$2"
    local msg="$3"
    
    if [[ "$haystack" != *"$needle"* ]]; then
        log_error "Assertion failed: $msg"
        log_error "Expected to contain: '$needle'"
        log_error "Actual: '$haystack'"
        panic "Test failed"
    else
        log_success "$msg"
    fi
}

assert_not_contains() {
    local haystack="$1"
    local needle="$2"
    local msg="$3"
    
    if [[ "$haystack" == *"$needle"* ]]; then
        log_error "Assertion failed: $msg"
        log_error "Expected NOT to contain: '$needle'"
        log_error "Actual: '$haystack'"
        panic "Test failed"
    else
        log_success "$msg"
    fi
}

assert_exit_code() {
    local expected="$1"
    local msg="$2"
    
    if [ "$LAST_EXIT_CODE" -ne "$expected" ]; then
        log_error "Assertion failed: $msg"
        log_error "Expected exit code: $expected"
        log_error "Actual exit code:   $LAST_EXIT_CODE"
        log_error "Stderr: $LAST_STDERR"
        panic "Test failed"
    else
        log_success "$msg"
    fi
}
