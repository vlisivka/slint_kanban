# Plan - Tests Refactoring

Move existing tests from `src/main.rs` and `src/model.rs` to the standard Rust `tests/` directory to reduce file sizes and improve organization.

## Proposed Changes

### Integration Tests

#### [NEW] [model_tests.rs](file:///home/vlisivka/workspace/slint_kanban/tests/model_tests.rs)
- Move all tests from `src/model.rs` to this file.
- Use `slint_kanban::model::{Board, Config, Ticket, Queue}` (requires making the crate accessible).

#### [NEW] [gui_tests.rs](file:///home/vlisivka/workspace/slint_kanban/tests/gui_tests.rs)
- Move `test_ui` from `src/main.rs` to this file.
- Requires `slint_kanban::App`.

#### [NEW] [cli_tests.rs](file:///home/vlisivka/workspace/slint_kanban/tests/cli_tests.rs)
- Move `test_cli_*` from `src/main.rs` to this file.
- Requires `slint_kanban::{run_main, CliArgs, Commands}`.

### Build Configuration

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Remove `#[cfg(test)] mod tests` block.
- Ensure necessary functions and types are public or `pub(crate)` and accessible to integration tests.
- Actually, for integration tests, the crate must be a library.

#### [MODIFY] [Cargo.toml](file:///home/vlisivka/workspace/slint_kanban/Cargo.toml)
- Ensure the project is both a library and a binary if we want to use `tests/` directory effectively for unit-style integration tests.
- Alternatively, move them to a submodule `src/tests/` and keep as unit tests.

> [!NOTE]
> Moving to `tests/` makes them integration tests, which means they only test the public API. If any tests require private access, they should stay in `src/` but in separate files like `src/model/tests.rs`.

Given the project structure, I will:
1. Convert `src/main.rs` to only handle the entry point and move everything else to `src/lib.rs`.
2. This allows `tests/*.rs` to work properly.

## Verification Plan

### Automated Tests
- Run `cargo test` to ensure all tests pass in their new locations.
