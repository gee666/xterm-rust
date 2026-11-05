# Composition end ordering

Task395 fixes one concrete input-family defect found while auditing task319. Overall equivalence remains NOT PROVEN.

## Source contract and fix

`CompositionHelper.compositionend` calls `_finalizeComposition(true)`. The original method removes the overlay's `active` class before clearing `_isComposing`, setting `_isSendingComposition`, or capturing the current range and suffix. DOM method replacement can throw or synchronously dispatch another composition event. Those operations must see the old composing state, and successful reentry can replace the range captured by the outer end.

The Rust DOM adapter previously called `Composition::end` before `classList.remove`. It now calls it afterward. No model borrow spans the DOM call. A thrown removal leaves flags unchanged and schedules no send. A nested start changes the range used by the outer successful end.

The retained failing Chromium run demonstrates the difference. During removal the source reports composing=true while old Rust reports false. With a nested start selecting `X` in `aXtail`, followed by a value of `aBtail`, source sends `B`; old Rust sends `a` and incorrectly remains composing.

## Regression and evidence

`tests/input_cases/composition_end.rs` is included by `wasm_input_callbacks.rs`. Its JS driver compares original method bodies with real Rust DOM listeners for ordinary removal, throwing removal with exact thrown-object identity, nested start and nested start/end. It records composing state, overlay class and every output message during and after removal. Both source and Rust use actual browser tasks, with no sleeps or timing thresholds. The driver restores the decorated method and error listener in `finally`.

The three copied method bodies are unchanged and retain MIT attribution. `scripts/composition-source-fixture.test.mjs` verifies them against the supplied source, normalizing line endings only. They contain no terminal-engine replacement used by Rust.

On native Windows x86_64, Node24.15.0, Rust1.85.1 and wasm-bindgen0.2.100:

- Before the fix, both new functions fail in Chromium148.0.7778.96. This run used a standalone target before the cases moved into the existing callbacks target to avoid unused shared test helpers.
- After the fix, callbacks pass5 functions in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Two functions cover the four new scenarios. Existing input7, live keyboard15, accessibility component25 and real-Terminal accessibility7 also pass per engine.
- Native input/keyboard/accessibility tests pass27 functions. The original clipboard/IME oracle executes11 unchanged source cases with16 assertions and passes252,412 Rust comparisons, seed1229800758. Accessibility matches32,871 state snapshots and65,536 BMP classifications, seed10559090.
- Source-integrity3, focused rustfmt and warning-denied native/WASM Clippy pass. These are scoped checks, not a workspace or three-native-OS gate.

Raw reports, exact hashes and logs are under `results/composition395/`. Paths to generated WASM and browser glue in those reports are execution provenance, not committed distribution files. Disposable build artifacts are removed after integration; reports preserve their hashes.

## Review and integration

Independent task395 review accepted the fix after rerunning source-integrity3, native27, callbacks5 per engine, package formatting and focused warning-denied native/WASM Clippy. Full-workspace formatting hit Windows OS error206; it is not a pass.

The canonical helper rebased onto main's accepted Base64 changes and integrated `a4e43ca600cfd00e9a60fc8bbe87986db95785c7`. Its changed-crate gate passed343 xterm-web tests across129 summaries with zero failed or ignored, affected native all-target checking, default workspace checking and applicable WASM library checking. All eight owned source hashes stayed unchanged. The helper removed its private target. `results/composition395/integration.log` retains the raw command output. This is not an all-features workspace test or a new browser run of unrelated Base64 behavior.

## Reproduce

From a checkout with the matching Rust/WASM target and external browser tools configured:

```sh
node --test scripts/composition-source-fixture.test.mjs
WASM_TEST=wasm_input_callbacks node scripts/browser-host-check.mjs
cargo test -p xterm-web --test input_upstream --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters --test accessibility_state --test accessibility_differential
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_input_callbacks -- -D warnings
node scripts/browser-input-oracle.mjs
XT_SOURCE_ROOT=<reference-with-esbuild> node scripts/accessibility-oracle.mjs
```

Set `WASM_BINDGEN` to the matching CLI and `PLAYWRIGHT_MODULE` to an installed Playwright `index.mjs` when they are outside this checkout. Use a bounded runner and the agreed Cargo `-j2` resource limit.

Task319 still owns complete input/accessibility closure. These tests do not run the entire original Terminal browser suite or establish every decorated DOM getter/value, OS IME delivery or native Linux/macOS behavior.
