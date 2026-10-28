---
name: browser-mouse-check
description: Use when validating browser mouse coordinates, report routing, wheel/touch state and original mouse assertions against the Rust policy implementation.
---

Run `node scripts/check-browser-mouse.mjs` from the repository root with Node24, source_repo's locked dependencies, Rust1.85+, rustfmt/Clippy and wasm32-unknown-unknown installed. Use the shared Cargo profiles/target directory and check disk before building. The checker stops below512MiB free and retains failed/unrun stages. Do not start redundant builds during a resource pause.

The source oracle rebuilds generated JS using the reference esbuild script, then executes all15 unchanged original Mouse/MouseService assertions through Mocha. It captures every source trigger invocation and generates deterministic geometry, wheel and touch traces. Source stays read-only. The typed Rust tests compare all1,562 trigger calls,1,024 geometry rows,2,522 wheel rows and516 touch rows at seed0xb0a5e. Floating-point checks preserve NaN and signed zero; encoded output compares exact UTF16 units and binary/data channels.

The checker runs12 native Rust tests, focused formatting, Clippy with warnings denied and a WASM compilation check. `results/browser-mouse/latest.json` points to immutable per-attempt command logs, hashes and summary. This is policy/source/WASM-compile evidence, not actual DOM input or a three-browser pass. Tasks33/110 retain those live browser obligations.

Use `node scripts/browser-mouse-oracle.mjs --generate` only after inspecting unchanged reference hashes and proposed fixture changes. Normal runs reject drift. Retain any first failing row as a counterexample; do not edit expected values manually.

See `docs/browser-mouse.md` for source-to-Rust assertion mapping, callback/error ordering, remaining DOM work and the public policy API. For actual Rust WASM tests in browser engines, reuse the browser-host-check skill with an appropriate WASM_TEST target and all3 browsers. Do not count native tests or a cross-compile as browser runtime proof.
