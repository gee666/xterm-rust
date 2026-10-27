---
name: verify-kitty-keyboard
description: Use when independently checking Rust Kitty keyboard encoding against unchanged xterm.js assertions and complete UTF-16, flag, modifier, physical-key and event matrices.
---

# Kitty policy oracle

Run `node scripts/verify-kitty-keyboard.mjs` from the root. Node.js must support `stripTypeScriptTypes` and use Unicode 17.0. Put cargo and rustc on PATH, or set CARGO and RUSTC. CARGO_TARGET_DIR selects the Cargo output directory.

The verifier executes unchanged `KittyKeyboard.test.ts` assertions with beforeEach hooks, checks fixtures byte-for-byte, builds `kitty_oracle`, and compares full result kind, cancellation and UTF-16 payloads. It includes every source mapping, unknown F26-F35 names, inherited Object.prototype names, all flag/modifier/event combinations, physical-code/mac cases, three full BMP passes, 25,000 seeded events and 5,000 seeded long numpad decimal suffixes. Seed and exact counts appear in `results/kitty-keyboard/verification.json`. Failures save counterexamples and are never deleted by successful reruns.

`node scripts/verify-kitty-keyboard.mjs --generate` regenerates only the two upstream fixtures. Review reference drift before using it. Do not weaken source assertions.

Also run `cargo test -p xterm-core --test kitty_upstream --test kitty_edges` and warning-denied Clippy. Source review checklist is `docs/kitty-acceptance.md`; independent findings and execution status are `docs/kitty-verification.md`.

This verifies the portable policy only. Browser dispatch, active-mode negotiation, native platforms other than the recorded host and browser-engine-specific native function strings need separate evidence.
