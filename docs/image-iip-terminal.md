# IIP execution on the browser terminal

Task365 adds `xterm_web::image::BrowserIipProtocol`. It connects the existing Rust streaming IIP controller to the real OSC parser, browser decoder, terminal input and shared image store. It does not expose a partial `ImageAddon` constructor or delegate protocol decisions to JavaScript.

## Ownership and use

Create `ImageRendererLifecycle` for the terminal, obtain its shared renderer, and create `ImageStorage` with `TerminalStorageHost::from_terminal`. Pass those same instances to `BrowserIipProtocol::new` with typed `IipOptions`. Call `register` and retain the returned `DisposableHandle`. The integration tests show this complete setup.

Registration uses the native streaming OSC1337 registrar, not the public whole-string OSC adapter. Header and BASE64 data therefore reach the controller as chunks, without the public adapter's ten-million-code-unit accumulation limit. The protocol reads current terminal dimensions, renderer metrics and DPR at the requested geometry phase. Cell reports use the terminal's native input route with `was_user_input=false`.

Unscaled QOI decoding, canvas creation and image placement finish synchronously. Resized QOI and encoded browser formats return a fallible pending parser result. The browser write queue drives that result before parsing the remaining text and invoking completion. No controller borrow crosses geometry reads, browser constructors, warnings, insertion callbacks or an await.

Synchronous decoder, ImageData, Blob, bitmap-call and QOI insertion exceptions propagate through the parser with their original JS value. The bitmap promise branch catches both decode rejection and insertion failures, warns with the original cause, then consumes the sequence. If that warning throws, the pending parser result rejects with that thrown value and does not resume the input tail.

`reset` resets the controller and QOI decoder. It does not cancel pending browser promises. Disposing a registration only unregisters that token. Pending work retains its original store and a strong browser-terminal context until completion, so storage reset/disposal or dropping the external terminal handle does not discard a late image. Registered handlers otherwise retain only a weak browser-terminal handle, avoiding a terminal/parser/protocol ownership cycle. Public buffer/parser JS wrappers can retain the native engine until browser GC; the lifetime regression checks the browser-terminal context rather than assuming synchronous JS finalization.

The full addon coordinator still must own renderer/storage disposal, reset/render/resize/buffer subscriptions, graphics policy, protocol selection and ordered repeated activation. Those responsibilities remain tasks311 and324. This producer intentionally does not claim them. Its typed options are not a raw-JavaScript constructor-options implementation.

## Verification

Observed on Windows, Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, based on main `a2a125247d5ebeb4105fff27518fa3ef56b792e8`:

- The new browser suite passes all12 Rust test functions in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Every engine reports zero failed, ignored or filtered tests and no unexpected page errors. These are directed producer tests, not12 original upstream tests.
- The comparator bundles the unchanged original browser Terminal, IIP handler, renderer and storage. It runs their actual parser and store. Comparisons include every buffer cell's text, width, foreground/background words and image/tile payload; wrapped rows; cursor/base/viewport positions; memory usage; added callbacks; replies; stored image bytes and rendered canvas bytes. QOI streaming uses seed `0x1337365`, and multipart input runs bytewise with BEL and split ST terminators.
- The cases cover real PNG and QOI pixels, immediate versus pending completion, input-tail ordering, raw exception identity, bitmap rejection, insertion callback failures, a throwing asynchronous warning, cancellation, disabled inline images, before/after-open reports, resized geometry, unregister/reregister, late insertion after reset/disposal and pending terminal lifetime. A separate test uses the real browser write queue and its completion callback.
- The throwing-warning case uses the supported `logLevel='off'` option on both terminals. This isolates protocol rejection from the source InputHandler's optional slow-handler logging branch. At default logging that source branch also produces a secondary unhandled rejection. A diagnostic run observed it, and Firefox still reports it through Playwright despite an expected-rejection DOM listener. No page-error filtering or source changes were used to obtain the final pass. That logger contract is outside this producer's claim.
- A temporary negative control registered OSC1338 instead of1337. All11 then-existing Chromium tests failed, zero passed. Restoring1337 made the suite pass. The later throwing-warning case brings the final total to12. No mutation remains in production.
- Existing canvas23 and renderer-lifecycle13 tests pass in all three engines through the updated runner. The native web all-target/all-feature suite passes328 tests across115 summaries, with zero failures or ignored tests. These native tests do not execute browser-only code.
- Warning-denied native web all-target/all-feature Clippy, scoped WASM library/new-test Clippy and the exact workspace formatting command pass. Full formatting required moving the owned worktree to a shorter O: path because the initial long path hit Windows error206.

The first new-suite run used an incorrect comparator payload accessor; it read extended attributes instead of their `payload`. The next run exposed invalid test `FilePart:` syntax where source expects `FilePart=`. Both test setup errors were corrected without changing source assertions or production protocol behavior. An initial lifetime assertion also assumed immediate JS wrapper finalization; the final regression checks weak browser-terminal availability directly. These diagnostic failures are not evidence of a source/Rust mismatch.

Run from a checkout with the matching reference dependencies and WASM tooling:

```sh
cargo test --locked -j2 -p xterm-web --all-targets --all-features
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_iip_terminal_browser -- -D warnings
cargo fmt --all -- --check
WASM_TEST=image_iip_terminal_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_canvas_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
WASM_TEST=image_lifecycle_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
```

For dependencies outside the checkout set `SOURCE_REPO`, `REFERENCE_NODE_MODULES`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` as described in `docs/image-canvas.md`. The runner records original source inputs, Rust inputs, compiler/WASM/bindings/CSS hashes and browser counts. Concise logs, hash reports and the negative-control receipt are retained under `results/image-iip-terminal`. Generated binaries and temporary test copies are disposable and can be rebuilt with these commands.

Full ImageAddon, original image suites and img01..48, raw option/coercion boundaries, complete native/browser/platform matrices and overall equivalence remain NOT PROVEN.
