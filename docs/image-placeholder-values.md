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

## Verification

On Windows with Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, the directed image-addon target passes31/31 test functions in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The target contains28 retained tests and the three new placeholder tests. It is not the full original639-case family or img01..48.

A negative control restored all four production files to their exact pre431 implementation at0ddac49b, plus the three existing Rust test files whose argument types changed. The new tests remained registered. Chromium passed28 and failed exactly the three new tests. All seven files were restored byte-for-byte in `finally`; the fixed target then passed31/31 in all three engines again. `negative-audit.json` records baseline, control and restored hashes. No original source or assertion changed.

Focused WASM Clippy passes with warnings denied. Formatting passes separately for all five workspace packages. The exact aggregate `cargo fmt --all -- --check` command still fails with Windows OS206, command line too long. Its failed receipt is retained, not reported as a pass. No native Linux/macOS runtime, full-workspace gate or complete addon equivalence is established here.

Raw browser reports, source hashes, negative control and quality logs are in `results/image-placeholder-values/`. `identity-audit.json` verifies all97 Rust input pins, all121 original comparator source files against the supplied P: reference, and all seven old/restored files against Git and saved fixed bytes. Historical paths identify the owned O: test checkout and are not promised to remain after cleanup.

Use the dependency/tool environment described in `docs/image-canvas.md`, with all temporary work on O:.

```sh
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
cargo fmt --all -- --check
```

Overall equivalence is NOT PROVEN. Independent review is pending.

## Remaining work

Other image options still use eager typed conversion. `_opts` is still a read-only diagnostic, not the source's writable private record. The complete input-handler/private ABI, full original image suites and img01..48 remain tasks311/324. This repair does not establish those contracts or full-workspace/native-platform acceptance.
