# Active image coordinate evaluation

Task515 repairs active `ImageAddon.getImageAtBufferCell` and `extractTileAtBufferCell` coordinate evaluation. Full image-addon scope remains open in311/324. Overall equivalence is NOT PROVEN.

## Source contract

The unchanged `addons/addon-image/src/ImageStorage.ts` selects the active buffer, calls `buffer.lines.get(y)`, and inspects x only if that returns a line. `src/common/CircularList.ts` evaluates `this._array[this._getCyclicIndex(index)]`, where `_getCyclicIndex` returns `(this._startIndex + index) % this._maxLength`.

The array and start precede y conversion. Addition uses JavaScript's default primitive hint, so a string can concatenate rather than add. Capacity is read after conversion. Resize can replace the backing array during conversion, but the pending lookup still uses the original array. A missing row or thrown y conversion skips x entirely.

`src/common/buffer/BufferLine.ts::getExtended` first multiplies x into a packed-word address, then tests that word's extended-attribute flag. Only a set flag triggers the second x conversion, this time to a property key with the string hint. The selected line precedes multiplication, and the selected property map precedes key conversion. In-place map changes remain visible, while fill/copy replacements leave pending reads on the previous map. Storage receiver identity also survives coordinate hooks that reactivate the addon on another terminal.

At the503 checkpoint, active bindings converted x and y before selecting the row and only inspected integer cells. This ran x hooks for missing rows, reversed error precedence, lost string addition and conditional key conversion, and selected a new buffer/store after reentry.

## Implementation and Rust API

`CircularList::prepare_number_read` creates `CircularListNumberRead`. Its consuming `get_with` callback receives the captured start and returns the numeric sum. The implementation reads current capacity after the callback and queries the captured backing. It holds no list borrow across conversion and copies no elements. Backing maps use shared ownership so in-place mutation remains visible while capacity replacement leaves pending reads on the original map.

`BufferLine::numeric_word` exposes the existing bounds-checked numeric packed-word read. `prepare_extended_read` returns `ExtendedAttributesRead`, retaining the current property object without borrowing it across conversion. Full fill/copy replaces that map; resize deletes entries in place, matching source. The existing `extended_attributes` accessor now returns a Rust `Ref` guard instead of a borrowed map reference. Callers must release that guard before mutation. The browser snapshot consumer already collects its values before calling JavaScript.

`ImageStorage::prepare_buffer_line_read` releases the active-buffer borrow before conversion. `get_image_from_extended` and `extract_tile_from_extended` share the original bitmap/tile projection helpers with the native numeric APIs. Browser `coordinates.rs` keeps the storage, array, line and extended map across the corresponding phases. `coordinates.js` only performs ECMAScript addition, multiplication and computed property-key conversion. It captures `Reflect.ownKeys` before user hooks. Terminal state, remainder, storage and image projection remain Rust.

JavaScript method names and native numeric addon methods are unchanged. No new dependency or unsafe code is needed. Shared map ownership avoids copying every row or property map for ordinary image inspection.

## Verification

Eight native ownership tests cover captured start, live capacity with original backing, repeated replacement, signed/fractional/NaN keys, error identity, retained-object release, map replacement and in-place mutation/deletion. Existing assertions remain intact. Two internal test reads changed only to borrow the new shared map representation.

Three new WASM functions compare unchanged source, direct generated exports and shipped browser constructors. They check missing-row short circuits, default/number/string hints, Symbol/BigInt and thrown-undefined precedence,35 raw y forms and25 raw x forms, numeric/string differences after ring rotation, and actual QOI image/tile pixels through resizing and coordinate-triggered reactivation. All61 inherited functions pass unchanged.

The first browser run exposed the missing second x conversion,63PASS/1FAIL in every engine. Its full reports remain under `pre-key-initial*`. The implementation was extended rather than dropping that assertion. The fixed initial and final matrices each pass64/64 in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4, with zero failures, ignored/filtered tests or page errors. Replacing only the two exported-wrapper/module files with exact main1106 bytes completes61PASS/3FAIL in Chromium. Exactly the three new functions fail. The helper restores both fixed hashes in finally before the final three-engine pass.

Post-fix native workspace all-target/all-feature tests pass2196 across498 summaries, with zero failed, ignored, measured or filtered tests. Warning-denied native workspace all-target/all-feature Clippy, WASM workspace all-feature library Clippy and WASM image-addon-test Clippy pass. All five member formatting checks pass. Exact `cargo fmt --all -- --check` fails with Windows OS206, retained rather than waived. Earlier test-only type-inference failure and the six-test pre-key checkpoint are retained separately.

`results/image-row-coordinates/audit.mjs` passes747 repeated phase-input checks, seven compiled/generated artifact pins, both776-file reference manifests, exact two-file baseline/restoration checks and three actual fresh=false Cargo compilations. Negative and final served WASM differ. These are scoped results, not the complete native OS or original-suite matrix. Artifact paths become historical identities after owned cleanup.

## Reproduction

Use an owned O: worktree with private target and TEMP/TMP, Cargo1.85.1 on PATH, the existing reference dependencies and locked wasm-bindgen0.2.100 CLI. Set `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE`, `WASM_BINDGEN` and `ORIGINAL_REFERENCE` to the preserved installations. Resource grants and fresh headroom checks precede compiler/browser work.

```sh
node results/image-row-coordinates/native.mjs native
node results/image-row-coordinates/native.mjs format
node results/image-row-coordinates/browser.mjs initial
node results/image-row-coordinates/negative-control.mjs
node results/image-row-coordinates/browser.mjs final
node results/image-row-coordinates/audit.mjs
```

The control uses exact Git objects from1106d480, preserves tests, rejects build failures/timeouts/unrelated failures, and verifies restoration in finally. Its browser matrix only succeeds as a control when all61 inherited functions pass and exactly three new functions fail.

## Remaining gaps

Private/prototype mutations can create property keys or payloads not produced by the addon. The authoritative native maps only contain addon-created integer keys and typed payloads; this task does not implement writable source-private backing objects, arbitrary prototype pollution or patched module-initialization intrinsics. Pixel/palette coercion, private options/storage ABI, GC, complete original image639 assertions and img01..48 remain311/324. Native Windows/Linux/macOS completion, release/features and final full browser/workspace coverage also remain open. Scoped green tests do not establish full image or terminal equivalence.
