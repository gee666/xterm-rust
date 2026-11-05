# Image placeholder property values

Task431 repairs the public `ImageAddon.showPlaceholder` property. It does not complete image addon parity.

The source constructor shallow-copies its options twice. The public getter returns the first copy's `showPlaceholder` value unchanged, and the setter stores its argument before calling the renderer. Although TypeScript declares a boolean, JavaScript callers can pass undefined, null, numbers, strings, BigInts, symbols or objects. Truthiness controls rendering, not the value returned by the getter. Reset changes scrolling and palette options, not this property.

The Rust/WASM getter and setter now use `JsValue`. The constructor retains the raw first-copy placeholder value. The read-only `_opts` inspection reads that same value. The existing native storage boolean and renderer boolean are derived truthiness projections, not independently writable public values. No borrow of the raw value survives a renderer call, so a throwing host callback retains the committed value and reentry can replace it. No dependency or unsafe code was added.

This is a browser binding representation change. Rust callers use `set_show_placeholder(JsValue::FALSE)` and receive a `JsValue` from `show_placeholder()`. Native `ImageStorage` and `ImageOptions` remain typed.

## Tests

`crates/xterm-web/tests/image_addon_support/raw_placeholder.rs` exercises the actual wasm-bindgen property and the unchanged source class.

- Eighteen values cover undefined, null, booleans, positive and negative zero, NaN, other numbers, strings, BigInts, a symbol, plain objects, arrays, a boxed false boolean and an object that throws if coerced. Constructor and setter checks use `Object.is`, including nested replacement loops, reset, disposal and post-disposal assignment.
- A getter returning distinct objects on each constructor copy verifies that the first value is retained and reset does not read the caller's options again.
- Real opened terminals exercise placeholder canvas allocation under truthy and falsy assignments. A temporary `ImageData` constructor proxy records allocation and tests exact thrown-object identity plus synchronous reentry. The original global property descriptor is restored in `finally`. Rendering still runs in Rust.

Existing boolean assertions are unchanged in meaning; their Rust argument and expected-value types were adapted to `JsValue`.

## Verification status

Focused browser and quality runs are pending. Record exact commands, counts, baseline control and source hashes here before review. Overall equivalence is NOT PROVEN.

## Remaining work

Other image options still use eager typed conversion. `_opts` is still a read-only diagnostic, not the source's writable private record. The complete input-handler/private ABI, full original image suites and img01..48 remain tasks311/324. This repair does not establish those contracts or full-workspace/native-platform acceptance.
