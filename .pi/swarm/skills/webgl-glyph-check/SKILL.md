---
name: webgl-glyph-check
description: Use when changing WebGL GlyphRenderer vertices, compact uploads, atlas page invalidation or GPU resource ownership.
---

# Glyph component checks

Read current main docs/webgl-glyphs.md and docs/webgl-rectangles.md. This is component coverage only, not a complete WebglAddon or TextureAtlas implementation.

Run scripts/webgl-glyph-oracle.mjs with SOURCE_ROOT pointing at the unchanged supplied reference and dependencies. It executes the original GlyphRenderer with a recording GL provider, records dependency hashes and generates the deterministic fixture. Preserve all assertions and seed0x61ab9423.

Run cargo test -p xterm-web --test webgl_glyphs --test webgl_glyph_backgrounds and scoped warning-denied Clippy. The oracle also generates the separate eight-case signed-background fixture without changing the original147-case fixture. Pass renderer GlyphColorNumbers directly to GPU update_cell_with_colors; packed background equality is not numeric equality. Read docs/webgl-signed-backgrounds.md for the intentionally still-packed atlas contract. Run the existing browser-host-check.mjs with WASM_TEST=wasm_webgl_glyphs for actual GPU tests in Chromium/Firefox/WebKit. Set WASM_BINDGEN to CLI0.2.100 and PLAYWRIGHT_MODULE if dependencies live elsewhere. Recheck wasm_webgl_rectangles when shared gpu_resources changes.

Use the current team's explicit resource limit, never the retired global build queue. For this swarm use owned O: target/scratch, Cargo -j2 and bounded commands. Retain concise logs, source/fixture/build hashes and exact browser counts in results/webgl-glyphs. Remove disposable compiler/browser artifacts only after preserving evidence and confirming no owned process remains. A missing WebGL2 context is not a pass. Full atlas rasterization, addon events/recovery and original WebGL suites remain separate unfinished work.
