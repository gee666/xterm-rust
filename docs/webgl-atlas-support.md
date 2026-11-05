# WebGL atlas support

This is component support for the unfinished TextureAtlas. It does not expose a WebglAddon or implement text/style rasterization, row packing, page merging/eviction, atlas caching/warmup, WebglRenderer, or terminal integration. Those remain task 306. Overall equivalence is NOT PROVEN.

Task 411 recovers `778d9d1d` from `swarm/ArjunaAtlas` onto current main without replaying the accepted pixel hotfix. The recovered implementation contains:

- `webgl/atlas_colors.rs`, source background/foreground resolution, inverse and bold-palette handling, dim opacity, and shared contrast-cache color/null entries. `GlyphPaint` captures attributes before callbacks, while `background_for` and `foreground_for` defer color reads to their respective drawing phases.
- `webgl/atlas_page.rs`, real Canvas2D pages, row and glyph aliases, used-pixel metadata, and clearing. Clear retains historical glyph references and used area, as upstream does. The constructor copies metadata from source pages, not their pixels. `PageClock` must be owned and shared by the future binding runtime, not recreated for each atlas. `StoredGlyph::place` records normalized coordinates consumed by the existing GlyphRenderer.
- Signed `wcwidth_i32` service/provider operations. The unsigned parser path is unchanged. Numeric atlas keys can contain signed combined-cell markers. V6/V11 treat negatives as controls; grapheme providers use their source trie error value. The existing JS provider bridge receives the actual signed Number, preserves the provider receiver and original thrown value, and permits registry reentry. There is no second Unicode registry.

## Tests and reproduction

Use an owned O: target and TEMP/TMP/TMPDIR, Cargo `-j2`, and the current team resource limits. Browser commands use `wasm-bindgen` CLI 0.2.100 and the installed Playwright module through `WASM_BINDGEN` and `PLAYWRIGHT_MODULE`.

```sh
SOURCE_ROOT=/unchanged/reference node scripts/webgl-atlas-color-oracle.mjs
SOURCE_ROOT=/unchanged/reference node scripts/webgl-unicode-number-oracle.mjs
SOURCE_ROOT=/unchanged/reference node scripts/webgl-atlas-page-oracle.mjs
cargo test -p xterm-web --test webgl_atlas_colors --test unicode_glyph_number -j2
cargo clippy -p xterm-core -p xterm-web -p xterm-wasm-support --all-targets --all-features -j2 -- -D warnings
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_webgl_atlas_page --test wasm_unicode_glyph_number -j2 -- -D warnings
WASM_TEST=wasm_webgl_atlas_page node scripts/browser-host-check.mjs
WASM_TEST=wasm_unicode_glyph_number node scripts/browser-host-check.mjs
```

The source is bundled in memory for the oracles. Private source exports are appended only to that virtual entry. Reference files and assertions are unchanged. The Unicode oracle deliberately uses browser `atob`, not Node's pooled `Buffer`, because the source trie loader ignores pooled byte offsets.

The original 452 palette cases and 112 signed-width cases regenerate byte-identically. Palette cases retain seed `0x3acb128f`. Four native test functions cover those cases, palette errors and signed-provider reentry. The recovered browser tests cover signed host getters/errors/reentry, page pixels/metadata/aliases and iframe document adoption. Two additional browser tests check five unchanged-source page metadata scenarios, including quad, non-quad and repeated-source entries, and version consumption when canvas context acquisition fails. There are five page functions and two signed-width functions per browser.

`results/webgl-atlas-support/checkpoint.json` and the earlier logs describe the original unreviewed checkpoint. Run-2 receipts use a `run2-` prefix so the old results are not mislabelled as current evidence. The original iframe-removal failure is retained. Browser test compile diagnostics during test development are not runtime failures and are recorded separately.

The exact workspace `cargo fmt --all -- --check` command hits Windows OS206 after main's test inventory grew, even in the short O: worktree and with Rust 1.85.1 and 1.97.1. All five workspace members are formatted and checked separately. This is not reported as an exact-command pass or a final quality-gate result.

## Integration

Task 411 was independently accepted and integrated together with independently accepted task 413 through the unmodified canonical helper onto main `2b291838`. The helper passed changed-core/wasm-support/web builds and all-target tests, 1,900 passing tests across 386 result summaries, none failed or ignored. Affected dependents' native all-target checks, workspace native checks and applicable WASM library checks passed. This gate uses default features and is not the full all-feature/platform acceptance matrix. The private target was removed by the helper.

The scoped 411 files are byte-identical to the reviewed `a916a857` source; the 413 lifecycle-view files are byte-identical to accepted `043c703a`. The raw gate and source/cleanup preflight receipt are `results/webgl-atlas-support/run2-canonical411-413.log` and `run2-integration.json`. Full atlas/renderer/addon implementation and final equivalence are still unfinished.
