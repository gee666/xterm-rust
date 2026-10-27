---
name: buffer-line-oracle
description: Use when changing BufferLine storage, editing, copying, resizing, UTF-16 translation or text caching, or regenerating its independent JavaScript fixtures.
---

# BufferLine oracle

Run from the repository root with Node 24.15 or newer and Rust 1.85 or newer. Supply explicit shell timeouts. Temporary transpiled modules stay under `tmp/buffer-line-oracle/`.

```sh
node scripts/buffer-line-oracle.mjs
cargo test -p xterm-core --test buffer_line_upstream --test buffer_line_translation --test buffer_line_extended --test buffer_line_differential
cargo test -p xterm-core --lib buffer::line
cargo clippy -p xterm-core --test buffer_line_upstream --test buffer_line_translation --test buffer_line_extended --test buffer_line_differential -- -D warnings
```

The script transpiles read-only upstream classes into temporary modules. It extracts the exact `createCellData`, `extendedAttributes`, and `NULL_CELL_DATA` helper definitions instead of loading unrelated terminal mocks. It replaces only imports in the original test file. All 51 original test bodies and assertions run unchanged against JavaScript. Node strict assertions stand in for the three used Chai assertion methods.

The script retains source SHA-256 hashes, discovered/executed/passed/failed counts, every original test outcome, seed `0xb0ff113e`, and fixture hash in `results/buffer-line/oracle.json`. The fixture is `crates/xterm-core/tests/fixtures/buffer-line-traces.tsv`. Do not hand-edit expected results to hide a mismatch.

The fixture has 128 traces of 64 operations each. It covers all 14 mutation families, lengths zero through 32, protected attributes, malformed UTF-16, combining strings, astral strings, both overlap traversal directions, cleanup return values, and copying. Every step compares all active packed cells and primitive getters, extended words and URL, wrapped state, trimmed lengths, complete and partial UTF-16 translations, complete and partial column maps, and canonical cache results. A separate Rust regression checks shared extended mutation and payload identity through all copy paths.

Read `docs/criteria/buffer-line.md` for the complete subsystem acceptance contract. These checks do not demonstrate full terminal integration or browser behavior, and Windows execution is not native Linux/macOS evidence.
