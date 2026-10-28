# Shared event conversion values

Task 187 adds dispatch-local memoization for Rust-backed browser events. The source `source_repo/src/common/Event.ts` passes the same event object to every listener. Converting a copied Rust struct into a new JavaScript object for each callback breaks observable identity and hides mutations made by earlier listeners. The core must not import JsValue or retain a last-event browser object indefinitely.

## Contract

`DispatchKey<T>` identifies one converter result type. Clone a single key across registrations made by the same browser event function. Different converters get separate keys even if both produce JsValue. The generic key is invariant, so Rust function-pointer/lifetime subtyping cannot reinterpret a cached Any payload as another type.

`Event::try_subscribe_with_context` receives the ordinary borrowed payload plus `&DispatchContext`. Existing subscribe, fire and disposal signatures remain unchanged. The context is synchronous, not cloneable or publicly constructible. A caller cannot capture that borrowed context in its own static cached value.

`context.memoize_with(&key, create)` returns `Result<Rc<T>, EmitError>`. The first completed conversion is retained for that key and reused by later listeners. `try_memoize_with` accepts a fallible converter; a failed conversion is not cached and its original error is preserved. No cache borrow crosses converter invocation. Recoverable reserve failures propagate instead of becoming a missing or successful payload.

Each ordinary `Emitter::fire` creates a fresh context. Recursive fires get separate contexts while the outer context remains intact. Any and forward preserve the originating context. Map and filter_map execute independently per listener and give each produced output a fresh context. Filter_map keeps one original registration and skips only explicit None outputs.

Native cache roots are released when fire returns, errors or unwinds. A listener may explicitly retain the returned Rc; a JavaScript callback may retain its JS object through normal JS references. There is no global mutable cache, address-based payload lookup or browser dependency. This mechanism does not replace a component's separate responsibility to model persistent object identity and mutable native state correctly across different fires.

## Binding use

Create one `DispatchKey<JsValue>` beside the cached callable for an event getter. Each registration uses the context-aware subscriber. Convert with memoize_with or try_memoize_with, then pass the referenced JsValue to the actual callback. All listeners using that key see the same JavaScript object and earlier property mutations. Do not allocate a fresh key per listener. Returning an existing class/view object from the converter preserves its existing identity naturally.

Cache cleanup follows the dispatch frame, not the lifetime of the event function. No last-payload field or deferred timer is needed. Arbitrary JS callback failures remain wrapped by the browser-owned JsCallbackError inside EmitError and are recovered through the standard Error::source chain.

## Required verification

`tests/event_context.rs` covers shared mutable results, no retained cache roots after fire, nested isolation/restoration, separate converter keys, cleanup after error/panic, any/forward origin sharing, per-map output contexts, None filtering, failed-conversion recovery and recursive memoization without a borrow conflict. Existing Event/CircularList source baselines and differential traces must remain green.

The WASM consumers must additionally assert `===`, mutation visibility across listeners, nested event ordering and no stale payload in progress/onResize tests. Those are integration obligations under tasks 31/164, not established by the native fake-object tests alone. Implementation and tests are source-only during the shared disk pause and require actual execution plus independent review before acceptance.
