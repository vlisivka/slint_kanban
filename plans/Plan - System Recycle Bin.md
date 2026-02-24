# Plan: Switch from custom "Deleted/" folder to OS Recycle Bin

## Status
Completed!

## Changes Made
- Added the `trash = "5.2.5"` crate to dependencies.
- Updated `src/model/board.rs` `delete_ticket` method:
    - Instead of moving to `Deleted/ticket_id`, calls `trash::delete(&ticket_path)` via the system Recycle Bin implementation (for production runs).
    - For tests, `cfg(test)` performs `std::fs::remove_dir_all` to keep tests isolated.
    - Queue iterating removes strictly matched broken symlinks based on matching the symlink stem with `ticket_id`.
- Removed `Deleted/` checks in CLI removal checks/assertions. Tests verified to pass.
