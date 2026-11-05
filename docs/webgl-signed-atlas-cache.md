# Signed WebGL atlas keys

Task 471 preserves the integer color arguments emitted by the accepted renderer engine before a TextureAtlas cache lookup. It also passes those same numbers from the actual GlyphRenderer into the actual atlas. Task 462 had fixed overlap clipping but deliberately left this cache identity problem open.

## Contract

The unchanged TextureAtlas calls FourKeyMap with the original bg, fg and ext numbers. For example, 2147483648 and -2147483648 have identical low 32 bits but different numeric property names. They must create different cached glyph objects and occupy different atlas positions. Repeated calls with the same original number reuse the first alias even when height restriction changes.

`TextureAtlas::get_glyph_with_colors` retains `[i64; 3]` in the glyph-cache key. Only a miss converts the words to u32 for the existing rasterizer's bitwise attributes. The `get_glyph` convenience API delegates with unsigned values, so existing callers and ASCII warmup keep their prior behavior. The integer representation matches `GlyphColorNumbers` from the typed incremental renderer. It is not a raw JavaScript binding and does not claim arbitrary object, fraction, NaN or infinity behavior. Values outside the source renderer's integer color-word domain do not establish JavaScript numeric equivalence.

`GlyphAtlas::rasterize_with_colors` lets numeric-cache owners receive the original words. Its default calls the existing packed API for atlases without such a cache. TextureAtlas overrides it. The real GPU `update_cell_with_colors` uses it before clipping or vertex writes. The old packed update/rasterize entry points remain available and delegate. No dependency, unsafe code or new terminal authority was added.

This change concerns the outer glyph cache, not ColorContrastCache's raw numeric identity. The rasterizer's color/contrast implementation remains unchanged. Task 306 retains that distinction, a live browser UpdateHost, arbitrary host/getter/reentry semantics, renderer construction/layers/runtime installation, WebglAddon/Root lifecycle and full original suites. Its browser host must pass original numbers directly, not reconstruct signedness from packed bits. Overall equivalence remains NOT PROVEN.

## Tests and observed results

`crates/xterm-web/tests/reference/webgl-texture-atlas.mjs` adds two scenarios to the in-memory bundle of the unchanged TextureAtlas source. The numeric and combined-character scenarios each make 20 glyph requests plus one clear. They distinguish signed and unsigned bg, fg and ext keys, repeated calls with changed restriction, signed-zero aliases and post-clear reuse. Existing scenarios and assertions remain intact. The Rust replay uses the numeric-key entry point and retains the exact comparisons of aliases, all glyph metadata, layout versions, events, packing rows, page versions, full-page RGBA hashes and alpha counts.

A separate actual-GPU integration test sends six original-number keys through GlyphRenderer into a real TextureAtlas. Each first key must already have packed one new glyph when update returns, each repeated key must reuse it, and a direct lookup must find the same cached object without another draw. Existing GPU pixels/clear/rebuild assertions remain in their original test.

On Windows with rustc/cargo 1.85.1, initial atlas tests passed 18 functions in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. The final restored run again passed all 18 atlas functions plus all eight existing glyph-GPU functions in each engine. All report zero failures, ignored or filtered tests. These counts are test functions, not individual snapshots or assertions.

`results/webgl-atlas-signed-cache/negative-control.mjs` mutates actual production code under a bounded browser/compiler grant. Packing colors before cache lookup produced 15 passes and three intended failures in Chromium. Replacing the GPU's new call with its old packed call produced 17 passes and the single forwarding failure. Each mutation restores the exact source hash in finally before the final passing matrices. The mutant sources, browser receipts, logs and restore hashes are retained alongside final reports.

Focused native glyph/background/renderer tests passed six functions. Native xterm-web all-target/all-feature tests passed 371 functions across 152 summaries, with zero failures, ignored or filtered tests. Strict native library and WASM library/atlas/glyph-test Clippy passed. Both focused WASM test artifacts compiled. Member formatting passed. Exact `cargo fmt --all -- --check` still failed with Windows OS206 and is retained as a failure, not replaced by the member result. Full workspace/native OS/release/public-addon verification is not claimed.

## Reproduce

Read `.pi/swarm/skills/webgl-texture-atlas-check/SKILL.md` for resource limits and reused tool paths. Use a private regular O: target/tmp, Cargo -j2, and an explicit Harry309 compiler/browser allocation.

1. Set SOURCE_ROOT to the unchanged supplied reference. Run `node scripts/webgl-texture-atlas-oracle.mjs`.
2. Set WASM_PRELUDE to the resulting absolute `tmp/webgl-texture-atlas/source.js`, WASM_TEST=wasm_webgl_texture_atlas, WASM_BINDGEN to CLI 0.2.100 and PLAYWRIGHT_MODULE to the existing Playwright installation. Run `node scripts/browser-host-check.mjs`.
3. Under the same allocation, run `node results/webgl-atlas-signed-cache/negative-control.mjs`. It runs both real mutants and the restored atlas/glyph matrices with bounded child processes and exact source restoration.
4. Run `cargo test --locked -j2 -p xterm-web --all-targets --all-features` and the scoped Clippy commands recorded in the logs. Keep the full fmt failure visible.
5. Set SOURCE_ROOT and run `node results/webgl-atlas-signed-cache/audit.mjs` before deleting disposable compiler/binding artifacts. The audit verifies all 776 original reference hashes, accepted462 pins, current source/evidence pins and actual browser artifact hashes. The checkpoint retains those hashes after cleanup; deleted binaries cannot be rechecked without rebuilding.

The private prerequisite replays are not integration commits. The branch starts at main32bf9544, privately replays original449 and462 chains, then adds only task471's own changes. Harry309 owns canonical integration after accepted462. Do not replay the private prerequisite IDs into main.
