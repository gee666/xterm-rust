---
name: string-builder-check
description: Use when validating UTF-16 string builder accumulation and payload limits against upstream xterm.js.
---

Prerequisites: Node 24 supporting `--experimental-transform-types`, Rust 1.85+, and the unchanged `source_repo` reference.

1. Run `node --experimental-transform-types scripts/string-builder-oracle.mjs` from the repository root. This executes all 17 upstream StringBuilder tests unchanged except module/assertion imports in a temporary copy. It imports original implementation code, generates 264 deterministic traces containing 16,480 operations with seed `0x51b01d`, and records source and fixture SHA-256 hashes in `results/string-builder/oracle.json`.
2. Run `cargo test -p xterm-core --test string_builder_upstream --test string_builder_differential`. All 17 upstream cases must pass. Four additional tests check trace equivalence, UTF-16 boundaries, UTF-8 helper behavior and maximal representable Rust limits.
3. Run `rustfmt --edition 2021 --check crates/xterm-core/src/string_builder.rs crates/xterm-core/tests/string_builder_upstream.rs crates/xterm-core/tests/string_builder_differential.rs` and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
4. Retain exact commands and results in `results/string-builder/`. Do not mistake this utility-only evidence for terminal, browser or platform equivalence.

The generated TSV oracle fixture is checked in so Rust tests do not require Node. Regenerate it from the reference whenever code or reference changes, and inspect fixture differences. Each TSV row records trace id, limit or `-` for unlimited, action, input UTF-16 hex units, overflow, output UTF-16 length, and output units. Empty final fields are significant. Failures identify the row to retain as a counterexample.

All temporary baseline modules are under repository-root `tmp/`. No source reference file is rewritten. Use explicit tool timeouts when invoking commands.
