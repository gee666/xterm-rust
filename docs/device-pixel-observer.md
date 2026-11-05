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

Set `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` to the existing tools if they are not on PATH. The browser runner executes every test in Chromium, Firefox and WebKit and closes its browser/server resources. The accepted artifact-server cleanup fix from task 495 is required by these recorded runs. The oracle prelude runs unchanged source for the expected side only. Rust execution uses the new Rust component.

The shared test inputs cover 25 lifecycle/getter/error scenarios, including repeated disposal, bounded reentry, retained predicates, zero/NaN dimensions, inherited properties, getter/proxy order, primitive/empty/null boxes, raw BigInt and coercion cutoffs. Another test observes actual physical dimensions and resizing in a supplied iframe window. Native device-pixel behavior is recorded explicitly. Browsers that reject this observation option must match the source's rejection/disconnect path, without pretending they delivered device-pixel measurements. A separate default-box observation proves actual native delivery in that realm; it is not an addon fallback. A third test verifies deterministic Rust Drop cleanup and expired callback state.

## Verification status

Native retained WebGL tests: 46 passed, zero failed/ignored/filtered. Strict native Clippy passed. The focused all-feature WASM test compiled and strict WASM lib/test Clippy passed. Initial compiler type-annotation and lint failures are retained, not discarded.

The exact workspace formatting command still fails with Windows OS206. Changed-file rustfmt checks pass. This does not replace full workspace quality gates.

Final Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 each passed all three functions, zero failures/ignored/filtered. Each compared all 25 source scenarios. Chromium and Firefox delivered actual device sizes `[123,38]` then `[142,38]`, equal on source and Rust sides. WebKit rejected the native device-pixel box option with TypeError. Source and Rust both disconnected once and produced no device-pixel callbacks. Its independent default-box observer delivered `[123.25,37.5]`. No supported-device delivery is claimed for WebKit.

The first matrix retained a WebKit 2-pass/1-fail source-oracle timeout. Diagnosis confirmed actual native option rejection, not a Rust behavior difference. The test now asserts that rejection/disconnect path and independent native delivery rather than waiting for an event the unchanged source cannot produce. Both the raw failed report and corrected/final reports remain under `results/device-pixel-observer/`.

The production negative control changes the observe catch from required to optional disconnect. Chromium then passes exactly two functions and fails only `observer_lifecycle_and_live_getters_match_unchanged_source` at the synchronous unsupported-delivery/throw trace. The script restores exact source bytes in finally and records matching pre/post SHA-256. The final full three-engine matrix and strict WASM lint passed after restoration. Run `node scripts/device-pixel-observer-negative.mjs` with the same environment to reproduce. It mutates only this owned source file while running, so do not run it concurrently with edits.

Receipts and raw outputs are under `results/device-pixel-observer/`, including initial compiler/lint failures, source/native traces, immutable compiler/binding/module hashes, exact negative-control restoration, final strict lint and OS206 formatting failure. Generated runtime binaries are inspected before cleanup and are not durable source distributions. Independent review and owned-path cleanup receipts follow there. Full equivalence remains NOT PROVEN.
