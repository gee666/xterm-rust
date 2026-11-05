# Fallible render window resolution

Task477 adds `FallibleWindowSource` and `BrowserRenderDebouncer::try_new_with_fallible_window_source`. The source's current-window getter can throw while resolving RAF requests and cancellation. Returning a typed Window unconditionally could not represent that result. The new adapter returns the original JsValue error without converting it to a string or creating a replacement Error.

The existing `WindowSource`, fixed-window constructors and `try_new_with_window_source` remain compatible. They delegate to the same scheduler with an infallible resolver. `set_window` installs a successful fixed-window resolver. No scheduling or coalescing algorithm is duplicated.

Window resolution runs after independent callback/source handles have been copied, with no live scheduler or resolver-cell borrow. A failed request retains updated rows and appended refresh callbacks, but does not assign a pending frame. A failed cancellation retains the current frame, including a replacement installed by a nested callback. Cancellation still resolves its pending argument only after successful window and method lookup, as established by task468.

## Verification scope

The unchanged-source oracle expands the retained66 programs to72 by including the six previously excluded source-throw and nested-refresh-then-throw cases. It compares the actual Rust browser scheduler's thrown-object identity, event order, pending frames, retained rows and callbacks with original RenderDebouncer. The original66 cases and real-window RAF test remain, including compatibility coverage through the fixed-window constructor. Another42 programs check source errors carrying undefined, null, false, zero, strings, symbols and BigInt without erasing the thrown value.

Source-only72 main programs plus42 raw-error programs and changed Rust formatting pass. Native, WASM lint/build and three-engine execution are pending resource grants. No runtime success is claimed yet.

This closes only the adapter's fallible window-resolution contract. BrowserService::window remains a typed infallible Rust contract. Raw JS row values, full317 root behavior, full318 rendering behavior and full-port equivalence remain NOT PROVEN. Production uses no reference JavaScript engine. The generated oracle exceeds350 lines because it bundles the intact reference and test corpus; production never imports it.

Run the existing render-scheduler skill and commands in docs/render-scheduler.md, with owned O: scratch/targets and fresh coordinator309 grants. Keep native and source-only counts separate from real browser results.
