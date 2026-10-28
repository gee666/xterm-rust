---
name: dom-font-check
description: Use when changing raw font/CSS callers, cache construction timing, measurement reentry or exception forwarding in DomRenderer and CharSizeService.
---

# Raw font callers

Run in an isolated worktree. Keep source_repo read-only. If reference dependencies live elsewhere, set SOURCE_ROOT to that reference directory for the source trace and PLAYWRIGHT_MODULE to its node_modules/playwright/index.mjs for browser checks.

- `node scripts/dom-font-oracle.mjs` transpiles unchanged reference TypeScript in memory, checks four directed font-caller traces, and verifies the retained code-unit fixture. Canvas and clocks are controlled source doubles, not pixel evidence.
- `cargo test --locked -p xterm-web --lib --test theme_css --test theme_css_raw` covers portable caches and lossless CSS fixtures.
- `WASM_TEST=dom_font_browser node scripts/browser-host-check.mjs` compares all four traces through the actual Rust DomRenderer and real canvases in Chromium, Firefox and WebKit.
- `WASM_TEST=lib node scripts/browser-host-check.mjs` includes CharSize and canvas component cases.
- `WASM_TEST=row_factory_browser node scripts/browser-host-check.mjs` replays all retained original row calls with the shared cache interface.

Use the matching wasm-bindgen0.2.100 CLI through WASM_BINDGEN if not on PATH. The browser runner owns server cleanup and writes immutable run directories under results/browser-host. Keep failed runs. Only use --write-fixtures after inspecting upstream drift. No command here establishes complete renderer or terminal equivalence.
