# Live custom-wheel callback receiver

Task 461 fixes the receiver passed by the actual Rust browser Terminal to `attachCustomWheelEventHandler` callbacks. The source calls `MouseStateService._customWheelEventHandler(ev)`, so `this` is the same live object exposed as `term._core.mouseStateService`. Rust previously used `Reflect.apply` with `undefined`.

The Terminal constructor now retains its existing native-backed mouse facade before converting the core-service wrapper to a JavaScript value. BrowserServices keeps another reference to that same facade, not another mouse service or mode store. DOM wheel invocation uses that reference as the receiver. The handler is cloned before invocation, so replacing the hook and recursively dispatching events does not cross a mutable borrow. `Reflect.apply` still avoids reading an own `call` property. The strict comparison against boolean false and original exception propagation are unchanged.

## Shared cases

`crates/xterm-web/tests/browser_wheel_receiver_cases.js` executes the same operations against the unchanged source Terminal and real Rust Terminal:

- The tracked and passive wheel paths observe the stable facade, live built-in modes, the original event and its extra object. Reset through the receiver affects real shared modes. An extra property survives the next callback. An own throwing `call` getter must not run.
- A passive callback replaces itself, activates tracking, recursively dispatches a wheel event and resumes. Later callbacks observe the replacement and the same service object.
- Twelve return classes exercise literal-false veto versus other falsy, primitive and object values. Eight arbitrary thrown values preserve identity through actual DOM error dispatch, including `undefined`, NaN, symbol and bigint. Mutations before throwing remain visible.

The cases are directed, not randomized. They are not an original upstream suite. The unchanged source ran all three cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. All captures matched across engines. Actual Rust WASM passed all three tests in each engine, with zero failures, ignored tests or filtered tests. The existing five-test live mouse-mode target also passed in each engine. `results/wheel-receiver/source-report.json`, `browser.json` and `modes.json` retain the exact results.

A negative control changed only the apply receiver back to `undefined`. Chromium failed all three tests. Restoring the production receiver passed all three tests again in all engines. The source fixtures were not regenerated to accommodate a Rust mismatch. An initial source-only capture exposed that pixel accumulation obscured allowed return values; the probe was strengthened to line-mode wheel events, with an explicit assertion that false produces zero reports and each of the other eleven values produces one. Final source capture and comparison both passed.

Fifteen native mouse regression tests passed across `mouse_failures`, `mouse_policy` and `mouse_routing`. Native all-target/all-feature xterm-web checking and warning-denied Clippy passed. Warning-denied WASM Clippy passed for the library and both browser targets. Exact `cargo fmt --all -- --check` failed with Windows OS206. All five complete member `cargo fmt -p NAME -- --check` commands passed; that workaround is not reported as an exact full-command pass. Raw logs are retained in `results/wheel-receiver`.

Both browser reports list 70 retained files each: the compiler WASM, generated bindings and WASM, all snippet dependencies and CSS. These copies are under `results/acceptance-artifacts` and `results/wheel-receiver/artifacts`. Historical temporary invocation paths are left intact in the original report fields. `retainedArtifacts` gives current relative paths, SHA256 hashes and sizes. The negative control retains its report and log, not its disposable binary. `source-pins.json` pins production, tests, runner, lockfile and the reference manifest; all 776 original files were verified unchanged before and after execution. `native-pins.json` identifies the temporary executables used during validation.

The pre-existing `terminal/mod.rs` is the cohesive browser Terminal entry point and constructor. It was already 360 lines. Retaining the facade adds three lines; keeping construction ownership together avoids a mechanical split for this small fix.

## Commands

From the owned O: worktree, with Node 24.15.0, Rust 1.85.1, matching wasm-bindgen 0.2.100 and the existing Playwright dependency:

```sh
MOUSE_CASE_FAMILY=wheel-receiver XT_REFERENCE_ROOT=/path/to/read-only/source_repo node scripts/mouse-mode-source.mjs
WASM_TEST=wasm_terminal_wheel_receiver node scripts/browser-host-check.mjs
cargo clippy -p xterm-web --target wasm32-unknown-unknown --test wasm_terminal_wheel_receiver -- -D warnings
```

Use `--generate` only for the first inspected source capture, never to erase a Rust mismatch. Source capture and Rust assertions use the same case file. Original inputs must match the retained 776-file reference manifest before and after execution. Coordinate compiler/browser grants and use Cargo `-j2` with a private O: target.

## Limits

This repairs the wheel receiver, not the complete raw MouseStateService API. The facade still lacks full raw registration and invocation methods, ordinary-object registries, inherited-name behavior and source own-key/prototype descriptors. Those remain task 305. Arbitrary raw protocol and encoder return values, lifecycle after disposal, full unchanged upstream suites and final native/platform/browser quality gates also remain separate. Overall port equivalence remains NOT PROVEN.
