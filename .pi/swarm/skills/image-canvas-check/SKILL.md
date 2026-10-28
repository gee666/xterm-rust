---
name: image-canvas-check
description: Use when changing browser image canvas operations, terminal lifecycle, storage resources, cropping, scaling, placeholders or pixel evidence.
---

# Image canvas and lifecycle checks

Run `node scripts/image-canvas-check.mjs` for the canvas suite. Run `WASM_TEST=image_lifecycle_browser node scripts/image-canvas-check.mjs` for live Terminal lifecycle tests. Both compile the full xterm-web crate, bundle the unchanged source renderer as comparator and run Rust assertions in Chromium, Firefox and WebKit.

The old isolated-component fallback was removed once main browser producers compiled. Build errors now fail the full command; do not bypass them with a substitute library.

Prerequisites are wasm32-unknown-unknown, wasm-bindgen0.2.100, and matching source esbuild/sixel/Playwright/browser installations. SOURCE_REPO, REFERENCE_NODE_MODULES, WASM_BINDGEN, PLAYWRIGHT_MODULE and BROWSERS override locations/selection. Missing builds, browsers and zero exports fail. The runner closes browser/server resources.

Retain results/image-canvas and results/image-lifecycle verification files, source hashes and immutable browser reports/logs. Seed0x272389 runs128 draw/extract pairs per browser; seed0x127389 runs64 lifecycle actions per browser. Native/workspace gates and failures remain separate. Full protocol/addon activation and original full-addon suites remain task38. See docs/image-canvas.md and docs/image-lifecycle.md.
