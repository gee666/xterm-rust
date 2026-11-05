# WebGL signed contrast-cache keys

Task 482 follows accepted 462 and 471. It repairs an integer-color contract in the actual Rust `TextureAtlas`, not the remaining `WebglAddon` or terminal renderer binding. Full equivalence is NOT PROVEN.

## Source contract and repair

`source_repo/addons/addon-webgl/src/TextureAtlas.ts` passes the original `bg` and `fg` arguments from `_drawToCache` to `_getForegroundColor` and `_getMinimumContrastColor`. Attribute bit operations use their low 32 bits, but contrast-cache get/set calls use the original numbers. For example, `-2147483648` and `2147483648` have identical attribute bits and distinct cache keys. The dim attribute selects the half-contrast cache without changing those keys. A cached `null` suppresses contrast recomputation, whereas an absent entry triggers it. A cached color returns the same object, including for dim text.

The existing `ColorContrastCache` already preserves numeric identity. The defect was earlier packing in the WebGL caller. `TextureAtlas::get_glyph_with_colors` now sends all original integer colors to `Rasterizer::draw`. The rasterizer derives packed drawing attributes and retains the original background/foreground keys in `AtlasColors::paint_with_colors`. Both full and half contrast-cache lookups and inserts use those keys. Existing unsigned entry points remain available. No dependency, unsafe operation, global state, or secondary color cache was added.

This contract covers the source renderer's integer words, from signed i32 through unsigned u32. Those integers convert exactly to the existing cache's f64 keys. It does not claim arbitrary JavaScript objects, fractions, NaN, infinities, raw mutable browser color-cache facades, or unrestricted i64 equivalence. The attributes passed to `resolve_with_colors` and `paint_with_colors` are the packed drawing state for the original integer keys.

## Regression checks

- `scripts/webgl-atlas-color-oracle.mjs` bundles the unchanged source into memory. All 452 original cases and their assertions remain an exact prefix. The 24 additions cover signed/unsigned background and foreground combinations, full/half caches, seeded color identity, cached null, and new cache inserts. `webgl_atlas_colors.rs` compares values, object identity and cache presence against this corpus.
- The existing in-memory TextureAtlas browser probe adds `signedContrast` and `signedHalfContrast`. Each draws ten numeric or combined-character glyphs using distinct signed/unsigned seed colors, a null entry and cache misses. The existing Rust comparison still checks all aliases, glyph metadata, packing rows, page versions, pixels, events, layout changes and DOM attachment. Eighteen existing atlas tests and their assertions remain intact. Two additions exercise direct atlas calls. A third uses actual `GlyphRenderer` GPU updates and draws, verifies each original key was installed before any direct atlas lookup, and compares all glyph metadata and the final source page's RGBA hash. These make twenty-one tests.
- `results/webgl-contrast-cache/negative-control.mjs` changes real production code, not a fake test adapter. The native mode restores prematurely packed cache keys. Browser mode tests both packed key storage and the rasterizer's old unsigned forwarding call. Every mutation has exact-byte restoration in `finally` and retains its source hash and failure receipt.

Final native focused checks passed eight tests across four executables, including the final 476-case color corpus. The packed-key native mutant produced one pass and one failure, followed by exact source restoration and eight passing focused tests. The new cache-miss cases use contrast ratio 21 to cover inserted colors rather than only inserted null. Warning-denied all-feature Clippy passed for the final native library/focused tests and WASM library/atlas/GPU tests, including the newly imported actual-GPU contrast test. Final browser runs rebuilt both WASM suites. `cargo fmt -p xterm-web -- --check` passed. Exact `cargo fmt --all -- --check` failed with Windows OS206, as retained in `fmt-all.log`; the crate-only pass does not replace that required workspace gate.

Both real production browser mutants produced exactly 18 passes and three failures in Chromium. Only the new full-cache, half-cache and actual-GPU contrast assertions failed. Each mutation restored the exact original production bytes. The first run reached those expected counts but its driver looked for failure names in console logs instead of the harness output and aborted after restoration. `parser-diagnostic.json` and `packed-contrast-keys-first.json` retain that result. The corrected parser validates the three failing test names in the actual harness output; both mutations and all final suites were then rerun.

The restored final atlas suite passed 21 tests and the unchanged glyph-GPU suite passed eight tests in each of Chromium, Firefox and WebKit, with zero failures, ignored or filtered tests. `final-atlas.json`, `final-gpu.json` and `negative-control.json` retain engine versions, commands, counts and artifacts. `checkpoint.json` ties the final production/tests/fixtures to those receipts and the actual browser WASM, bindings and source prelude. Compiler paths reused during mutations describe their recorded stage, not retained historical binaries. Only final artifact pins are audited against files before cleanup.

No Linux, macOS, full workspace quality, release, public WebGL-addon, full unchanged WebGL/custom-glyph or demo result is claimed here.

## Reproduction and retained evidence

Use the existing workspace toolchain and original source Node dependencies. Set `SOURCE_ROOT` to the unchanged reference and use an owned O: target/TEMP/TMP, Cargo `-j2`, and the team's compiler/browser allocation limits.

```sh
node scripts/webgl-atlas-color-oracle.mjs
node scripts/webgl-texture-atlas-oracle.mjs
cargo test --locked -j2 -p xterm-web --test webgl_atlas_colors --test webgl_glyph_backgrounds --test webgl_renderer --test webgl_glyphs
node results/webgl-contrast-cache/negative-control.mjs --native-only
```

For browser checks set `WASM_BINDGEN` to CLI 0.2.100, `PLAYWRIGHT_MODULE` to the installed Playwright module, and `WASM_PRELUDE` to the generated `tmp/webgl-texture-atlas/source.js`. Then run `node results/webgl-contrast-cache/negative-control.mjs` under a compiler/browser grant. Its final reruns cover Chromium, Firefox and WebKit. `node results/webgl-contrast-cache/audit.mjs` verifies 776 original source hashes, the unchanged case prefix, retained source/evidence pins and mutant restoration. Before temporary artifact cleanup, add `--browser-artifacts --native-artifacts` to verify the seven actual WASM/binding/prelude artifacts and four focused native executables. Their temporary paths are intentionally absent after cleanup; the default audit checks durable source and evidence.

Task 306 still owns the actual browser update host, renderer construction and layers, shared atlas cache/runtime/limits/clock/timers, live getters/reentry/callback phases, WebglAddon exposure/events, Root context restoration and full unchanged suites. Existing components and these scoped tests do not close that task.
