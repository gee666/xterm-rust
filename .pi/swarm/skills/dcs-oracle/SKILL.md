---
name: dcs-oracle
description: Use when verifying DCS dispatch, regenerating DCS oracle traces, or rerunning unchanged upstream DcsParser assertions against Rust.
---

# DCS oracle

Run from the repository root with Node.js 24.15 or later and Rust 1.85.1 or later available on PATH:

```sh
node scripts/verify-dcs.mjs
```

This runs all 20 unchanged upstream DcsParser.test.ts bodies, verifies a 512-trace corpus with seed 0xdcc512, runs Rust tests, checks formatting for owned DCS files, and runs focused warning-free Clippy. It writes commands, platform, toolchain and raw logs to results/dcs/. docs/dcs.md records Rust binding and async executor obligations.

To intentionally regenerate the corpus from unchanged upstream behavior:

```sh
node scripts/check-dcs.mjs --generate
```

Do not hand-edit crates/xterm-core/tests/fixtures/dcs-oracle.tsv. A failed comparison is a counterexample. The oracle rewrites only imports in memory, preserving original methods, test bodies and assertions. It does not modify source_repo. No npm dependencies are required.

On this Windows Git Bash host cargo tools may require adding C:/Users/user/.cargo/bin to PATH. That host-specific adjustment is not a portable project prerequisite.
