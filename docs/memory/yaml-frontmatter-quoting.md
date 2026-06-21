---
topic: yaml-frontmatter-quoting
updated: 2026-06-21
tags: [bug, yaml, serde, ticket, save]
---

# YAML Frontmatter Quoting Bug

## Key Learnings
- When manually constructing YAML frontmatter in Rust with `write!()`, all string values must be quoted to prevent serde_yaml from misinterpreting special characters.
- Unquoted values containing colons (e.g., "Error: colon in title") cause serde_yaml to fail with "mapping values are not allowed in this context at line 1 column X".
- The `save` function in `ticket.rs` already quoted `assigned_to` and `author` but missed `title`, `created_at`, and `updated_at`.

## Patterns
- **YAML string quoting**: Always use `"{}"` for string values in manually-constructed YAML, not `{}`.
- **Test-driven reproduction**: Write a failing test with the exact pattern from the bug report before fixing.

## Decisions
- Quote all string fields in YAML frontmatter consistently, even if they currently work without quotes (e.g., `created_at` format "YYYY-MM-DD HH:MM:SS" contains colons).

## Code Reference
- Bug location: `src/model/ticket.rs:288` in `Ticket::save()`
- Fix: Changed `title: {}` to `title: "{}"` in the format string.
