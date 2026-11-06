# Image-required internal core inspection

Task569 repairs the missing internal service paths in the unchanged original ImageAddon accessor test. These are adapters over the Rust terminal, not fixture objects or a JavaScript terminal engine. Full image, browser and private ABI equivalence remains NOT PROVEN.

## Implemented contracts

`terminal._core._inputHandler` is a stable object. `_parser` registers streaming DCS, OSC and APC handlers on the existing native parser registries. CSI callbacks receive a Params adapter. Registration does not borrow the active parser, and returned disposables unregister the actual handlers. JavaScript method lookup preserves the handler receiver. Callback failures propagate and Promise completion is driven by the existing Rust write queue.

`_curAttrData` and `getAttrData()` return the same current attribute object. SGR updates its foreground/background and color/inverse reads in place. Reset changes the identity and freezes retained old views using the core's existing attribute identity, rather than keeping a synchronized browser copy. Native tests cover hard/soft reset and borrow errors.

`_dirtyRowTracker` marks the native pending render range. `clearRange`, `markDirty`, `markRangeDirty` and `markAllDirty` operate on that range. A native DCS callback test checks the actual emitted render range, not just an inspection getter. `lineFeed()` invokes the real LF control and its line-feed event. `onRequestReset` subscribes to the common reset notification and returns an actual disposable.

`_core.buffer` aliases `_bufferService.buffer`. Its `lines.get()` retains the actual native row. `_data` returns an owned Uint32Array of packed logical cells, and `_extendedAttrs` returns real extended-attribute values. Re-reading a retained row observes parser changes. Copies avoid unsafe aliases into reallocating WASM memory.

`_themeService.colors` reads the browser's ThemeService colors, including foreground/background/cursor/ANSI entries. `_coreBrowserService` reads the real owning window, main document, DPR and focus state. These services are undefined before open and remain stable after open.

The upstream test transport bundles the generated bindings. It previously renamed the exported `_AttributeData` class to `__AttributeData`, although the direct WASM binding had the correct name. `keepNames: true` preserves binding constructor names through transport bundling. It does not install fake constructors or change source assertions. A transport unit regression reproduces the old rename.

## Scope limits

This is read-only inspection plus the listed operations, not a source-writable private object graph. Packed arrays, extended-attribute maps, color arrays and Params data are owned projections, not shared mutable JavaScript backing arrays. Retained Params are callback snapshots rather than parser scratch identities. Streaming payloads preserve codepoints but use owned slices with zero-based offsets. Repeated line/color reads need not return identical wrapper objects. Private CSI callback-array receiver identity is not represented by this adapter.

Raw JavaScript number/operator semantics for dirty-row indices, direct mutation/replacement of private services and attributes, all other private parser/attribute methods, complete descriptor/enumeration semantics and GC equivalence remain open under the full image/public/browser families. The pending dirty getters expose the native pending range; they do not promise the source's retained post-parse tracker history. Unsupported methods are absent, not successful no-ops.

## Verification

The shared `scripts/image-core-trace.js` runs through real original and Rust Terminals. It checks19 complete snapshots covering live/retained attributes and rows, nonempty underline/hyperlink maps, LF/events, numeric dirty ranges, theme mutation, browser ownership, streaming DCS LIFO/async cleanup, OSC/APC/CSI, reset listener disposal and normal/alternate buffers. The generated fixture exceeds350 lines because it preserves complete source snapshots, including packed rows.

The unchanged original ImageAddon accessor test runs separately through the actual production WASM package. The raw initial transport-name failures are retained, not normalized. The final shared19-snapshot fixture also ran against the exact old224d wiring and failed at the missing input handler, then restored every fixed source byte in finally. That old0-pass/1-fail result is a negative control, not a passing old implementation. Scoped results, source/input/artifact pins, exact commands and remaining unrun gates are recorded in `results/image-core/summary.json`.

```sh
CORE_VIEW_CASES=image-core node scripts/core-views-source.mjs
WASM_TEST=wasm_image_core node scripts/browser-host-check.mjs
# Optional control, only in a clean owned swarm worktree, allow240s externally:
node results/image-core/negative-control.mjs
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite addon-image --grep 'test for private accessors' --no-build --timeout-ms 120000
node --test scripts/e2e/build-bootstrap.test.mjs
cargo test --locked -j2 -p xterm-core --all-targets --all-features
cargo clippy --locked -j2 -p xterm-core --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_image_core -- -D warnings
cargo fmt --all -- --check
```

Set Cargo, rustc and wasm-bindgen0.2.100 paths as needed. Use owned O: target/TEMP/TMP/TMPDIR paths and an explicit resource grant. The source runner needs the built unchanged original under SOURCE_ROOT and its existing Playwright installation under PLAYWRIGHT_MODULE. Original-suite execution uses a physical reference copy with owned dependency access. Unlink any owned dependency junction before whole-worktree cleanup. Do not recursively delete a reference junction.
