---
name: mode-query-option-check
description: Use when changing ANSI20 or DEC12 DECRQM raw option getter borrowing, truthiness, reply ordering or callback errors.
---

# DECRQM option checks

Run from an isolated O: worktree with a private regular Cargo target. Coordinate a compiler grant first. Do not use the retired build-slot queue.

With Node and the reference's installed esbuild, run:

```sh
XTERM_SOURCE_ROOT=/absolute/read-only/source_repo node scripts/mode-query-option-oracle.mjs
cargo test --locked -j2 -p xterm-core --test mode_query_option
cargo test --locked -j2 -p xterm-core --test terminal_modes --test terminal_mode_callbacks --test terminal_modes_differential
```

The source runner selects all eight unchanged original InputHandler DECRQM tests separately from its added corpus. It uses its own worktree-root tmp directory, removes the bundle in finally, and verifies all 776 original reference byte/hash records before and after. `--generate` is an explicit fixture update, never a way to normalize Rust failures.

The fixture contains 432 added source-derived cases, not 432 unchanged upstream tests. ANSI20 convertEol and DEC12 cursorBlink queries each use multiple parameters to check first-parameter-only behavior. Both buffers, nine raw values and twelve callback actions compare full cell/attribute/buffer/mode snapshots, getter counts, callback-time state, replies and ordered events. Actions include reset, resize, buffer activation, cursor and mode mutation, disabled stdin, getter throws and data-observer reset/throws. Source and Rust must preserve the original thrown error identity.

For a sensitivity check, save exact bytes/hashes of mode_handlers.rs and mode_query.rs. Temporarily use both files from the retained main1106 baseline, run the new Rust test expecting 432 mismatches, then restore both exact byte buffers in finally and rerun the fixed test. Never run controls on shared source. Record original/fixed hashes and logs. Broad parser, browser and platform equivalence remain separate obligations.
