---
name: color-check
description: Use when regenerating faithful Color.ts assertions or checking Rust color behavior against the original Node implementation.
---

# Color verification

Run from the repository root with Node 24 and Rust 1.85 or later on PATH. This skill uses checked-in `tools/generate_color_tests.mjs`, `tools/check_color.mjs`, and `crates/xterm-core/examples/color_oracle.rs`.

1. Install isolated oracle dependencies with `npm install --prefix tmp/color-oracle --no-audit --no-fund --ignore-scripts chai@6.2.2 mocha@11.7.5`.
2. Run `node tools/generate_color_tests.mjs`. It rejects unrecognized assertion syntax and requires every upstream assertion to be translated.
3. Run `rustfmt --edition 2021 crates/xterm-core/tests/color_upstream.rs`.
4. Run `cargo test -p xterm-core --test color_upstream --test color_edges`.
5. Run `node tools/check_color.mjs`. It transforms TypeScript only into repository-root tmp files, removes one type-only import, resolves imports, and runs original assertion bodies with real Chai/Mocha. It then compares seeded Rust/JavaScript operations.
6. Inspect `results/color/original.json`, `results/color/differential.json`, and retained `results/color/counterexamples.json`. A retained counterexample can be a fixed regression; `docs/color.md` distinguishes its status.

Every shell tool invocation needs an explicit timeout. Do not edit source_repo to fix a failure. Browser canvas fallback requires separate adapter integration and browser evidence. These checks do not establish complete terminal or cross-platform equivalence.
