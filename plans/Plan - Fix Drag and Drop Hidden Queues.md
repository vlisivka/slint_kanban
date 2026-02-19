# Implementation Plan: Fix Drag and Drop with Hidden Queues

This plan covers the fix for incorrect target queue resolution when some queues are hidden.

## Problem
The UI calculates the drop target index based on the mouse position relative to visible columns. However, the backend `resolve_queue_id` uses this index to look up a queue in the global `self.queues` list, which includes hidden queues. This causes an off-by-N error when hidden queues exist to the left of the drop target.

## Proposed Fix
Update `Board::resolve_queue_id` in `src/model.rs` to filter for visible queues before applying the index.

## Task List
- [x] Update `Board::resolve_queue_id` to:
    - Get all visible queues first.
    - Resolve the index against the list of visible queues.
- [x] Add a unit test to verify this behavior.
- [x] Update `TODO.md` to mark the fix as complete.
