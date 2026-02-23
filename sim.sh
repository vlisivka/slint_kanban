#!/bin/bash
set -ueE -o pipefail

cargo run --bin simulate_work /tmp/kanban_sim

cargo run --bin slint_kanban -- --root /tmp/kanban_sim stats

cargo run --bin slint_kanban -- --root /tmp/kanban_sim
