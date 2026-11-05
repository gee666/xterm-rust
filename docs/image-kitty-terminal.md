# Kitty graphics on the browser terminal

Task373 adds `xterm_web::image::BrowserKittyProtocol`. It connects the existing portable Kitty controller to the real streaming APC parser, browser codecs, shared ImageStorage and live cursor state. It is a producer for the unfinished ImageAddon coordinator. It does not export a partial public ImageAddon constructor or run the original JavaScript terminal on the Rust side.

## Ownership and execution

Create ImageRendererLifecycle and ImageStorage for the terminal, then pass their shared instances to BrowserKittyProtocol::new with KittyOptions. Retain the token returned by register. ParserApi::register_streaming_apc_handler bypasses the public whole-string adapter's accumulation limit, removes the unused APC prefix and validates its packed identifier. Disposing the token detaches active and future delivery.

The native KittyGraphicsHandler still owns control parsing, strict BASE64, multipart transfers, image/placement maps, responses and storage deletion policy. The browser adapter adds these operations:

- Capture transmitted bytes into a real Blob before the native decoder releases its payload. Blob.arrayBuffer is called eagerly when display begins. Both synchronous read throws and promise rejections become asynchronous rendering failures, unlike a synchronous Blob-constructor failure during transmission.
- Decode PNG with browser ImageBitmap. RGB/RGBA conversion uses the existing safe Rust raw_pixels policy, then actual ImageData and ImageBitmap. Unknown non-PNG formats select RGB, matching source. Zlib uses actual DecompressionStream, with the source's raw-DEFLATE retry after failure.
- Reuse native geometry policy for crop, cell layout, scaling, offsets, layers, z-index and cursor movement. Browser crop/scale/offset promises supply actual converted bitmap dimensions to the next policy stage. Command records are reread after awaits; no record or terminal-state borrow crosses browser code.
- Transfer bitmap ownership to shared storage only after insertion succeeds. Failed placement closes the current bitmap. Replacement closes the previous bitmap after the next promise resolves. Cleanup preserves source order even when close itself throws. A successful placement applies cursor movement to the buffer object captured before scaling, not a newly selected buffer.

`kitty_host.js` only implements the PNG Image-event fallback used when window.createImageBitmap is absent at entry. These DOM closures stay in the JavaScript heap so abandoned images and their listeners remain garbage-collectible. Protocol decisions, raw conversion, decompression orchestration, geometry, storage and terminal state remain Rust-owned. There is no new dependency or unsafe code.

Reset clears native pending uploads and Kitty maps without cancelling already started browser work. Explicit dispose follows the source coordinator's Kitty storage-then-handler order. Registration disposal is separate. Late placements can complete after reset or disposal of either protocol or ImageStorage. The streaming adapter retains the browser-terminal context through the entire pending handler, including its response callback after the display future finishes. Registrations otherwise retain a weak browser-terminal reference. The lifetime regression verifies that the context disappears after pending completion rather than leaving a parser cycle.

The Rust handler accessor exposes the existing typed live maps and shared command records. This is not a raw JavaScript constructor-options or Map facade. The public coordinator must still select protocols and own reset/render/resize/buffer subscriptions, event/public-property bindings, ordered repeated activation and complete addon disposal.

## Verification

Observed on Windows with rustc/cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, based on main331f0eea:

- All19 new Rust browser test functions pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine has zero failed, ignored or filtered tests and no unexpected page errors. These are directed differentials, not19 unchanged original upstream tests.
- The comparator bundles unchanged source Terminal, KittyGraphicsHandler, KittyImageStorage, ImageRenderer and ImageStorage. It compares full captured cells, widths, attributes, image/tile payloads, wrapped rows, cursor/scroll positions, replies, added callbacks, memory usage, encoded Blob bytes, image metadata and placement-map order. Pixel cases also compare stored canvas bytes and rendered layer bytes. Multipart chunking uses seed0x4b373.
- Cases cover RGB/RGBA/unknown raw formats/PNG, real zlib and constructor-failure raw-DEFLATE fallback, PNG Image-event fallback, before/after-open metrics, crop/scale/offsets, bottom/top layers, scrolling/C=1, transmit/place/replace/delete, multipart uploads with omitted continuation id, cancellation, limits, quiet/errors, actual browser queued writes, host throws, bitmap rejection/cleanup, reentrant reset, retained-command edits during a Blob read, late insertion after protocol/storage disposal and pending terminal lifetime.
- The final regression matrices pass IIP12, SIXEL15, canvas23 and renderer-lifecycle13 tests in each engine. Existing source assertions were not changed.
- Native workspace all-target/all-feature tests pass2043 tests across401 summaries, with zero failed or ignored tests. This includes two new streaming APC registration tests. Native browser-only test crates do not execute WASM assertions.
- Exact full workspace formatting, native workspace all-target/all-feature warning-denied Clippy and scoped WASM library/Kitty/IIP/SIXEL warning-denied Clippy pass. JavaScript syntax and git whitespace checks pass.

A negative control advanced the post-placement cursor by one cell. The Chromium run rejected it, then aborted in fixture cleanup, so there is no trustworthy passed/failed count for that run. `negative-cursor.json` retains the rejected run, not a claimed complete negative matrix. The mutation was restored before final gates and all five final browser matrices.

The first test build used an incorrect native map-iterator method and an ambiguous Promise conversion. Later fixture work incorrectly assumed a fixed microtask count reached the bitmap boundary. The corrected test waits for the actual calls. A real implementation defect then surfaced: retaining the terminal only through placement released it before the native response callback. The adapter now retains it through complete pending-handler resolution, with a regression that drops external terminal/native handles before completion. The final19-test matrix includes that fix.

## Error-diagnostic boundary

The first12-test Chromium run passed every Rust assertion but reported five secondary page errors. Raw-DEFLATE input first rejected the zlib stream's ignored writer promises, and the default source InputHandler logging path also observes a rejecting response callback. The source explicitly does not await writer.write/close. This port preserves those calls rather than silently swallowing their promises.

The green suite uses supported logLevel=off on both terminals for handler-rejection tests. Its raw-DEFLATE case exercises the explicit retry through a synchronous DecompressionStream constructor failure, then performs actual browser raw decompression. It does not claim that malformed/default-zlib writer rejections are free of secondary errors. No page-error filtering or source/assertion changes were added. The initial diagnostic is retained separately as `initial-source-error-diagnostic.json`; broad unhandled-rejection timing remains unproven.

## Reproduce

Use the reference/tool environment described in docs/image-canvas.md. No original source modifications are needed.

```sh
cargo fmt --all -- --check
cargo test --locked -j2 --workspace --all-targets --all-features
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_kitty_terminal_browser --test image_iip_terminal_browser --test image_sixel_terminal_browser -- -D warnings
WASM_TEST=image_kitty_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_iip_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_sixel_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_canvas_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_lifecycle_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
```

Concise reports, input/artifact hashes and native logs are under results/image-kitty-terminal. Generated browser binaries and run directories are disposable. Full public ImageAddon/coordinator/raw options, all original image suites and img01..48, broader patched-host/malformed-input equivalence, final cross-platform matrices and overall equivalence remain NOT PROVEN under tasks311/324/327/304.
