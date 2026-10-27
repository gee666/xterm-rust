---
name: vt500-check
description: Use when validating or regenerating VT500 parser transitions against the unchanged upstream table initializer.
---

Run `node scripts/vt500-oracle.mjs` with Node 24 or later. It checks every Rust table and test fixture entry against fresh execution of the original TypeScript initializer. Source hashes and counts are in `results/vt500/oracle.json`.

Run `cargo test -p xterm-core --test parser_table` and targeted Clippy. The Rust tests need no Node or source reference. Retain logs under `results/vt500`. The independent parser integration task is not covered by these table tests.

Use `node scripts/vt500-oracle.mjs --generate` only after reviewing an intentional source change. Then rustfmt `crates/xterm-core/src/parser/vt500_table.rs` and rerun the normal check. See `docs/parser-table.md` for typed API mappings and invalid custom-write boundary differences.
