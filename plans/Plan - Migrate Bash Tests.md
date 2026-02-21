# Plan - Migrate Bash CLI Tests to Rust

## Current State
The project has a suite of CLI tests written in Bash (`tests/cli/*.sh`) and another integration test suite written in Rust (`src/main_tests.rs`). Currently, some commands are tested in both places, some only in Bash, and some only in Rust. We want to consolidate everything into Rust because Rust tests are cross-platform natively, integrate with 'cargo test', provide better type safety, and have easier access to the internal data structures for validation compared to parsing CLI stdout.

Currently, `test_cli_add`, `test_cli_update`, `test_cli_move`, `test_cli_remove`, `test_cli_change_limit`, `test_cli_comment`, and `test_cli_attach` are implemented in `src/main_tests.rs`.

The bash tests cover:
- `test_configure.sh` -> Not yet in Rust
- `test_add.sh` -> Partially in Rust (`test_cli_add`), needs missing queue/title tests.
- `test_list.sh` -> Not yet in Rust (List with user filters, searches, missing fields).
- `test_show.sh` -> Not yet in Rust (Show validations, invalid ID checks).
- `test_update.sh` -> Partially in Rust (`test_cli_update`), needs checking if fields are successfully empty.
- `test_move.sh` -> Partially in Rust (`test_cli_move`), needs checking invalid target queue failure.
- `test_remove.sh` -> Partially in Rust (`test_cli_remove`), needs checking invalid ID.
- `test_attach.sh` -> In Rust (`test_cli_attach`).
- `test_open.sh` -> Just opens GUI, ignorable or can add mock test.

## Execution Plan

### Step 1: Migrate `configure` Tests
- Create `test_cli_configure` in `src/main_tests.rs`.
- Test `--add-user` (add to `users` array).
- Test `--active-user`.
- Test `--show-only-mine`.
- *Note*: Verify state changes via `Board::load(root)`.

### Step 2: Migrate Error Scenarios for Existing Tests (Add, Update, Move, Remove)
- `run_cli` currently returns `anyhow::Result<()>`. We can assert that the result `is_err()` for invalid inputs.
- Ensure `test_cli_move` handles moving to "non-existent-queue".
- Ensure `test_cli_remove` handles removing a fake ticket.
- *Note*: Tests for missing `--title` in clap are handled by Clap's internal validation, so we just focus on logic errors.

### Step 3: Output Redirection Refactoring
- **COMPLETED**: Modified `handle_command` and `run_cli` in `src/main.rs` to take `mut out: impl std::io::Write`. Replaced `println!` with `writeln!(out, ...)`. `main_tests.rs` adjusted to pass `&mut std::io::stdout()`.

### Step 4: Implement List/Show tests
- Implement `test_cli_list` using output capturing (pass `Vec<u8>` to `run_cli`). Add specific assertions looking for listed tickets or hidden ones.
- Implement `test_cli_show` using output capturing and test invalid IDs.

### Step 5: Cleanup
- Remove the `tests/cli` directory (`rm -rf tests/cli`).
- Remove the call to `tests/cli/run_all.sh` from `run-all-tests.sh`.
- Run `cargo test` to ensure 100% success.
