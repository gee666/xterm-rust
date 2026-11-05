# Inactive image-cell access

Task503 fixes one public browser contract. Full image-addon parity remains open in311/324. Full xterm.js equivalence is NOT PROVEN.

In the unchanged source `ImageAddon.ts`, `getImageAtBufferCell(x, y)` and `extractTileAtBufferCell(x, y)` return the optional `_storage` method call. Before activation `_storage` is absent. Neither coordinate is inspected or converted. Reset and repeated disposal of a never-activated addon do not create storage, so this remains true afterwards.

The old Rust exports accepted `f64`. The JavaScript-to-WASM call converted both arguments before Rust could check storage. Symbols, bigints, revoked proxies and throwing conversion hooks could therefore throw. User conversion callbacks could also mutate or dispose an otherwise inactive addon.

The exported JavaScript methods now accept raw `JsValue` arguments and check for storage before the existing numeric conversion path. The native Rust `get_image_at_buffer_cell(f64, f64)` and `extract_tile_at_buffer_cell(f64, f64)` methods are unchanged. The new Rust binding methods end in `_raw`, while their JavaScript names remain `getImageAtBufferCell` and `extractTileAtBufferCell`. No dependency, unsafe code or JavaScript terminal implementation was added.

This is not full active raw-argument parity. With storage present, the existing binding still converts x then y to numbers. Source active lookup instead evaluates circular-list addition/remainder on y, and conditionally evaluates x multiplication and property-key conversion. Preserving the old backing array and line during conversion-triggered resize, scrolling and buffer switches requires separate work in324. Pixel/palette coercion, writable private image state, GC and complete unchanged image suites/img01..48 also remain open.

Two new WASM functions compare the unchanged source constructor, direct generated WASM class and shipped browser bootstrap. The first uses27 raw forms in both coordinate positions, both methods and missing arguments,110 calls per constructor. The second uses24 calls per constructor across construction, reset and repeated disposal. A conversion hook attempts mutation/reset/disposal and throws undefined; it must never execute. All54 inherited functions and their assertions remain intact. The existing public-bootstrap test still checks actual active QOI pixels, retrieval, tile extraction, events and terminal disposal.

## Reproduction

Use an owned O: worktree with private target, TEMP and TMP. Set `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to the existing matching installations. The tests use Node24.15.0, rustc/cargo1.85.1 and locked wasm-bindgen0.2.100. The accepted495 owned-HTTP-server cleanup helper is a separate prerequisite, not part of503.

```sh
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown \
  --lib --test image_addon_browser --all-features -- -D warnings
node results/image-inactive-coordinates/positive.mjs initial
node results/image-inactive-coordinates/negative-control.mjs
# After finally restores both production hashes, run the full fixed matrix again.
node results/image-inactive-coordinates/positive.mjs final
ORIGINAL_REFERENCE=/path/to/preserved/source_repo \
  node results/image-inactive-coordinates/audit.mjs
```

The negative control replaces only503's two production files with main5088 bytes, preserves all tests, requires the two new functions to fail and all54 inherited functions to pass, and verifies restoration in finally. A build failure, timeout or unrelated assertion failure is not a successful control.

## Observed results

Initial and final matrices pass56/56 functions in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each has zero failures, ignored tests, filtered tests or page errors. The main5088 production control completes54PASS2FAIL in Chromium. Exactly the two new inactive-coordinate functions fail, all54 inherited functions pass, and finally restores both production hashes before the final three-engine run.

Native web all-target/all-feature tests pass400 tests across164 summaries, with zero failures, ignored, measured or filtered tests. Warning-denied native web all-target/all-feature Clippy, WASM library plus image-addon-test all-feature Clippy and the final post-restoration WASM lint all pass. The accepted495 helper's seven Node tests pass independently in this worktree. All five member formatting checks pass. Exact `cargo fmt --all -- --check` fails with Windows OS206, retained in `results/image-inactive-coordinates/workspace-fmt.log`. This is not a workspace formatting pass or waiver.

`audit.mjs` passes706 input-pin checks across the three phases, seven generated/compiled artifact pins, both776-file reference manifests and three actual Cargo test compilations with fresh=false. Negative and final WASM bytes differ. `negative-audit.json` verifies baseline bytes directly against main5088 Git objects and both restored product hashes. These counts include repeated input checks, not706 distinct files. Temporary artifact paths document historical build identity and will be absent after owned cleanup; rerun the commands to reproduce them.

Exact logs, commands, counters, source/comparator pins and restoration receipts are retained under `results/image-inactive-coordinates/`. Full workspace/native-OS quality and complete original suites are not claimed by this scoped task. Independent review and cleanup receipts follow in the handoff.
