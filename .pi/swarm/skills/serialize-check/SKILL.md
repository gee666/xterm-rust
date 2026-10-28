---
name: serialize-check
description: Use when verifying ANSI and HTML terminal serialization against the source oracle and live Rust buffers.
---

# Serialize checks

Read `docs/serialize-acceptance.md` for the exact contract. Keep `source_repo` read-only. Output is compared as UTF-16, not lossy UTF-8.

1. Run `node scripts/serialize-oracle.mjs` to execute every original addon test and regenerate deterministic seeded fixtures. The source runner and fixture comparison belong to task 87. Retain source hashes, seed, test outcomes and failing cases under `results/serialize/`.
2. Run `cargo test -p xterm-core --test serialize_ansi --test serialize_native` for focused ANSI branch and live native-buffer borrow/error checks.
3. Run `cargo test -p xterm-core --test serialize_html` when HTML implementation task 103 is ready.
4. Run the independent fixture test identified in `results/serialize/oracle.json` and task 87 evidence.
5. Run focused Clippy with `-D warnings` and scoped rustfmt checks. Shared-library failures must stay visible and their owners must fix them. Do not suppress warnings or remove exports to make a focused test pass.
6. Task 88 verifies native live activation, disposal, modes and same-size source-compatible round trips. Run `node scripts/serialize-live-oracle.mjs` and `cargo test -p xterm-core --test serialize_live --test serialize_live_callbacks --test serialize_live_reentrancy`. Task 131 separately owns actual browser/WASM addon bindings and three-browser tests. Oracle output fixtures alone are not those integration checks.

Use the repository root target directory and existing Cargo profiles. Run commands with explicit timeouts, preserve bounded process behavior and check drive free space before large builds. Temporary artifacts belong in root `tmp/`.
