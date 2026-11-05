# IIP warning callback release boundary

Task360 repairs the early IIP failure paths from `source_repo/addons/addon-image/src/IIPHandler.ts`. BASE64 decoding failures, unsupported formats and rejected source dimensions call `console.warn` before releasing decoder payload. The Rust controller previously released payload while constructing its warning effect. A warning getter or callback therefore observed zero loaded bytes, and a synchronous exception lost state that the source retained.

`IipEffect::Warning` now retains the decoder state. A host must deliver it through `IipHandler::report_warning` after ending the controller borrow used by `prepare_end` or `end`. This helper holds no controller borrow while the host runs. It propagates the original callback error unchanged. Successful delivery releases the live decoder, including payload from a transfer started inside the warning getter or callback. An exception leaves that live transfer untouched. There is no generation guard, retry or cancellation.

The existing recording IIP oracle models a successful warning sink and releases payload at that host boundary. Its source observations and assertions remain unchanged. `HostWarning` from an asynchronous bitmap failure is a different path, already reached after successful Blob/ImageData preparation and payload release. It does not acquire the new pre-release warning contract.

## Verification

Run from a checkout with the reference's installed TypeScript/esbuild/Playwright dependencies, supported Rust with the WASM target and wasm-bindgen CLI 0.2.100. Set `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE`, `WASM_BINDGEN` and `SOURCE_REPO` as documented in `docs/image-canvas.md` when dependencies live outside the checkout.

```sh
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
node scripts/image-iip-oracle.mjs
CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_canvas_browser -- -D warnings
cargo fmt --all -- --check
```

Observed on Windows with Rust/Cargo 1.85.1 and Node 24.15.0, based on integrated main `5ca9f593`:

- Before removing the three early releases, the new native regression target failed two tests with loaded-byte observations of 0 instead of 1 and 4. The third reset case passed. `results/iip-warning-release/baseline-early-release.log` records this negative control, not a source suite execution.
- After the repair, all 166 native addon tests pass across 29 summaries, including three new warning tests. They cover all three diagnostic classes, error identity, reset reentry and successful versus throwing multipart reentry.
- The existing source IIP policy oracle passes all 9,925 state observations, retaining seed `0x13372026` and unchanged expected-state and operation hashes. This is recording-host protocol/geometry evidence, not codec pixels or the original full addon suite.
- Native addon all-target/all-feature Clippy and scoped WASM library/canvas Clippy pass with warnings denied. The full workspace formatting command passes from a shorter O: worktree path. The initial longer path hit Windows error 206; that failed attempt is retained separately.
- All 23 canvas browser test functions pass in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, with zero failed, ignored or filtered tests. The new test expands to 27 warning cases per engine, giving 81 directed source/Rust comparisons. These are not 81 original upstream tests. Source-input hashes, compiler/WASM/bindings/CSS hashes and exact browser counts are retained in the canvas verification and browser reports. Independent task360 review accepted commit `e7cf1d5`. The reviewer reran the full 166-test native addon suite, native and scoped WASM warning-denied Clippy, full workspace formatting and the WASM test build. It hash-verified the retained compiler/WASM/bindings/CSS artifacts, bundled the unchanged source afresh and independently reran all 23 tests in each browser, including the 27 warning scenarios per engine. No findings or source changes remained.

The new browser differential runs the unchanged original IIP handler and the Rust controller against the real browser `console.warn` getter and callable property. It compares warning text, original thrown-value identity, getter/call receiver, loaded payload and multipart/abort/parser state. Each of three diagnostic payloads runs nine host behaviours, giving 27 directed comparisons per engine. The behaviours include getter and call exceptions, reset, multipart transfers from either getter or call, and inline-disabled transfer reentry. The existing pixel, QOI memory, bitmap, storage and prior release-boundary tests remain intact.

Full ImageAddon constructor/protocol activation, raw JS options and the original full image/demo suites remain task311 and task324. This repair is not complete addon equivalence or native Linux/macOS evidence.
