# Backend Refactoring Plan - Modularization

This plan covers splitting the large `src/model.rs` and `src/main.rs` files into smaller, more manageable modules.

## Proposed Structure

### 1. Data Model (`src/model/`)
Move away from a single 600-line file to a dedicated directory:
- `src/model/mod.rs`: Module entry point.
- `src/model/ticket.rs`: `Ticket` and `TicketMetadata` (parsing, markdown references).
- `src/model/queue.rs`: `Queue` structure and per-queue logic.
- `src/model/board.rs`: `Board` orchestration (loading, moving tickets, creating tickets).
- `src/model/config.rs`: `Config` (search history, global settings).

### 2. Tests partitioning
Split the massive `src/model/tests.rs` into matching test files:
- `src/model/tests/mod.rs`
- `src/model/tests/ticket_tests.rs`
- `src/model/tests/board_tests.rs`
- `src/model/tests/config_tests.rs`

## Implementation Steps

### Phase 1: Model Splitting - [DONE]
1. Create `src/model/` directory. - [DONE]
2. Extract `Config` to `src/model/config.rs`. - [DONE]
3. Extract `Ticket` and `TicketMetadata` to `src/model/ticket.rs`. - [DONE]
4. Extract `Queue` to `src/model/queue.rs`. - [DONE]
5. Extract `Board` to `src/model/board.rs`. - [DONE]
6. Create `src/model/mod.rs` to re-export types. - [DONE]

### Phase 2: Test Splitting - [DONE]
1. Create `src/model/tests/` directory. - [DONE]
2. Move relevant tests from `src/model/tests.rs` to individual files. - [DONE]

### Phase 3: Cleanup - [DONE]
1. Remove old `src/model.rs` and `src/model/tests.rs`. - [DONE]
2. Ensure all imports are correct. - [DONE]
3. Verify with `cargo test`. - [DONE]

## File Documentation Standards
Every new file MUST start with a header comment:
```rust
//! [File Name]
//!
//! Purpose: [Brief description]
//! Includes: [List of main structs/functions]
//! Constraints: [What should NOT be here]
```
