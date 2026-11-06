# WebGL canvas layers

The source `BaseRenderLayer` is `webgl::canvas_layer::CanvasLayer`. Its font construction and persistent cell clipping are in `canvas_layer_text.rs`. The source `LinkRenderLayer` is `webgl::link_layer::LinkRenderLayer`. Both use a real browser canvas. They do not render through xterm.js.

`LayerServices` receives the actual native terminal, CoreBrowserService, ThemeService and the terminal runtime's existing `webgl_atlas_cache`. It adapts validated terminal options to `AtlasOptions`, resolves Unicode/log services only on a cache miss, and uses the shared real TextureAtlas. Canvas layers keep the source's maximum texture size of 2048 and default custom glyphs. The device dimensions are captured before reading DPR; live theme handles must be read after that getter, which can reenter the theme service. No theme/pool borrow may span atlas construction, disposal or a host callback.

`CanvasLayer::subscribe_theme` installs a subclass reset after atlas refresh/warmup. The link constructor supplies its own reset. `set_transparency` clones the canvas, initializes the new context, replaces the old node, refreshes the atlas and calls the subclass redraw with the current terminal row count. Unchanged alpha does none of that. A context error preserves the source's partial alpha/canvas mutation. Explicit disposal is idempotent and removes the canvas after unsubscribing. Last-owner Rust drop attempts cleanup and reports errors. Base-layer disposal does not release the terminal from the cache; source WebglRenderer owns that release.

The existing `LinkUnderlineRenderer` capability accepts the current Rust linkifier's typed events. `show` also accepts `Rc<Cell<LinkLayerEvent>>` to retain event identity and observe mutations between drawing and clearing. Hide ignores its incoming event and clears the stored show event. Single-row clearing deliberately clears both the first-row remainder and the last-row prefix. A new show does not first erase the previous link. Resize discards the stored state only after the base resize succeeds. Foreground selection preserves inverted-default, palette-mode lookup errors and ordinary foreground fallback.

Optional focus/cursor/grid/selection hooks whose upstream defaults are empty are not added as successful no-op APIs. A future renderer calls only the capabilities it needs. The source base's text helper leaves its clip in place; callers own save/restore.

## Typed boundary and remaining integration

These are Rust browser components, not arbitrary JavaScript class replacements. Dimensions and terminal options use the repository's typed services. Link foreground is an optional packed unsigned word; native linkifier events have no foreground override. Retained events support typed field mutation, not arbitrary JS getters, objects or coercions. Raw substituted services/options/events and source-hanging extreme numeric inputs are not established by this component suite.

Task306 still owns actual WebglRenderer construction, GPU/frame consumption, layer attachment to renderer replacement, public WebglAddon/events, Root/context restoration and complete unchanged WebGL/custom-glyph/demo suites. Implementing `LinkUnderlineRenderer` does not by itself install the WebGL renderer in Root. The shared cache must come from RuntimeContext, not a new private pool. See `docs/webgl-browser-cache.md` for the accepted cache contract.

## Verification commands

Use an untouched source reference with the repository's existing esbuild and Playwright dependencies. No source assertions are edited. The source wrapper uses the unchanged BaseRenderLayer, LinkRenderLayer, TextureAtlas, OptionsService, ThemeService, UnicodeService/V6 and LogService as the oracle. Rust uses its actual native terminal, CoreBrowserService, theme, runtime cache, atlas and canvas.

```sh
SOURCE_ROOT=/path/to/original node scripts/canvas-layer-oracle.mjs
WASM_TEST=wasm_webgl_canvas_layer WASM_PRELUDE=tmp/canvas-layer/source.js node scripts/browser-host-check.mjs
cargo test -p xterm-web --test webgl --all-features -j2
cargo clippy -p xterm-web --lib --test wasm_webgl_canvas_layer --target wasm32-unknown-unknown --all-features -j2 -- -D warnings
```

Private O: target/TMP/TEMP, bounded processes and the coordinator's heavy-job allocation are required in this run. The existing browser runner owns its server/browser cleanup. The oracle bundles source in memory and records all input hashes; source code executes only on the expected-result path.

The test compares per-engine canvas traces and bitmap hashes for single/multiline links, inverted/fallback/palette-error colors, retained event mutation, theme refresh, actual shared atlas glyph pixels/identity, observable warmup scheduling, resize/state reset, draw/constructor/context failures, draw reentry, opaque clearing, truecolor clipping, font strings, transparency cloning and disposal. A further directed regression exercises theme mutation from the real DPR getter before outer atlas configuration.

`results/canvas-layer/` retains exact reports, logs, failing inputs and restoration hashes. The first ten-case matrix and restored ten-case matrix passed all three engines. The production negative control inverted middle-row clearing, producing4passed/6failed in Chromium, then restored exact bytes in finally. A final eleven-case DPR regression matrix is pending at this documentation checkpoint.

Native Windows focused `webgl` passed46 tests. All-feature native and WASM lib/test Clippy passed with warnings denied. The WASM target compiled with all features; browser execution used default features. Member formatting passed; exact workspace `cargo fmt --all -- --check` failed with Windows OS206 and is not a pass. Full workspace/release/native Linux/macOS, random/raw-host coverage and full addon acceptance are not established here. Full equivalence remains NOT PROVEN.

Run `scripts/canvas-layer-audit.mjs --runtime` before owned runtime cleanup, and its default check afterward. `--record` captures a producer pin snapshot, not independent review. Never remove another owner's outputs or follow reference links during cleanup.
