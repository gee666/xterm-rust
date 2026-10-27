---
name: buffer-range-marker-check
description: Use when checking inclusive buffer ranges and marker identity, lifecycle ordering, callback failures and ownership against the unchanged TypeScript reference.
---

Run from the repository root with Node 24 or newer and Rust 1.85 or newer on PATH:

```sh
node scripts/check-buffer-range-marker.mjs
cargo test -p xterm-core --test buffer_range_marker
cargo test -p xterm-core --lib buffer::marker::tests
cargo clippy -p xterm-core --lib --test buffer_range_marker -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/buffer/range.rs crates/xterm-core/src/buffer/marker.rs crates/xterm-core/tests/buffer_range_marker.rs
```

Set a timeout for the shell command. Keep shared .cargo/config.toml profiles unchanged and use the shared target directory. Monitor available disk before large test matrices.

The Node script transforms original source into temporary modules under tmp/buffer-range-marker/, removing type-only imports and changing relative runtime imports only. It executes the four original BufferRange cases with strict Node assertion equivalents. It checks a committed 1,149-row oracle fixture against fresh JavaScript results and compares a shared marker callback trace. It adds direct JS checks for live resource registration, event failure, invalidation, ids and post-disposal mutation.

The Rust suite verifies the same range fixture and marker trace plus typed-domain, failure, reentrancy, snapshot and weak-ownership cases. The allocator test covers the 2^53 transition without changing shared global state. Use `--write-fixtures` on the Node script only for intentional reviewed regeneration. Hashes, runtime, seed 1112690247 and counts are recorded under results/buffer-range-marker/.

These are utility checks. Real Buffer trim/delete/reflow/resize integration and the five original Buffer marker cases belong to task27 and criterion122. Do not claim those pass solely from this skill. See docs/buffer-range-marker.md and docs/acceptance-color-range-marker.md for typed bindings, fallible disposal mapping and exact acceptance cases.
