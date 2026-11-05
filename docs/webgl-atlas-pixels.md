# Atlas pixel processing

This is the pixel-filtering and crop phase of TextureAtlas, not a complete atlas or WebglAddon. Task 306 still requires font/style/custom-glyph drawing, packing, page management, shared cache/limits, WebglRenderer layers, root replacement and public addon lifecycle. Overall equivalence remains NOT PROVEN. Original WebGL tests that silently fall back to the DOM renderer do not prove a working WebGL addon.

## Source mapping

`source_repo/addons/addon-webgl/src/TextureAtlas.ts` maps as follows:

- `clearColor` to `webgl/atlas_pixels.rs::clear_background`.
- `checkCompletelyTransparent` to `completely_transparent`.
- `_findGlyphBoundingBox` to `find_glyph_bounds`.
- The post-rasterization ImageData phase to `webgl/atlas_canvas.rs::prepare_image` and `prepare_canvas`.

These paths are under `crates/xterm-web/src/`. The pixel code is portable Rust and adds no dependency. Canvas access is WASM-only. The existing `renderer::utils::js_round` is shared through crate visibility, preserving ties toward positive infinity and large integral inputs without duplicating numeric policy.

The source's color-empty result is deliberately not alpha emptiness. A non-background pixel whose alpha is already zero still makes `clearColor` return false. Exact RGB matches clear alpha even if foreground and background alpha differ. Near matches clear only when their channel-distance sum is strictly less than the contrast-relative threshold, the foreground/background RGB distance divided by twelve and floored. Neither routine changes RGB bytes.

The four bounding scans are not a common minimum rectangle. Top and bottom scan the allowed width; left includes right padding; right stops at the padding boundary. The all-transparent fallback uses inclusive width/height plus one. Padding-only data can produce a zero or negative source width, so the result retains numeric extents rather than clamping them to unsigned dimensions. Restricted glyphs scan cell dimensions; restricted/custom glyph offsets apply source centering and line-height rules.

## Typed boundary and ownership

RGBA slices require complete pixels. Bounds require an exact canvas byte count, checked multiplication, finite metrics and a scan inside the image. Restricted physical cell dimensions must be nonnegative integers. Invalid inputs return `PixelError` before pixel indexing. These are Rust typed-boundary contracts, not a claim about arbitrarily mutated JS ImageData objects or fractional property indices.

The portable clear routine mutates its supplied slice. `prepare_image` borrows a real ImageData, processes an owned byte copy and returns a new ImageData with the filtered pixels. It does not mutate the caller's input object. `prepare_canvas` reads actual pixels through the supplied context and does not write to that canvas. A full atlas caller receives the entire image plus crop/offset metadata and must copy only the dirty crop during packing. Empty glyphs return None before bounds work, as source does. Browser canvas/security/read failures propagate as their original JS values.

Copying browser bytes into a JS-owned Uint8ClampedArray avoids retained views into WASM memory. ImageData retains the array supplied to its constructor, so the borrowed Rust-slice overload is not suitable for a returned bitmap. A regression asserts the buffer is not WASM memory and remains intact after actual memory growth. The new owned ImageData is a Rust representation choice for this internal phase. Public/raw ImageData aliasing, getter/reentry behavior and the full rasterizer remain outside this component and must not be inferred from its pixel results. No source engine or font rasterizer is delegated back to JavaScript.

## Verification

```sh
SOURCE_ROOT=/path/to/unchanged/source_repo node scripts/webgl-atlas-pixel-oracle.mjs
cargo test --locked -j2 -p xterm-web --test webgl_atlas_pixels
cargo clippy --locked -j2 -p xterm-web --test webgl_atlas_pixels -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_webgl_atlas_pixels -- -D warnings
WASM_TEST=wasm_webgl_atlas_pixels node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

Use wasm-bindgen CLI 0.2.100 and the existing Playwright installation. The browser runner accepts `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` paths. All build and scratch paths for this run are owned O: directories, with Cargo -j2 and the agreed team resource/browser caps. No new build queue is used.

The generator reads the unchanged original TextureAtlas bytes. A virtual entry appends exports for two private functions; the original file and assertions are never edited. It records every bundled dependency hash and the exact export wrapper. The original private bounding method executes through its actual prototype, supplied only the dimensions it reads.

The one-line fixture is generated test data. There are 527 cases, including 512 randomized images using xorshift seed `0x671ba92d`. Directed cases preserve strict threshold boundaries, transparent nonmatching colors, asymmetric/padding-only bounds, source empty fallbacks, restricted/custom metrics and a large-integer Math.round trap. Rust compares all output bytes, empty flags, bounds, sizes and offset f64 bits before and after clearing. Browser tests construct actual ImageData for both transparency modes and exercise real CanvasRenderingContext2D pixels. The zero-size pure-data case is tested separately as an expected ImageData constructor error, not silently omitted.

All four native tests pass, including all 527 original cases and index-sized zero-area geometry without scanning unbacked rows/columns. The three browser tests pass in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 on Windows. Each engine exercises all 527 fixture cases, including the expected zero-size ImageData constructor error. Both transparency modes run for every positive-size image. Native/WASM focused Clippy pass with warnings denied; full workspace formatting passes.

The first browser run passed two functions and failed the fixture function in each engine, exposing a real retained-WASM-view ownership bug. The correction uses the JS-array ImageData constructor and JS-owned storage, without weakening byte assertions. Original failure receipts, the exact case and corrected outputs are preserved. Evidence belongs in `results/webgl-atlas-pixels/`. This does not establish a complete original WebGL/custom-glyph suite pass, working public addon or native Linux/macOS runtime result.
