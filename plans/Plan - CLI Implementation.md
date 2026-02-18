# Plan - CLI Implementation

This plan covers the initial steps of implementing the Command Line Interface (CLI) for Slint Kanban, focusing on refactoring for testability and setting up the command structure.

## Proposed Changes

### Core Logic Refactoring

#### [MODIFY] [main.rs](file:///home/vlisivka/workspace/slint_kanban/src/main.rs)
- Extract main function into a `run_main(args)` function.

### Dependency Update

#### [MODIFY] [Cargo.toml](file:///home/vlisivka/workspace/slint_kanban/Cargo.toml)
- Add `clap = { version = "4.5", features = ["derive"] }`.

### CLI Implementation

#### [NEW] [cli.rs](file:///home/vlisivka/workspace/slint_kanban/src/cli.rs)
- Define `CliArgs` struct using `clap`.
- Define subcommands: `add`, `update`, `move`, `remove`, `open`.

## Verification Plan

### Automated Tests
- Create unit tests that call `run_main` with mock arguments and verify filesystem changes.
- Create unit tests that call `run_main` with some missing arguments.
- Create unit test that call `run_main` with --help or -h for built-in help.

### Manual Verification
- Run `cargo run -- add -t "Test" -d "Desc" -q "1. Incoming"` and check if ticket appears on disk.
- Run `cargo run -- open .` to ensure GUI still opens correctly.
