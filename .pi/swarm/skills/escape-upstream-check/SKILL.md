---
name: escape-upstream-check
description: Use when verifying the original escape parser synchronous assertion migration and its source-case mapping.
---

Run `node scripts/escape-example-fixtures.mjs` to check every literal input/expectation in all21 source examples. Run `node scripts/escape-test-mapping.mjs` to check the pinned original source hash and all161case identities, including147mapped synchronous cases and14async cases owned separately.

Run `cargo test -p xterm-core --test escape_upstream`, the same command with `--release`, focused Clippy with `-D warnings`, and rustfmt check on the root test. The mapping requires the TypeScript package installed by original source_repo setup. Fixture generation uses Node24 TypeScript transformation and normal Cargo-home rustfmt, overridable with RUSTFMT. Use --generate only after reviewing source changes.

Read docs/escape-upstream-tests.md for representation adaptations, the43 Rust test functions,224 synchronous direct assertion sites and60helper sites. Static sites are not dynamic assertion invocations. Retain results under results/escape-upstream; task26 separately owns parser implementation and async/differential evidence. Do not count original JS baseline as Rust coverage.
