# WebGL device pixel observer

`webgl::device_pixel_observer::observe_device_pixel_dimensions` ports the supplied `addons/addon-webgl/src/DevicePixelObserver.ts`. It constructs the caller's `parent_window.ResizeObserver`, observes the supplied element with the original one-element `box` array, and returns a fallible Rust disposable. This component does not expose or complete WebglRenderer, WebglAddon, Root restoration or shared atlas runtime wiring. Task 306 retains those obligations.

## Observable ordering

Rust calls the live `entries.find` method, compares each entry's target by strict identity, and checks the inherited `devicePixelContentBoxSize` property. Unsupported entries disconnect and clear the captured observer. Width and height come from separate reads of `devicePixelContentBoxSize[0]`, before either comparison. Positive dimensions reach the callback without rounding or coercing the delivered values. Zero, negative and NaN dimensions do not fire the callback. Catching property-read and greater-than bindings retain primitive receiver, BigInt, coercion and exception behavior. All observer decisions run in Rust, not the source engine.

Only `observe` is caught. Constructor errors escape. The observe catch performs an unconditional disconnect of the live observer binding. This matters if observe synchronously delivers an unsupported entry and then throws, because the callback has already cleared that binding. Callback/disposable disconnects are optional instead.

The supplied `common/Lifecycle.ts` implements `toDisposable` as `{dispose: fn}`. Each explicit disposal therefore calls disconnect again, including after an earlier disconnect error. It does not suppress queued callbacks. The Rust adapter preserves that behavior rather than assuming an idempotent store. A synchronous constructor delivery can still report the source's uninitialized observer ReferenceError.

Rust ownership adds last-owner cleanup if disposal was never attempted. Explicit disposal errors return unchanged. Drop reports cleanup errors through the existing unhandled-error reporter. JavaScript-managed callbacks retain weak Rust state, avoiding an observer/closure/state reference cycle; callback delivery after Rust ownership ends does nothing. Engine-generated TypeError/ReferenceError wording is not asserted as a cross-language contract. Error classes and deliberately thrown object identities are checked.

## Reproduction

Use the current team resource grant, a private O: target and explicit O: TEMP/TMP. CLI wasm-bindgen must be 0.2.100. No dependency installation is needed if the existing source dependencies are available.

```sh
SOURCE_ROOT=/absolute/unchanged/source_repo node scripts/device-pixel-observer-oracle.mjs
cargo test -p xterm-web --test webgl --all-features -j2
cargo clippy -p xterm-web --lib --test webgl --all-features -j2 -- -D warnings
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_device_pixel_observer --all-features -j2 -- -D warnings
WASM_TEST=wasm_device_pixel_observer CARGO_FEATURES=acceptance-hooks WASM_PRELUDE="$PWD/tmp/device-pixel-observer/prelude.js" node scripts/browser-host-check.mjs
```

Set `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` to the existing tools if they are not on PATH. The browser runner executes every test in Chromium, Firefox and WebKit and closes its browser/server resources. The oracle prelude runs unchanged source for the expected side only. Rust execution uses the new Rust component.

The shared test inputs cover 25 lifecycle/getter/error scenarios, including repeated disposal, bounded reentry, retained predicates, zero/NaN dimensions, inherited properties, getter/proxy order, primitive/empty/null boxes, raw BigInt and coercion cutoffs. Another test observes actual physical dimensions and resizing in a supplied iframe window. Unsupported native device-pixel boxes must produce a real delivery and disconnect, not a skipped success. A third test verifies deterministic Rust Drop cleanup and expired callback state.

## Verification status

Native retained WebGL tests: 46 passed, zero failed/ignored/filtered. Strict native Clippy passed. The focused all-feature WASM test compiled and strict WASM lib/test Clippy passed. Initial compiler type-annotation and lint failures are retained, not discarded.

The exact workspace formatting command still fails with Windows OS206. Changed-file rustfmt checks pass. This does not replace full workspace quality gates.

Three-engine execution and negative-control receipts are pending. Results belong in `results/device-pixel-observer/`. Full equivalence remains NOT PROVEN.
