# Synchronized-output cancellation

Task511 repairs cancellation in the real browser RenderService. It does not complete tasks317/318/301 or prove whole-port equivalence.

The reference is `source_repo/src/browser/services/RenderService.ts`, SHA-256 `17edb34b5e757cc7a20ece8384ce44d8bcaccf09b36b081f5da5ec985b2805fa`. Its `SynchronizedOutputHandler.flush` and `dispose` test whether a timer exists, resolve the current window and its `clearTimeout` method, then read the current timer argument. Getter reentry can clear that argument or replace it. A successful call clears the timer slot. A thrown getter or method leaves the slot and buffering state unchanged by the interrupted outer operation. Reentrant operations can still have changed them.

Rust previously took the handle before looking up or calling the host method, used a non-catching binding, and dropped its callback regardless of the outcome. The repair catches lookup/call exceptions without converting their identity, reads the argument after the method getter, and clears the slot only after success. `flush` now returns `Result<Option<RenderRange>, JsValue>`; explicit service disposal propagates cancellation failures. Destructors report cleanup failures through the existing console path because Rust destructors cannot return errors.

Callbacks use the existing TaskQueue JS-owned closure pattern. The RenderService constructor already initializes TaskQueue's checked ES2021 GC runtime. A host can retain a callback despite cancellation, or reentry can leave a different registration queued after its handle is overwritten. Transferring ownership before scheduling prevents Rust from invalidating those functions. Closures capture weak state, not a strong reference cycle. The normal timeout slot is still canceled on disposal/drop. As with the existing browser queue, a callback whose Rust state has been dropped cannot resurrect it.

## Regression programs

`scripts/render-sync-cancel-source.mjs` bundles unchanged source in memory and preserves eight original-module hashes. `--write` refreshes the generated test-only oracle and source fixture. Without that flag, byte drift is an error. Generated fixtures exceed the repository's normal file-length limit because they contain the unchanged bundled reference and complete trace data. Production never imports them.

The shared driver runs54 programs through the source and Rust RenderService. Ten cover ordinary cancellation, getter/call reentry that buffers, flushes, replaces or disposes, and ignoring an overridden method `.call` property. Thirty-seven cover object, undefined, null and string throws from lookup/call on refresh/disposal paths, with optional reentrant buffering, plus five noncallable method values that must throw TypeError rather than invoking an object's `.call` property. Seven cover delivery during cancellation and retained callback delivery after successful cancellation. Tests compare receiver identity, exact handle reads including undefined, row ranges, viewport/render events, timer/frame counts, disposal, thrown-value identity and continued callback delivery.

The existing RenderService test keeps its original assertion and source trace. Its setup is reused rather than duplicated. The original source assertions and all source files remain unchanged.

## Commands

Use an owned O: worktree, private target and TEMP/TMP. Obtain compiler/browser grants before heavy work, keep Cargo at-j2, and reuse the installed esbuild, Playwright and matching wasm-bindgen0.2.100 CLI through the existing environment overrides.

```sh
node scripts/render-sync-cancel-source.mjs
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test render_service_browser --all-features -- -D warnings
WASM_TEST=render_service_browser node scripts/browser-host-check.mjs
```

The browser test includes three new Rust test functions expanding54 programs and the retained RenderService function. It compares actual Rust against an unchanged source bundle executed in the same browser, and separately checks the committed source trace.

## Remaining scope

This change retains typed numeric timeout IDs and the existing infallible BrowserService window contract. It does not establish arbitrary raw timeout IDs, throwing window resolution, all setTimeout getter/call phases, complete synchronized-output/root integration, whole-renderer pixel equivalence, original complete browser/demo suites, final stable workspace gates or native Linux/macOS runtime evidence. Full-port equivalence remains NOT PROVEN.

## Executed checks

Receipts are in `results/render-sync-cancel`. On Rust1.85.1 Windows/MSVC, the scoped native xterm-web all-target/all-feature run passed415 tests across171 harnesses with zero failed, ignored, measured or filtered tests. Initial and final warning-denied native all-target/all-feature Clippy and actual WASM library/render_service_browser Clippy passed. The WASM test build passed.

Both the initial and restored final Chromium/Firefox/WebKit matrices passed all four Rust functions per engine. Three new functions expand54 source/Rust programs in each engine; the fourth is the retained RenderService trace. The early-take/noncatch cancellation control failed all three new functions while the retained function passed. The old Rust-owned callback control failed the delivery function while the other three passed. Both controls ran in Chromium only. Exact production bytes were restored in finally blocks.

The initial/final compiler artifact, served WASM and binding-JS hashes are identical. The audit verified296 generated-stage pins,14 source/build-input pins,28 durable raw report/log copies and all776 reference hashes in both the original and private physical copy. Compiler pins use the helper's immutable acceptance-artifact snapshots, not Cargo's reused output filename. Generated runtime artifacts are retained through review, then removed during owned cleanup. Their recorded hashes remain in the receipts.

All five workspace package formatting checks passed. The exact aggregate `cargo fmt --all -- --check` still failed with Windows OS206. That is recorded as a failure, not upgraded to a full formatting pass. The native tests are scoped to xterm-web, not a full final workspace or OS matrix. Browser tests are actual RenderService component tests, not complete terminal/demo equivalence.
