# Buffer ranges and markers

## Callable mappings

| Source | Rust |
| --- | --- |
| `IBufferRange` and coordinate objects | `buffer::range::{BufferRange, BufferPosition}` with public `start/end` and `x/y` |
| `getRangeLength(range, cols)` | `get_range_length(BufferRange, i32) -> Result<f64, RangeError>` |
| `new Marker(line)` | `buffer::marker::Marker::new(i64)` |
| marker `id`, `line`, `isDisposed` | `id() -> u64`, `line() -> i64`, `set_line(i64)`, `is_disposed()` |
| marker `onDispose` | `on_dispose() -> Event<()>` |
| marker `register(resource)` | `register(DisposableHandle) -> DisposableHandle` |
| marker `dispose()` | `dispose() -> Result<(), EmitError>` |

Ranges retain signed coordinates and perform the source binary64 formula in source operation order. Only reversed rows fail. Reversed columns, negative widths and coordinates are not silently validated away. `RangeError` retains the range and displays the exact source message. Coordinates and column counts outside i32, fractions and non-finite values are not represented by this typed core API. Future bindings must define conversion rather than cast unchecked.

Markers use i64 lines so every u32 buffer row and the negative disposed sentinel are representable. Valid JS integer lines within its safe-number range are exact. Values outside the JavaScript safe-integer domain are not promised equivalent by this typed API. The line stays writable after disposal because it is a public mutable field upstream. Repeated disposal does not overwrite a line subsequently edited by a caller.

`Marker` clones share one `Rc` state. `WeakMarker` supports callbacks without strong self-reference cycles. The library creates no strong marker-to-callback-to-marker cycle itself. A caller that captures a strong Marker in its own retained callback can create a cycle and must break it explicitly, just as other Rc APIs require. Dropping the last marker drops its resources but does not invoke their `dispose` callbacks. Explicit disposal is required, matching the source lifecycle rather than adding fallible callback execution inside Drop.

IDs come from one process-wide atomic counter. This small shared counter is required by the source static identity contract, not a global object registry. Allocation is relaxed-atomic and requires no object locks. IDs advance from one through 2^53, then remain at 2^53, matching JavaScript's numeric increment precision boundary. Concurrent Rust threads allocate from the same counter even though each marker remains single-threaded. Distinct JavaScript isolates would instead have separate module counters; a browser binding must scope its runtime accordingly.

Marker disposal sets disposed=true and line=-1 before firing its event. After a successful event it disposes the emitter, then visits registered resources in live insertion order. Duplicate resources are called more than once. Resources appended during event or resource callbacks participate in the same traversal. Late registration after a completed disposal is retained without eager disposal. No RefCell borrow crosses a callback.

A fallible event listener returns `EmitError` and stops cleanup. Panic-based callback failures unwind, leaving disposed state visible and later resources uncalled. Repeated disposal does not retry cleanup after either failure. Marker deliberately does not implement the infallible `Disposable` trait. That would force errors to be swallowed or converted to panics. Buffer consumers call its fallible method and propagate errors explicitly.

## Assertion mapping and evidence

`crates/xterm-core/tests/buffer_range_marker.rs` contains:

- Four unchanged upstream BufferRange case expectations, single-line, invalid-row, multi-line and next-line/out-of-width.
- Exact error text, signed/zero/negative results and i32 precision boundaries.
- A 1,149-row JavaScript-generated range fixture, including 125 boundary combinations and 1,024 seeded cases, checked by exact binary64 result bits or error text.
- Clone identity, distinct increasing ids, mutable lines including u32::MAX, invalidation-before-event and repeated disposal.
- The shared upstream marker callback trace, duplicate registration, reentrant disposal and late registration.
- Live resource additions during cleanup, listener failure, listener/resource panics, event snapshot removal and weak ownership/drop checks.

The ID precision-limit transition is tested as a pure allocator transition inside marker.rs, without mutating the global counter and interfering with parallel tests. The generated 1,149-line TSV exceeds the normal source-file line cap because it is deterministic oracle data, not implementation code.

No standalone Marker.test.ts exists in this source. Buffer.test.ts contains three addMarker cases at lines 1158-1189, two marker reflow cases beginning at lines 489 and 551, and their assertions. Those are not claimed executed by utility tests. Task27 and criterion122 own their real Buffer integration and faithful migration.

Commands from the repository root:

```sh
node scripts/check-buffer-range-marker.mjs
cargo test -p xterm-core --test buffer_range_marker
cargo test -p xterm-core --lib buffer::marker::tests
cargo clippy -p xterm-core --lib --test buffer_range_marker -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/buffer/range.rs crates/xterm-core/src/buffer/marker.rs crates/xterm-core/tests/buffer_range_marker.rs
```

The Node checker uses built-in TypeScript transformation on unmodified source copies under tmp/. It changes import plumbing and uses strict Node assertion adapters for the four original range assertions, without modifying original tests. Default mode verifies committed fixtures. `--write-fixtures` regenerates them when intentionally reviewing an upstream change. Source and fixture hashes, seed 1112690247, runtime and counts are retained under results/buffer-range-marker/.

Observed execution is native Windows x64, Node v24.15.0 and Rust 1.85.1. Linux/macOS and browser bindings remain unrun. No unsafe code is present in these modules. Full terminal equivalence is NOT PROVEN.
