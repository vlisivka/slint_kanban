#!/bin/bash

# run-all-tests.sh
# Purpose: Run all automated tests for the Slint Kanban project, including 
# unit tests, CLI integration tests, and GUI tests.

set -ueE -o pipefail # Exit immediately if a command exits with a non-zero status.

echo "--- 🛠️  Starting Comprehensive Test Suite ---"

# 1. Format code
echo "--- 📝 Formatting code---"
cargo fmt

# 2. Run Cargo Tests (Unit, Main tests, GUI, Performance, Fuzzy)
# Note: --test-threads=1 is critical because Slint testing backend 
# initializes a global platform state that cannot be shared across threads.
echo "--- 🧪 Running Cargo tests (Unit, Main tests, GUI/Perf/Fuzzy) ---"
cargo test -- --test-threads=1 --nocapture

echo "--- ✅ All tests passed successfully! ---"
