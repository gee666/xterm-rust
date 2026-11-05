# WebGL cursor blink timer manager

`webgl::cursor_blink::CursorBlinkStateManager` ports the supplied `addons/addon-webgl/src/CursorBlinkStateManager.ts`. It uses the caller's live browser service, window timer methods and global `Date.now`. This is a browser component, not an exported WebglRenderer or WebglAddon. Task 306 retains actual renderer construction/layers, shared atlas runtime/cache/limits, addon events, Root restoration and full original WebGL/custom-glyph/demo suites.

## Rust representation and ownership

`new` takes an `Rc<dyn Fn() -> Result<(), JsValue>>` render callback and the supplied browser service as `JsValue`. Clone shares the same manager. `is_cursor_visible`, `is_paused`, `restart_blink_animation`, `pause`, `resume` and fallible `dispose` correspond to the source manager's methods/properties. The Rust callback has no JavaScript manager receiver. This component does not export a JS object or claim arbitrary mutation of the source class's public field. Timer functions retain their window receiver, raw handles and thrown object identities.

No manager borrow spans a browser getter, method or render callback. The window and method are read before the live cancellation argument, preserving getter reentry. Clock values and elapsed-time arithmetic retain raw JS values, coercion and errors through the existing shared interop primitives. Minimal new JS performs only the catching `> 0` operator. All manager decisions and state live in Rust.

Explicit disposal only cancels currently truthy handles in source order. It does not permanently close the manager, reject queued callbacks or reset visibility and idle state. Pause does reset visibility and idle state. Resume clears previous timers and the saved restart timestamp, starts the initial blink timeout, resets idle timeout and requests a visible frame. Every clear happens before the corresponding handle is forgotten, so errors and callback reentry retain the source's partial state. Zero-valued timer handles remain falsy, just as in the source.

Browser callbacks hold weak Rust state. Last-owner Drop cancels remaining handles and reports cleanup errors through the existing unhandled-error reporter. After the last owner disappears, retained browser callbacks do nothing. This Rust ownership boundary has no JS destructor equivalent. It also cleans successfully installed timers if construction subsequently fails, whereas the JS constructor can leave those timers installed. Explicit-operation equivalence does not imply equivalence of unreachable JS object retention. Future renderer callbacks must not capture a strong manager clone in a cycle.

## Preserved timing details

The blink interval is 600 ms and idle timeout is five minutes, matching source constants. Restart while active saves a timestamp and coalesces a visible animation frame rather than thrashing intervals. The initial timeout defers only for a positive remaining delay. Interval restart forwards zero or negative remaining delays to the supplied host. Fired timeout handles remain stored until cancellation/replacement, as upstream. Idle completion makes the cursor visible, stops blinking and renders directly. Restart after idle resets idle timeout but does not implicitly restart blinking. The supplied source has this behavior; it is not corrected here.

## Reproduction

Use the team compiler/browser grant, Cargo `-j2`, private O: target and O: TEMP/TMP. Existing source dependencies suffice. `wasm-bindgen` CLI must be 0.2.100.

```sh
SOURCE_ROOT=/absolute/unchanged/source_repo node scripts/cursor-blink-oracle.mjs
cargo test -p xterm-web --test webgl --all-features -j2
cargo clippy -p xterm-web --lib --test webgl --all-features -j2 -- -D warnings
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_cursor_blink --all-features -j2 -- -D warnings
WASM_TEST=wasm_cursor_blink CARGO_FEATURES=acceptance-hooks WASM_PRELUDE="$PWD/tmp/cursor-blink/prelude.js" node scripts/browser-host-check.mjs
node scripts/cursor-blink-negative.mjs
```

Set `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to existing installations if required. The oracle verifies all 776 baseline original file hashes, bundles the unchanged manager and records every actual source build input. Shared test code supplies deterministic timers only; it makes no blink decisions. Rust does not execute the source manager. Browser assertions compare 38 source scenarios covering focus, timeout/interval/RAF/idle, deferred restarts, zero/string/object handles, raw clock values, exact errors, getters, synchronous callbacks and bounded reentry. Another assertion waits for actual 600 ms iframe-window timer delivery and native animation frames, then verifies a microtask restart. A third tests deterministic Rust last-owner cleanup and expired callbacks.

The negative control inverts the production initial-timeout positive-delay decision, expects only the lifecycle test to fail in Chromium and restores exact source bytes in finally. It must run without concurrent edits to the owned source file. Final verification must follow restoration. It does not mutate source_repo or assertions.

## Verification status

The source oracle passed all 38 scenarios with 776 original hashes verified. Focused all-feature WASM compilation and strict lib/test Clippy passed. Retained native WebGL tests passed 46 functions, zero failures/ignored/filtered, and strict native lib/test Clippy passed. Initial module-privacy compile errors are retained and fixed. Changed-file formatting passes; exact `cargo fmt --all -- --check` fails with Windows OS206 and is not waived.

Initial and final runtime matrices each passed all three functions in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, zero failures/ignored/filtered. Each lifecycle assertion compared all 38 unchanged-source traces. Actual iframe timers and frames produced `[[false,false],[true,false]]` on source and Rust sides in every engine. Rust last-owner cleanup cancelled both timeout handles and retained callbacks were inert.

The production initial-timeout positive-delay guard inversion passed exactly two functions and failed only the lifecycle comparison in Chromium. The script restored exact source bytes in finally with equal SHA-256 before/after. Final full three-engine execution and strict WASM lib/test Clippy passed after restoration. The negative report retains the full mismatching source/Rust traces.

Raw reports are under `results/cursor-blink/` and `results/browser-host/wasm_cursor_blink/`. `node scripts/cursor-blink-audit.mjs` verifies source/durable evidence against retained pins. Before cleanup, `--runtime` also checks actual compiler WASM, every generated binding/snippet module, prelude and native EXE. `--record` is the producer snapshot operation, not independent verification. Cargo reuses the raw compiler path across mutant and restored builds. The final raw compiler artifact is checked against the final report; each run's separately retained delivered WASM/bindings and generated modules are checked independently. The overwritten raw mutant compiler file is not claimed retained. Runtime files are disposable only after their hashes are checked; durable evidence remains after cleanup. These scoped checks do not replace full workspace/platform/browser matrices. Full equivalence remains NOT PROVEN.
