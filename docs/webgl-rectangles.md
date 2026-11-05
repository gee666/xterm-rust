# WebGL rectangle renderer

This is component coverage, not a working WebglAddon. Glyph rasterization, atlas packing/upload, glyph drawing, render layers, root renderer replacement and public addon bindings remain unfinished. Swarm task 306 tracks that work. The demo must still report WebGL unavailable until the actual addon exists.

## Source and Rust contracts

`source_repo/addons/addon-webgl/src/RectangleRenderer.ts` maps to:

- `crates/xterm-web/src/webgl/rectangles.rs`, portable model-to-vertex preparation.
- `crates/xterm-web/src/webgl/rectangle_gpu.rs`, actual WebGL2 instanced drawing and theme event subscription.
- `crates/xterm-web/src/webgl/gpu_resources.rs`, private shader/program/buffer/vertex-array ownership.

`RectangleGeometry` accepts existing `RenderModel`, `RenderDimensions` and shared `ThemeService` colors. Each rectangle contains eight f32 values, normalized x/y/width/height followed by RGBA. Background runs preserve the source's packed attribute comparisons, including inverse foreground transitions and non-color flag transitions. The viewport retains theme alpha. Non-default runs force alpha to one. Palette colors remain live; viewport and cursor colors are cached until a theme event. Cursor support includes bar, underline and outline. Block glyphs belong to the missing glyph renderer, so block and absent cursors correctly produce no rectangle instances.

`set_dimensions` only replaces geometry. `handle_resize` updates the viewport rectangle. A theme event updates cached colors and the viewport without silently recomputing old background runs or cursor vertices. Call `update_backgrounds` and `update_cursor` before drawing new model state. Direct portable users must forward color changes through `colors_changed`; the browser renderer subscribes automatically.

The caller owns canvas dimensions, GL viewport and blending, just as upstream `WebglRenderer` does. The rectangle renderer owns its program, three buffers and vertex array. It uses the original shaders, triangle-strip indices, attribute layout and projection matrix. No JS terminal engine executes these draws. WASM uploads copy into JS typed arrays rather than using unsafe WASM-memory views. Drop and explicit disposal delete resources and unregister the theme listener. Shader and link failures include the driver log in a returned JS Error; partial construction releases allocated resources. The Rust API returns an error on rendering after disposal or context loss. Full context-loss notification and recovery belong to task 306.

Native boundaries validate short packed models, missing palette entries and overflowing dimensions rather than panicking on untrusted indices. These errors are Rust representation choices, not a claim of equivalent behavior for arbitrarily mutated JS internals. Normal terminal models preserve the source behavior.

## Reproduce

From the worktree root, with the repository Rust toolchain and original reference dependencies installed:

```sh
SOURCE_ROOT=/path/to/original/source_repo node scripts/webgl-rectangle-oracle.mjs
cargo test --locked -j2 -p xterm-web --test webgl_rectangles
cargo clippy --locked -j2 -p xterm-web --test webgl_rectangles -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_webgl_rectangles -- -D warnings
WASM_TEST=wasm_webgl_rectangles node scripts/browser-host-check.mjs
```

The existing browser runner accepts `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` when tools live elsewhere. The CLI version must match locked wasm-bindgen 0.2.100. Set `CARGO_TARGET_DIR`, `TEMP` and `TMP` to owned O: temporary directories for this swarm run. Use the run's existing four-job/two-browser limits; no new queue is needed.

The source oracle bundles the unchanged RectangleRenderer and its original runtime dependencies, recording SHA-256 for each input. A GL recorder observes the source's actual uploaded Float32Arrays and draw counts. The generated fixture contains 141 cases: eight directed color cases, five cursor shapes and 128 randomized models using xorshift seed `0x57e61a23`. Rust compares every backing-array f32 bit, including unused entries, growth capacity and active counts. The one-line JSON is generated test data, intentionally not hand-formatted.

Native tests also cover construction versus resize, color/size invalidation ordering, malformed models and palette failures. Browser tests assert actual readPixels results, resize clipping, transparency, cursor edges, theme events, explicit/drop cleanup and partial allocation/shader failures. They require a real WebGL2 context; an unavailable context is a failed/unavailable check, never a skipped pass. Upstream's own WebGL integration suites explicitly skip Firefox and WebKit; extra component attempts do not change those original skips.

## Evidence

Native source differential: 141/141 cases pass within one Rust test. All three native tests pass. All four real GPU tests pass in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 on Windows. The two focused native/WASM Clippy commands pass with warnings denied. Durable commands, hashes and browser output are under `results/webgl-rectangles/`. No full addon, original WebGL integration suite or cross-platform equivalence result is claimed.
