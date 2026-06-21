#!/bin/bash
PROJECT_ROOT="$(readlink -f $(dirname "$0"))"

echo "Project root: $PROJECT_ROOT"

slint_kanban --root="$PROJECT_ROOT/kanban" "$@"
