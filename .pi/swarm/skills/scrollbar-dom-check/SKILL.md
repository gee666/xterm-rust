---
name: scrollbar-dom-check
description: Use when checking Rust scrollbar layout, pointer monitoring, arrow timers and trusted browser drag/capture.
---

Run in your isolated worktree with Cargo, Node, Playwright and wasm-bindgen available. Reuse `scripts/browser-host-check.mjs` for Rust/WASM assertions:

```
WASM_TEST=wasm_pointer_monitor node scripts/browser-host-check.mjs
WASM_TEST=wasm_scrollbars node scripts/browser-host-check.mjs
WASM_TEST=wasm_scrollbar_arrow node scripts/browser-host-check.mjs
node scripts/scrollbar-drag-check.mjs
```

The suites contain eight pointer, six layout/drag and three deterministic arrow tests. No upstream *.test.ts exists for these internal scroll classes, so these are source-derived tests. The last command reuses the compiled fixture and drives trusted Playwright mouse events in Chromium, Firefox and WebKit. It asserts actual pointer capture, outside-node movement, release, drag callbacks and listener removal for both orientations. It closes browsers and its server in finally blocks.

Use the existing `WASM_BINDGEN`/`PLAYWRIGHT_MODULE` overrides where required. Results are in `results/browser-host/` and `results/scrollbars/trusted-drag.json`. Do not count synthetic fallback tests as trusted capture evidence. Raw malformed scroll-position compatibility remains task215, not a waived pass.
