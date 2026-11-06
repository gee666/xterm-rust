# Typed browser frame composition

`webgl::browser_frame::BrowserFrame` connects the existing Rust `RendererState` to the real rectangle and glyph GPU renderers, actual `TextureAtlas`, binding-owned `BrowserAtlasCache`, canvas context and link canvas. It does not implement the public WebglAddon or replace Root's renderer yet.

## Contract

The caller supplies `LayerServices` from the live terminal and its existing runtime cache, the screen, measured render dimensions and context attributes. Construction creates the WebGL2 context before creating the link canvas or subscribing to its theme events. It attaches the GPU canvas after the link canvas, creates the real GPU resources, resizes both renderers and acquires the atlas through the existing terminal identity. Atlas limits belong to the binding cache's existing runtime.

A `CellSource` supplies terminal cells, joins, color resolution, cursor colors and blink visibility. The frame owner handles glyph geometry, signed numeric color routing, rasterization, atlas uploads, incremental model updates, backgrounds and cursor drawing itself. The enclosing renderer must provide dimensions and `UpdateContext` at their upstream call sites. Detached rendering waits for a connected screen with nonzero character measurements. An explicit source error is propagated without stringifying its thrown identity, retaining mutations performed before that error.

This API uses an exclusive Rust borrow. It does not claim raw JavaScript getter ordering or recursive access to the same frame owner. Source callbacks can mutate their terminal dependencies. They cannot borrow the frame recursively through this contract. A future raw browser adapter must preserve those source phases and partial state instead of treating the typed context as proof of raw equivalence.

`resize` resizes the model and link canvas, sets physical and CSS canvas sizes, sets the screen CSS size, resizes rectangle and glyph geometry, refreshes the actual atlas, clears the model and requests a synchronous full redraw. Unlike observer-driven physical sizing, assigning the same canvas size still clears its backing store. Color changes refresh the atlas and clear glyph/model state. Clearing the atlas requests a redraw.

`restore` releases the terminal's cache owner before replacing the GPUs and reacquiring/rasterizing the atlas. Its resize requests a synchronous redraw. The enclosing `ContextLifecycle` then makes its normal restoration redraw. The enclosing renderer must cancel context, cursor and text timers before `dispose`. Frame disposal releases GPU resources and forwarding subscriptions, disposes the link layer, detaches the canvas and releases the terminal's shared atlas ownership. Other terminals retain their atlas. Explicit cleanup errors propagate and Drop does not retry a failed explicit cleanup.

## Verification scope

`wasm_webgl_frame` compares actual model words, line lengths, cursor state, atlas page counts and full RGBA framebuffer hashes against the unchanged source WebglRenderer constructed with the original browser Terminal and real GPU renderers. The only injected source service is a fixed 8 by 16 character measurement, matching the Rust dimensions. ASCII updates, ANSI colors, wide/combined UTF-16 text, custom blocks, cursor visibility, blinking cells, clearing, restoration and resize are separate scenarios. Rust-only checks cover attachment, source error identity, shared-terminal cache release and disposal. Equal-dimension resize replaces the atlas because the source link layer acquires with max texture size 2048 before the GPU renderer reacquires with the real device limit. A source assertion verifies that identity change. Restoration tests compose the actual ContextLifecycle around the Rust frame, including the final normal redraw after GPU reconstruction's synchronous resize redraw.

The oracle wrapper is `scripts/support/webgl-frame-source.ts`. `scripts/webgl-frame-oracle.mjs` bundles unchanged source in memory and records each dependency hash separately from the test wrapper. The JavaScript source engine runs only as the oracle. Rust uses its own terminal parser, atlas, model and GPU drawing.

Run with the existing browser-host checker and an explicit resource allocation:

```sh
SOURCE_ROOT=/path/to/supplied/source_repo node scripts/webgl-frame-oracle.mjs
WASM_TEST=wasm_webgl_frame WASM_PRELUDE=tmp/webgl-frame/source.js node scripts/browser-host-check.mjs
```

Use the locked wasm-bindgen CLI 0.2.100 and the same source's Playwright module. Build targets, temporary bundle and generated browser artifacts must stay in the owned O: worktree. Observed Windows results are under `results/webgl-frame`. Corrected and exact-restored final matrices passed all 11 functions in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, with no ignored/filtered tests or page errors. Nine source scenarios execute 16 per-step full framebuffer/model comparisons, plus two Rust lifecycle groups. The production glyph-clear negative control produced 2 passes and 9 ordinary assertion failures in each engine. Exact source restoration was followed by another 11 passes in each engine.

Native existing WebGL/model suites passed 48 tests. Warning-denied native library, all-feature WASM library and all-feature new WASM test Clippy passed. The xterm-web formatting check passed. Full workspace formatting failed with Windows OS206, retained in `fmt.log`, so this is not a full quality-gate claim. Initial browser matrices passed eight and failed two functions in each engine. The saved initial test incorrectly expected atlas reuse and omitted the enclosing context's restoration redraw. Both failures are retained; the source-backed replacement assertion and real ContextLifecycle composition fix the test inputs rather than hiding failed behavior. No original source assertions changed.

The first two 11-pass matrices still emitted two WebKit context-quota console errors and are not clean proof. They are saved as `browser-quota-corrected.json` and `browser-quota-final.json`, with exact test and source-wrapper reconstructions checked against their original SHA pins. Final fixture cleanup waits for the actual `WEBGL_lose_context` lost event and one real animation frame after event dispatch. This uses no sleep, polling or retry. Both the clean corrected and exact-restored final 11-pass matrices have zero console errors, page errors or context-quota messages. This cleanup repair does not establish a cause or fix for context540's earlier burst/GC counterexample.

Remaining task306 work includes the complete live browser renderer constructor/service subscriptions, raw service/reentry adapter, Root renderer replacement, WebglAddon exports/events and unchanged complete WebGL/custom-glyph/demo suites. Initial context540 burst/GC failures remain unclosed. No overall equivalence claim follows from this component.
