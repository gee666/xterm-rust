---
name: cell-oracle
description: Use when changing packed cell or attribute behavior, regenerating cell fixtures, or checking upstream CellData assertions.
---

# Cell oracle

Run `node scripts/cell-oracle.mjs` from the repository root with Node 24.15 or newer. It uses native TypeScript transformation, loads read-only upstream classes, runs all five CellData.test.ts tests and the seven AttributeData/CellData tests embedded in BufferLine.test.ts. It does not claim the remaining BufferLine suites run.

The script writes transpiled temporary modules under `tmp/cell-oracle`, source hashes and baseline counts to `results/cell/oracle.json`, and fixtures under `crates/xterm-core/tests/fixtures/cell-*.tsv`. Seed `0xce11da7a` produces 2048 raw packed cells and attribute comparisons, 1024 legacy CharData cases including malformed UTF-16 and raw JS width shifts, and 4096 extended mutation steps across shared/cloned objects. Fixture SHA-256 values are retained in the oracle report. Regenerate twice and compare hashes to check reproducibility.

Run `cargo test -p xterm-core --test cell_upstream --test cell_differential --test cell_branches --test cell_extended`, then repeat with `--release` and focused Clippy `-- -D warnings`. The Rust assertions check packed content, every attribute query, UTF-16 strings, URL underline overrides, and attribute equality. On Git Bash add `/c/Users/user/.cargo/bin` to PATH if needed. Keep all temporary output inside repository-root `tmp/` and supply timeouts for shell commands. Do not edit fixtures by hand to hide a mismatch.
