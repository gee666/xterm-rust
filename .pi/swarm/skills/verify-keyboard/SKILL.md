---
name: verify-keyboard
description: Use when verifying the Rust legacy keyboard policy against xterm.js Keyboard.ts and its unchanged upstream assertions.
---

# Legacy keyboard verification

From the repository root, run `node scripts/verify-keyboard.mjs` with Node.js supporting `stripTypeScriptTypes`, Unicode 17.0 data, and rustc on PATH. RUSTC may select an explicit compiler. The script reuses the Win32 verifier's TS-erasure/oracle method without adding a dependency.

It executes every upstream Keyboard.test.ts body and assertion, checks the checked-in Rust fixture, compiles a temporary Rust runner under `tmp/keyboard`, and compares result kind, cancellation and UTF-16 payload for key-code/modifier/options combinations, mobile arrows, every BMP single-unit key and dead-key lowercase case, and 20,000 seeded inputs. A failure saves `results/keyboard/counterexample.json`. The success report is `results/keyboard/verification.json`.

Use `node scripts/verify-keyboard.mjs --generate` only after reviewing reference changes. It regenerates `crates/xterm-core/tests/fixtures/keyboard_upstream.tsv` and the Unicode 17.0 BMP lowercase table. RUSTFMT may select an explicit formatter. Preserve failing counterexamples even after fixes.

Run `cargo test -p xterm-core --test keyboard_upstream --test keyboard_edges` and workspace fmt/Clippy/tests. This checks portable legacy key policy, not actual browser event wiring or complete terminal equivalence.
