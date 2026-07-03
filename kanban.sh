#!/bin/bash
PROJECT_ROOT="$(readlink -f $(dirname "$0"))"

echo "Project root: \"$PROJECT_ROOT\", kanban board root: \"$PROJECT_ROOT/kanban\"."

RUST_BACKTRACE=1 cargo run --bin slint_kanban -- --root="$PROJECT_ROOT/kanban" "$@"
