---
name: webgl-atlas-support-check
description: Use when changing atlas palette resolution, Canvas2D page metadata or signed numeric glyph widths before full TextureAtlas integration.
---

# Atlas support checks

Read `docs/webgl-atlas-support.md`. These components are not a complete TextureAtlas or WebglAddon.

Use the unchanged source through `SOURCE_ROOT` and regenerate these fixtures with the existing scripts:

- `scripts/webgl-atlas-color-oracle.mjs`, preserve all 452 cases and seed `0x3acb128f`.
- `scripts/webgl-unicode-number-oracle.mjs`, preserve 112 provider cases. Keep browser `atob` rather than injecting Node Buffer into the VM, because the source trie ignores pooled Buffer offsets.
- `scripts/webgl-atlas-page-oracle.mjs`, preserve five metadata scenarios, including quad, fallback and duplicate source-page glyph aliases.

Run `cargo test -p xterm-web --test webgl_atlas_colors --test unicode_glyph_number -j2`. Check changed core/web/wasm-support crates with warning-denied native all-target/all-feature Clippy. Check the web WASM library and the two focused test targets with warning-denied Clippy.

Use `scripts/browser-host-check.mjs` sequentially with `WASM_TEST=wasm_webgl_atlas_page` and `WASM_TEST=wasm_unicode_glyph_number`. Require all three engines. The retained tests cover page pixels, row/glyph aliases, clear-retained used area, supplied iframe documents, context failure consuming a version, signed Number forwarding, callback receiver/reentry and original throw identity. Read iframe pixels before removing the frame; Chromium may stop painting after removal.

PageClock must be explicitly shared across atlases in the future binding runtime. A page merge constructor copies metadata, not pixels. Full merge/repack/cache/warmup and addon integration still need their own tests.

Use bounded owned O: scratch/targets with separate TEMP/TMP/TMPDIR assignments and the current team grants. Keep compiler/WASM/binding/source hashes with concise receipts. Retain exact full-format command failures rather than replacing them with per-member passes. Delete only owned generated outputs after preserving receipts and checking processes and links. Never traverse reference/dependency junctions during cleanup.
