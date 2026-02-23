# Plan - Implement Points Field in Model

## Goal
Add the `points` field to `TicketMetadata` and `Ticket` structs in `src/model/ticket.rs` and ensured it is properly serialized/deserialized and saved to disk.

## Changes
### `src/model/ticket.rs`
- Add `pub points: u32` to `TicketMetadata`.
- Add `pub points: u32` to `Ticket`.
- Update `Ticket::from_metadata` to copy `points`.
- Update `Ticket::save` to include `points` in the YAML frontmatter.
- Update `Board::create_ticket` (in `src/model/board.rs`) and `Board::update_ticket` signatures if necessary (though `update_ticket` usually takes full `Ticket` or specific fields). Wait, `Board::update_ticket` takes specific fields in the current implementation.

### `src/model/board.rs`
- Update `create_ticket` to accept `points`.
- Update `update_ticket` to accept `points`.

## Tests
- Update `test_ticket_metadata_deserialization` in `src/model/tests/ticket_tests.rs`.
- Add a new test case for saving and loading a ticket with points.
- Ensure existing tests pass.

## Manual Verification
- Check if old tickets still load correctly (points should default to 0).
- Check if new tickets can be saved with points and re-read correctly. (This will be easier once UI/CLI is updated, but can be verified with unit tests now).
