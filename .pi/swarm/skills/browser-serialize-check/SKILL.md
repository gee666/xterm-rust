---
name: browser-serialize-check
description: Use when verifying callable SerializeAddon ANSI/HTML output, actual host selection/theme/reentry, public cell enumeration and unchanged browser restoration assertions.
---

# Browser serialization checks

Read `docs/browser-serialize.md` and `docs/serialize-browser-acceptance.md`. The native-only serialize skills do not establish browser compatibility.

1. Obtain a current heavy/browser allocation from the swarm resource owner. Do not use the retired global build queue. Use an owned O: worktree/target and `CARGO_BUILD_JOBS=2`.
2. Set `WASM_BINDGEN` to the installed CLI matching runtime0.2.100, then run `node scripts/build-demo.mjs --debug`. The following command verifies that the current source matches this actual Rust/WASM package.
3. Set `SOURCE_ROOT` to the unchanged supplied reference with its existing generated demo bundle and Playwright dependencies. Run `node scripts/serialize-browser-check.mjs --suite`. This uses the actual compatible `browser/xterm.mjs` export, identical directed/seeded source/Rust operations, and unchanged SerializeAddon suites in all three engines.
4. Read `results/serialize-browser/verification.json` and the source/Rust traces. Report discovered/executed/passed/failed/skipped/unrun separately. A raw-number mismatch deliberately returns nonzero. Do not filter it into a passing total or weaken original assertions. Task346 retains full raw serialization/GC closure.
5. Run `node --test scripts/buffer-shape.test.mjs`, focused native serialize and cell tests, warning-denied native/WASM Clippy and `cargo fmt --all -- --check`. No `toJSON` shim or assertion normalization substitutes for actual public field semantics.
6. Preserve concise raw results/counterexamples before cleanup. The runner closes its own browsers/servers and removes transient `tmp/serialize-browser`. Remove only your owned remaining build/generated outputs after review/integration. Never recursively remove a dependency junction.

The fixed seed is `0x05e21a11`, 32 traces of12 operations. A passing suite still does not prove every SB01-SB08 contract, full demo scope, full internal events/margins or native cross-platform behavior.
