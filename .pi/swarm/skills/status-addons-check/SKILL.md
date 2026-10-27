---
name: status-addons-check
description: Use when validating program-status OSC7501 and progress OSC9 addons against the supplied xterm.js reference.
---

Run from repository root. Do not modify source_repo.

1. `node --experimental-transform-types scripts/status-addons-oracle.mjs` executes 38 unchanged pure ProgramStatus upstream unit cases, captures every parser invocation, generates seeded malformed/boundary inputs and progress traces. Source and fixture SHA-256 hashes and exact counts go to `results/status-addons/oracle.json`. Seed is `0x750109`. Temporary copies use `tmp/status-addons`.
2. `cargo test -p xterm-addons --test status_addons_differential --test program_status_store --test program_status_addon --test progress_upstream` compares 2,215 parser cases and 2,048 sequential progress operations, faithfully ports 11 store assertions and ten upstream progress assertions, and tests registration fixtures.
3. `cargo clippy -p xterm-addons --all-targets --all-features -- -D warnings`, `cargo fmt --all -- --check`, and full workspace test gates must pass. Do not suppress unrelated core failures. Record exact failures and notify the owning agent.
4. Run actual terminal and Rust-backed Chromium/Firefox/WebKit integration suites separately. These fixture tests cannot prove chunking, terminal reset, DA1 query ordering, browser loadAddon or native platform equivalence.

Read `docs/status-addons-acceptance.md` for required edge cases. Keep all source assertions intact. Generated fixtures are test data, not hand-edited expected results. Always set command timeouts and respect the workspace disk-saving Cargo configuration. Do not create separate target directories.
