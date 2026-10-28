---
name: events-circular-list-check
description: Use when checking Event listeners or CircularList storage against upstream xterm.js.
---

Run from your worktree with Rust and Node 24 available. Keep source_repo as the unchanged reference.

- Fast Event review: `cargo test -p xterm-core --test event_context --test event_late_collection`.
- Fast ring review: `cargo test -p xterm-core --test circular_list_splice --test circular_list_numbers`.
- Full component oracle: `node scripts/check-events-and-circular-list.mjs`.
- Source-only checks without a compiler: add `--source-only`.

The full oracle currently passes 96 Rust tests, 26 unchanged source tests containing 95 assertions, 25 directed source checks and 30,055 seeded states/traces. Seed is `0xc1ac01a7`. It compares slots, lengths, return/error values and ordered callback snapshots. Failing prefixes are retained.

The runner uses normal scoped Cargo commands, so it respects the worktree's target. `--isolated` is only a utility-debug fallback, not workspace proof. No global build lease or artifact pinning is needed.

For exact acceptance cases see docs/acceptance/events-circular-list.md. This component result does not prove the whole terminal, browser or every platform equivalent. Report review results briefly with the command and outcome.
