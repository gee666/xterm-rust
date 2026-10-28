# Live Params view follow-up

This is a design note for a known open compatibility gap, not an implemented contract. Task 194 needs the actual parsing-error logger argument's `state.params` object to stay live and retain identity. `source_repo/src/common/InputHandler.ts` passes the parser's IParams object, not `toArray()` or `clone()`. A cloned Vec in a log record would be a fake snapshot and must not count as coverage.

## Required outcomes

- Retaining the same Params object across error callbacks, resets and later parse calls observes the current logical length, parameters, subparameters and stale backing slots.
- Host wrappers retain object identity while retained by JS. Repeated `.params` reads expose the same live typed backing array, not a freshly copied Int32Array.
- Valid writes to allocated parameter slots through that retained view are visible to subsequent parser/handler reads. Include a write through an Int32Array subarray, not only a helper method.
- The existing `Params::clone()` remains a deep source-compatible copy, including inactive storage and rejection flags. A separate explicitly named shared handle is needed; changing clone silently into sharing would break DcsHandler and ordinary source callers.
- The proposed public ParserApi still receives source-compatible copied arrays. Do not accidentally make proposed handler arguments alias the parser merely because low-level diagnostics need a live handle.
- Source capacity, sentinel, clamping, digit targeting, packed subparameter offsets and the SIXEL stale slot-1 behavior remain unchanged.
- No Params data borrow may span a logger, provider, native or JS callback. Error values and partial state propagate without string conversion or panic.
- A retained parameter view owns parameter storage, not the whole terminal. Do not introduce a terminal/logger/view reference cycle or an unbounded global host-object cache.

## Storage constraint

The current `values() -> &[i32]` API cannot safely coexist with mutation behind a shared reference. Do not cast Vec storage through UnsafeCell or maintain a synchronized second copy to evade that constraint. The primary storage and all readers need a coherent ownership/API change.

Possible approaches need review before implementation:

1. Shared native storage with checked short-lived read/write guards and an explicit live handle. Native consumers must release guards before any callback. This fits Rust borrowing but cannot directly expose an aliasing JavaScript Int32Array over Rust-owned memory without an additional FFI design.
2. A portable parameter-array backend contract. Native arrays can remain Rust-owned; a WASM backend can use an actual JS-owned Int32Array as its primary storage. Its host view then exposes that exact typed array, including subarray/buffer writes, without unsafe aliases into Rust memory. The core must depend only on the portable contract, never js-sys or a browser. This has concrete interop need, but allocation/cross-boundary costs require measurement rather than an assumed speedup.

WASM ownership must also follow task 181's JS-traceable-owner contract. A Rust Rc that strongly roots a JS typed array can leak if user code stores the Params wrapper on that array, forming a cross-heap cycle. Merely avoiding a Rust-only cycle is not enough. The authoritative array must have an appropriate JS owner graph, with weak Rust externrefs and source-compatible lifetimes for retained Params objects, extracted backing arrays and deep clones. Coordinate that design rather than adding a strong global wrapper cache.

An ordinary copied array or proxy over a stale mirror is not a solution. A proxy that intercepts numeric writes but fails to reflect `.buffer` or `.subarray` mutations also does not satisfy the source contract. A stable host identity needs explicit lifetime management; it is not enough that two wrapper objects point to one Rust allocation.

## Checks to add

Capture unchanged-source logger traces that retain Params, compare strict identity after multiple parser errors, read after reset/new CSI, mutate an allocated slot and subarray, deep-clone before later mutation, and observe all resulting cells/events/replies. Port those assertions to native and actual WASM callbacks. Keep malformed backing-object replacement and unsupported Number domains explicit rather than claiming them proved by ordinary Int32Array tests.

Run all existing Params, parser, DCS, SIXEL and terminal suites after the storage migration. The getter added by task 182 is a narrow read-only compatibility accessor, not a substitute for this live view. Until the follow-up has an implementation and current evidence, parser diagnostic object equivalence remains NOT PROVEN.
