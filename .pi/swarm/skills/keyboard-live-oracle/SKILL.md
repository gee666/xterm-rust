---
name: keyboard-live-oracle
description: Use when changing legacy keyboard live-property reads, numeric key codes, case conversion, short-circuit paths or getter error identity.
---

# Live legacy keyboard oracle

Read docs/acceptance/legacy-live-keyboard.md and docs/live-keyboard-event.md. Reuses verify-keyboard's unchanged-source TS-erasure method without a new package dependency.

After the coordinated disk/build pause is lifted, run node scripts/keyboard-live-oracle.mjs --generate once for reviewed initial fixture creation, then node scripts/keyboard-live-oracle.mjs for drift checks. The generator loads the original evaluator and enum/constants, wraps live getters with value sequences, records every read/result, and injects the same marker error at every reached boundary for branch/mutation families. Every changed BMP uppercase mapping is exercised through a changing final Alt-letter numeric getter. Reports retain exact counts, seed and hashes under results/keyboard-live.

The Rust replay test is developed under tmp/keyboard_live.rs until initial fixtures exist; move the complete verified test to crates/xterm-core/tests/keyboard_live.rs and replace its draft minimum-count assertion with the exact generated count before requesting review. Do not claim this draft as completed evidence.

Also rerun node scripts/verify-keyboard.mjs and the original keyboard_upstream/keyboard_edges Rust tests, then debug/release, fmt and warning-free Clippy for changed files. Preserve static public APIs and native outputs. This oracle is not actual browser DOM integration; tasks151/33/141 separately verify those paths and original JsValue errors. Never bypass a resource pause or add duplicate target directories to obtain a green result.
