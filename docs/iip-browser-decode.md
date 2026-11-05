# IIP browser image decoding

The recovered `image::qoi` codec and `xterm_web::image::IipImageDecoder` turn native IIP display requests into real browser images. The original JavaScript terminal engine is not involved. Original `IIPHandler.ts` runs separately as a test oracle, bundled without source changes.

## Host contract

Keep one `IipImageDecoder` per IIP handler. Call `prepare` synchronously after the native `IipHandler` emits a `DisplayRequest`.

- `Immediate` contains a real canvas for unscaled QOI. Insert it into `IipImageStorage` immediately, without awaiting a future. The source does not yield in this path.
- `Pending` owns the actual `createImageBitmap` promise. Await `resolve`, then insert its bitmap into storage. Both promise rejection and storage insertion failure belong to the source IIP warning path. The decoder does not swallow either error or report successful insertion.
- `prepare` returns QOI, ImageData, Blob construction and synchronous `createImageBitmap` call errors directly. They do not belong to the bitmap promise rejection handler.
- `reset` releases QOI memory according to its retained-memory threshold. It does not cancel bitmap promises. Pending resources survive reset or destruction of the decoder, matching source late completion. Decoder methods take shared references. Private mutable QOI borrows end before every browser call, allowing synchronous ImageData construction to reenter reset or another transfer.

QOI decoding is safe Rust. A browser-owned WebAssembly.Memory mirrors the Rust decoder bytes solely for typed-view identity, retained-buffer mutation and growth detachment. It executes no codec or terminal instructions. Snapshots end native borrows before foreign calls; host-side view mutations copy back before the next native decode. Nested ImageData transfers therefore see the source's shared pixels rather than an outer owned snapshot. The platform decodes PNG/JPEG/GIF/WebP/AVIF with its real Blob/ImageBitmap implementation. The QOI fast path constructs ImageData, releases QOI memory, creates a canvas in the global document and copies pixels only when a 2D context exists, as upstream does. Resized QOI passes ImageData to the same platform bitmap resize path. Storage owns bitmap close behavior after insertion.

## Verification

Run from the worktree with Cargo and the pinned wasm-bindgen 0.2.100 CLI on PATH. External reference dependencies can be reused with `SOURCE_REPO`, `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN`, as documented in `docs/image-canvas.md`.

```sh
cargo test --locked -j2 -p xterm-addons --test qoi
node scripts/qoi-oracle.mjs
CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
cargo clippy --locked -j2 -p xterm-addons --test qoi --example qoi_trace -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown \
  --lib --test image_canvas_browser -- -D warnings
```

The native replay retains all 1,329 installed-WASM codec observations, including seed `0x90a12026`, every opcode, malformed inputs, retained memory and original fixtures. Its six Rust tests also compare the complete original 204,800-byte palette. `results/qoi/summary.json` records the fresh source/native oracle result.

The browser canvas suite now includes IIP transfer tests. They feed incremental header/base64 chunks to native `IipHandler`, decode the resulting request, and compare actual pixels with unchanged source `IIPHandler` execution. They cover all palette pixels on the synchronous QOI path, resized QOI and PNG through real ImageBitmaps, real buffer storage/rendering, pending completion after decoder reset/drop, synchronous failure identity, rejected-promise identity, ImageData failure, real invalid PNG rejection and nested QOI transfers. The nested source/Rust regression checks green-then-green retained pixels at 32x16 and 640x80, and matching detached-buffer errors when a larger nested image grows memory. Existing renderer tests remain intact. The complete suite passes 21 Rust test functions in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. These are Rust test-function counts, not upstream suite counts. The pre-alias-repair native addon all-target checkpoint passed 157 tests across 26 summaries. Scoped native QOI and WASM library/canvas Clippy pass with warnings denied. Reports are under `results/browser-host/image_canvas_browser/`, `results/image-canvas/` and `results/iip-browser-decode/`. The local environment is Windows, rustc/cargo 1.85.1 and Node v24.15.0. These results do not prove native Linux or macOS behavior.

## Still missing

This is not a public `ImageAddon` constructor or complete addon acceptance. Task311 must connect the existing control policy, SIXEL, IIP, Kitty, storage and lifecycle owners to the real browser terminal and addon manager. It must preserve source registration/disposal order, raw JS options and reentry. Current native `IipHandler` copies and releases its base64 payload before emitting `DisplayRequest`; the complete coordinator must revisit that boundary to prove source decoder-release ordering on synchronous host failure. The component tests do not establish that state ordering.

Full original image suites, img01 through img48, all formats at terminal level, late storage insertion after full addon disposal, release/platform/feature gates and broader performance comparisons remain open. Overall image addon equivalence is NOT PROVEN.
