# SIXEL on the browser terminal

Task370 adds `xterm_web::image::BrowserSixelProtocol`. It connects the safe Rust SIXEL controller to the real streaming DCS parser, live terminal attributes/theme, browser canvas and shared image store. It is a producer for the unfinished ImageAddon coordinator, not a partial public addon constructor.

## Ownership and initialization

Create `ImageRendererLifecycle`, share its renderer with `ImageStorage`, and pass those instances plus `Rc<Cell<ImageOptions>>` to `BrowserSixelProtocol::new`. Retain the token returned by `register`. The new `ParserApi::register_streaming_dcs_handler` registers the handler directly, without the public whole-sequence adapter's accumulation limit. Registration disposal detaches both active and future delivery, as verified against the source terminal.

The producer starts with a pending decoder. The coordinator must deliver a `SixelDecoder` through `complete_loading` when decoder initialization completes. Build that decoder using `SixelHandler::decoder_options` and the options captured at constructor time. Live scrolling, palette and sequence-size policy continue to use the shared options. Pending hook/put calls are discarded, not replayed. The browser tests exercise before-readiness and mid-sequence readiness against the unchanged asynchronous source handler. This explicit boundary does not implement or prove the public constructor's initialization scheduling.

At hook time, the controller reads the live native attributes and theme colors. Before `open`, background extraction returns transparent black even for RGB attributes, matching source. Default, ANSI, RGB and inverse backgrounds use the existing native policy. A P2 value of1 skips background lookup.

At unhook, the adapter captures dimensions, creates a canvas in the global document, looks up the context/method/constructors, copies current decoder pixels into ImageData, releases oversized decoder storage, and inserts through the shared SIXEL placement policy. A null context still inserts the blank canvas. Empty-width images advance the real terminal line-feed path only when scrolling is enabled. Browser methods, warnings and storage callbacks run without a retained controller borrow. Reset during `putImageData` does not discard the already copied image.

Synchronous host and storage errors retain their original JS identity and stop the parser tail. Decode/size warnings are emitted before the handler aborts/releases, so a throwing warning preserves the source partial state.

### Engine diagnostic strings

An added malformed-input test found a real mismatch. The portable Rust decoder's remainder error text matched Chromium, but Firefox said `integer divide by zero`. WebKit included `this._wasm.decode(0, length)` in its diagnostic. Native structured error kinds cannot supply these browser-generated strings.

`image/sixel_errors.js` creates tiny standard WASM modules solely to obtain the current engine's remainder, memory-access and indirect-call-signature diagnostic strings. No image decoding, protocol parsing, terminal state or source engine code runs there. The helpers catch the trap at the WASM invocation so WebKit reports the same expression rather than an inlined helper call. There is no new dependency, unsafe Rust, or source assertion normalization. All three trap categories have real source/Rust terminal tests in Chromium, Firefox and WebKit. This is directed malformed-input coverage, not a claim that all malformed integer combinations or patched global constructors are equivalent.

## Verification

Observed on Windows with Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, based on main `50514541e1d57b0d8aedaa692b556df95297fa68`:

- 15 new browser test functions pass in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine has zero failures, ignored or filtered tests and no unexpected page errors. These are directed source differentials, not15 original upstream tests.
- The comparator bundles unchanged source Terminal, SixelHandler, SixelImageStorage, ImageRenderer and ImageStorage. Shared IIP/SIXEL fixtures compare every captured cell's text, width, foreground/background and image/tile payload, wrapped lines, cursor/scroll positions, replies, added callbacks and storage usage. Tests also compare stored pixels and rendered canvas bytes. Seed `0x51370370` controls stream chunking.
- Cases include raster/color decoding, palette persistence/reset, cancellation, bytewise ST, readiness, backgrounds before/after open, empty-width cursor movement with fallback metrics, scrolling policy, null contexts, size/decode warnings, all three runtime trap categories, constructor/method getter ordering, synchronous error identity, reset reentry, storage callback exceptions, large images above the release threshold, active unregister/reregister and actual browser queued writes.
- The shared fixture extraction preserves all12 existing IIP tests, passing in each engine. Canvas23 and lifecycle13 regressions also pass in each engine. IIP's original assertions were not weakened; the shared comparison additionally checks replies.
- A negative control registered DCS `r` instead of `q`. All13 then-existing Chromium tests failed, zero passed. The mutation was restored before the final15-test matrix. The receipt is retained.
- `cargo test --locked -j2 --workspace --all-targets --all-features` passes2036 tests across396 summaries with zero failures/ignored tests. This includes two native streaming registration tests. Native browser-only tests compile as empty crates and do not prove browser behavior.
- Workspace all-target/all-feature warning-denied Clippy, scoped WASM library/IIP/SIXEL warning-denied Clippy, exact full workspace formatting and JS syntax checks pass. Full formatting first hit Windows error206 at the longer worktree path and passed after moving the owned worktree to a shorter O: path.

Initial image getter fixtures incorrectly inspected cells overwritten by trailing text. The source/Rust full-state comparisons had passed. The fixtures now move the tail beyond the image or inspect pixels before overwriting. A new native fixture also initially assumed active parser registrations survived disposal and used an incorrect debug variant name. Its corrected disposal expectation is independently exercised against the unchanged source in the browser suite. These initial fixture errors are distinct from the real engine-message mismatch described above. Concise diagnostics and final receipts are under `results/image-sixel-terminal`.

Reproduce with the reference/tool environment documented in `docs/image-canvas.md`:

```sh
cargo fmt --all -- --check
cargo test --locked -j2 --workspace --all-targets --all-features
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_sixel_terminal_browser --test image_iip_terminal_browser -- -D warnings
WASM_TEST=image_sixel_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_iip_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_canvas_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_lifecycle_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
```

## Integrated-base refresh

After acceptance, the producer rebased onto main `2f1be3b980494d9618a9049fd1a7440534ff6444`. All owned source/test/runner bytes match the reviewed commit. The union15-test matrix again passed in all three engines. Workspace all-target/all-feature tests passed2040 tests across398 summaries with zero failures or ignored tests, and workspace Clippy/full formatting passed. Exact union reports are retained as `results/image-sixel-terminal/union-*`. The batch also includes the confirmed documentation-only task367 integration receipt `4a8ef32`, without replaying its already integrated source.

## Canonical integration

The canonical helper integrated the accepted producer into main `331f0eead64246aa67d951bbd3c9ff3ae65cc4f7`. Changed core/web all-target builds and tests, affected all-target/native workspace checks and applicable WASM library checks passed1875 tests across357 summaries. The helper removed the private target. It includes the separately accepted alternate-cleanup fixture correction371.

An earlier gate completed its tests but safely refused to move main because a documentation-only371 receipt advanced main during the build. Its log is retained as `canonical-first-refusal.log`. The helper then rebased and reran every gate without bypassing the stale-ref check. This did not change the reviewed SIXEL source. No extra browser pass is claimed for the documentation-only main advance.

## Independent review

Task370 was accepted at `4520a1f`. The isolated reviewer inspected the implementation, the diagnostic-only WASM helper, original-source comparator and fixture extraction. It independently reran15 SIXEL and12 IIP tests per browser, two native streaming tests, all2036 workspace tests across396 summaries, native workspace/scoped WASM warning-denied Clippy, full formatting and JS syntax checks. No findings remained. Its browser reports and focused native log are retained as `results/image-sixel-terminal/reviewer-*`.

The runner retains source/Rust input hashes and compiler/WASM/bindings/CSS identities. Generated binaries and browser run directories are disposable; concise reports are copied to this task's results directory. Full ImageAddon initialization/coordinator/lifecycle hooks, Kitty integration, raw constructor options, complete original image suites and img01..48, all malformed input/host-mutation behavior, final cross-platform matrices and overall equivalence remain NOT PROVEN under tasks311/324/327.
