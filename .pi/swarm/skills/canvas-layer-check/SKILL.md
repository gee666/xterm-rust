---
name: canvas-layer-check
description: Use when changing WebGL base/link canvas geometry, text clipping, theme refresh, transparency or retained underline state.
---

# Canvas layer checks

Read `docs/webgl-canvas-layers.md` for typed representation boundaries and the remaining renderer/Addon/Root scope. Reuse the existing browser runner and shared atlas cache.

1. With the original source's installed esbuild and `SOURCE_ROOT` pointing at that untouched reference, run `node scripts/canvas-layer-oracle.mjs`. It bundles the unchanged BaseRenderLayer, LinkRenderLayer and real source atlas/services in memory.
2. With a coordinator allocation, set `WASM_TEST=wasm_webgl_canvas_layer`, `WASM_PRELUDE=tmp/canvas-layer/source.js`, private O: target/TMP/TEMP and Cargo -j2. Run `node scripts/browser-host-check.mjs` in Chromium, Firefox and WebKit. Each engine must run every assertion with no ignored/filtered tests.
3. Run focused native `webgl` tests and warning-denied native/WASM all-feature lib/test Clippy. Run member and exact workspace formatting separately. Do not call an OS206 workspace failure a pass.
4. Run `node scripts/canvas-layer-negative.mjs` under the same browser allocation. It inverts the production middle-row clearing condition, requires actual Rust failures in Chromium and restores exact source bytes in finally. Then run the complete restored three-engine suite and strict WASM Clippy.
5. Retain raw failures, final reports, unchanged source hashes, test/probe hashes, and served compiler/binding/snippet identities before owned cleanup. Producer pin recording is not independent verification.

This is actual canvas component coverage. It is not a WebglRenderer, WebglAddon or complete original-suite pass. No JavaScript terminal engine executes Rust behavior.
