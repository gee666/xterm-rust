# IIP payload release boundary

The IIP handler now retains decoded BASE64 state when it emits a `DisplayRequest`. Request bytes are owned, but emitting the request is not successful host preparation. The original `IIPHandler.ts` retains its payload if QOI decoding, ImageData construction or Blob construction throws synchronously.

A live browser coordinator must call `IipImageDecoder::prepare_transfer` with the actual `RefCell<IipHandler>`. The adapter holds no protocol borrow across browser calls. It releases the live payload after QOI/ImageData preparation or Blob construction, before creating the unscaled canvas or calling `createImageBitmap`. A synchronous exception before release keeps the current payload. An exception after release does not restore it. If an ImageData callback starts a multipart transfer, successful outer preparation releases that new live payload too, matching the source. This is deliberately not a generation-guarded release. The returned pending bitmap owns only its browser promise, not the protocol controller or a borrow, so later completion can still outlive reset or handler disposal.

`IipImageDecoder::prepare` remains a standalone operation for owned requests without an associated protocol controller. It cannot update protocol state. The full ImageAddon coordinator must use `prepare_transfer`, not the standalone method. Non-browser hosts can call `IipHandler::release_payload` at their corresponding successful preparation boundary. The recording IIP oracle does so when its successful Blob host consumes a display request.

## Verification

The new browser regression compares unchanged original `IIPHandler` execution against native IIP parsing and the real Rust browser decoder. It observes loaded BASE64 bytes at ImageData, Blob, canvas context and bitmap calls; checks exact thrown-value identity; and compares payload state, multipart flags and header-parser state after normal and nested multipart transfers. Eight directed cases run in each browser. Existing pixel, retained QOI memory, resize, bitmap and storage tests remain intact. The real QOI/PNG pixel tests now use `prepare_transfer`, verify release against source state and let pending bitmaps finish after both protocol and QOI decoders are reset and dropped.

The native `iip_payload` tests check retained state after dropping a failed host request and release of a nested live payload. Existing metadata tests still require zero loaded bytes after the host release, and now also assert retained bytes before that release. The original protocol oracle still compares all existing state snapshots after its successful recording host.

Observed on Windows with Rust/Cargo 1.85.1 and Node 24.15.0, atop integrated main `3baa698`:

- Restoring the previous early-release line causes both new native payload tests to fail with zero loaded bytes instead of the source-required 44. The line was restored to the fixed implementation before final checks. `baseline-early-release.log` records this controlled baseline, not a second upstream suite run.
- The complete native addon all-target/all-feature suite passes 163 tests across 28 summaries. The focused IIP/payload/QOI run passes 18 tests.
- The existing IIP policy oracle passes all 9,925 state observations with seed `0x13372026`. These use recording image hosts and do not prove pixels.
- All 22 canvas test functions pass in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, with zero failed, ignored or filtered tests. This is 66 Rust test-function executions, including the eight new source/Rust boundary cases per engine, not 66 original upstream tests.
- Native addon all-target/all-feature and scoped WASM library/canvas Clippy pass with warnings denied. Full `cargo fmt --all -- --check` passes. `full-fmt.log` retains the earlier failure on the inherited marker module order; `full-fmt-current.log` records the pass after rebasing onto the integrated fix.

Independent task352 review accepted commits `ba4454e` and `94fc7a5`. The reviewer reran all 18 focused native tests, full workspace formatting and scoped native/WASM Clippy with warnings denied. It verified 32 Rust/build-input hashes, 15 unchanged source-input hashes, and retained compiler/WASM/bindings/CSS artifacts for the 22-per-browser reports. Browser execution was not repeated under the compiler-only review allocation. No findings remained.

Run from a checkout with the reference dependencies available. `scripts/image-iip-oracle.mjs` now accepts `REFERENCE_NODE_MODULES` without requiring a dependency junction in the source copy. The browser runner also accepts `SOURCE_REPO`, `PLAYWRIGHT_MODULE` and the pinned 0.2.100 `WASM_BINDGEN` CLI as described in `docs/image-canvas.md`.

```sh
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
node scripts/image-iip-oracle.mjs
CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_canvas_browser -- -D warnings
cargo fmt --all -- --check
```

Commands and observed results are retained under `results/iip-payload-release/`, `results/image-iip/`, `results/image-canvas/` and `results/browser-host/image_canvas_browser/`. Only concise reports, source hashes and logs are retained; generated browser binaries are disposable. Full ImageAddon activation and original image suite/demo acceptance remain task311 and task324. This focused repair does not prove full addon equivalence, raw JS option/getter ordering, decoder warning callback ordering, or native Linux/macOS behavior.
