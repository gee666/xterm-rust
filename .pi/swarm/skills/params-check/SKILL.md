---
name: params-check
description: Use when validating parser Params storage against the original TypeScript class and seeded operation corpus.
---

Run `node scripts/check-params.mjs` under Node 24 or later. The script transforms only TypeScript syntax and its type-only import, then compares a regenerated seeded operation corpus to the committed fixture. Use `--generate` only after reviewing reference changes. Source and corpus hashes, seed and counts are in `results/params/oracle.json`.

Run `cargo test -p xterm-core --test params --test params_differential` for all 20 migrated upstream cases, extra boundaries and 49,409 oracle snapshots. These tests do not require Node. Run targeted rustfmt and Clippy and retain output under `results/params`. API conversion decisions are in `docs/params.md`. This verifies Params only, not the whole terminal.
