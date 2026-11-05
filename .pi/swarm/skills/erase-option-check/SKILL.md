---
name: erase-option-check
description: Use when changing ED2 raw scrollOnEraseInDisplay getter ownership, truthiness, buffer replacement, callback phases or errors.
---

# ED2 option checks

Reuse `scripts/erase-option-oracle.mjs` and `crates/xterm-core/tests/erase_option.rs`. Run the oracle with `XTERM_SOURCE_ROOT` pointing to the untouched source tree with its existing esbuild dependency. It verifies all 776 manifest entries before and after, rejects fixture drift, and removes its owned `tmp/erase-option-*` bundle in `finally`. Use `--generate` only when deliberately capturing reviewed unchanged-source behavior. Use `--no-receipt` for independent read-only replay.

Run `cargo test --locked -p xterm-core --test erase_option -j2` after acquiring the team's compiler allocation. The corpus compares both buffers and all cell fields, cursor, modes, dimensions, raw truthiness, synchronous getter/scroll callback checkpoints, original error identity and event ordering. Keep failed cases and negative controls. Never count this finite native corpus as browser, complete InputHandler suite or full equivalence evidence.

All new temporary work belongs in an owned O: worktree. Preserve source and concise receipts, inspect processes and reparse points, then clean owned scratch and release allocations. Do not change original source or revive the retired global queue.
