# Fallible render window resolution

Task477 adds `FallibleWindowSource` and `BrowserRenderDebouncer::try_new_with_fallible_window_source`. The source's current-window getter can throw while resolving RAF requests and cancellation. Returning a typed Window unconditionally could not represent that result. The new adapter returns the original JsValue error without converting it to a string or creating a replacement Error.

The existing `WindowSource`, fixed-window constructors and `try_new_with_window_source` remain compatible. They delegate to the same scheduler with an infallible resolver. `set_window` installs a successful fixed-window resolver. No scheduling or coalescing algorithm is duplicated.

Window resolution runs after independent callback/source handles have been copied, with no live scheduler or resolver-cell borrow. A failed request retains updated rows and appended refresh callbacks, but does not assign a pending frame. A failed cancellation retains the current frame, including a replacement installed by a nested callback. Cancellation still resolves its pending argument only after successful window and method lookup, as established by task468.

## Verification scope

The unchanged-source oracle expands the retained66 programs to72 by including the six previously excluded source-throw and nested-refresh-then-throw cases. It compares the actual Rust browser scheduler's thrown-object identity, event order, pending frames, retained rows and callbacks with original RenderDebouncer. The original66 cases and real-window RAF test remain, including compatibility coverage through the fixed-window constructor. Another42 programs check source errors carrying undefined, null, false, zero, strings, symbols and BigInt without erasing the thrown value.

Source-only72 main programs plus42 raw-error programs pass. All74 focused Windows native tests, strict focused native/WASM Clippy and both WASM test builds pass. Independent early review reran the source programs and all five native executables with no failures. All five member format checks pass. Exact aggregate cargo fmt still fails with Windows OS206, so it is not reported as a workspace formatting pass. Initial and restored final browser matrices pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine executes72 main programs,42 raw-error programs and the real-window RAF test across three Rust test functions. The existing RenderService regression passes its one Rust test in each engine. There are no failed, ignored or filtered tests in these positive runs.

Two negative controls replace only request or cancellation source-error propagation with a replacement undefined error. Each Chromium run fails exactly two Rust test functions and retains one passing real-RAF test. The first mismatch is request/source/throw or disposal/source/throw, respectively. Each mutation restores the original bytes in finally and verifies SHA-256. The complete final matrices,74 native tests and strict native/WASM checks pass after restoration. Initial and final generated WASM and binding hashes are identical.

Commands, raw logs, source/native binary pins, browser reports and restoration records are in results/render-window-source. The negative compiler target paths are historical and reused by later builds; their content-addressed compiler copies and generated binding/WASM outputs remain available through review. These are scoped Windows/browser results, not a full workspace or Linux/macOS runtime pass.

This closes only the adapter's fallible window-resolution contract. BrowserService::window remains a typed infallible Rust contract. Raw JS row values, full317 root behavior, full318 rendering behavior and full-port equivalence remain NOT PROVEN. Production uses no reference JavaScript engine. The generated oracle exceeds350 lines because it bundles the intact reference and test corpus; production never imports it.

Run the existing render-scheduler skill and commands in docs/render-scheduler.md, with owned O: scratch/targets and fresh coordinator309 grants. Keep native and source-only counts separate from real browser results.
