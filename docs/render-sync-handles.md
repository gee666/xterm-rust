# Raw synchronized-output timer handles

Task522 extends accepted task511 in the actual Rust RenderService. Full renderer task318 and whole-port equivalence remain open.

The unchanged `source_repo/src/browser/services/RenderService.ts` stores the exact result of `window.setTimeout`. Scheduling uses `this._timeout ??= ...`; cancellation tests `this._timeout !== undefined`. A null result therefore allows another registration but is still passed to `clearTimeout`. Undefined allows another registration and is not canceled. False, zero, NaN, strings, symbols, BigInts and objects are retained without conversion. Returning an object must not invoke its conversion hooks.

The old `web_sys::Window` timer binding converted the result to i32. Rust now keeps a raw `JsValue`, resolves the host method and invokes it through `Reflect.apply`. No timer-slot borrow spans a host getter or call. A successful schedule assigns its result after reentry, just like the source assignment. A failed getter/call leaves the interrupted outer assignment undone. Cancellation retains task511's live post-getter handle read, exception identity and clear-only-after-success behavior. Callbacks retain task511's JS-owned closure lifetime and weak state capture.

## Tests

The existing RenderService fixture and unchanged-source generator are reused. The original RenderService assertion, its fixture and all54 cancellation program traces remain unchanged. Two additional Rust tests compare actual Rust, intact source executed in each browser and generated source traces.

`render_sync_handles.mjs` defines116 new programs. Sixteen handle kinds each run flush, dispose and retained-delivery paths, totaling48. Object, Symbol, null and undefined handles each run17 getter/call reentry or primitive/object-throw paths, totaling68. These check nullish rescheduling, exact cancellation identity, negative zero, absent conversion hooks, receiver identity, ignored own `.call`, callback delivery before assignment, callback survival after cancellation/errors, buffered ranges, render/viewport events and pending host work.

The source-only generator verifies eight unchanged source hashes. It produces `tests/fixtures/render-sync-handles.json` and extends the existing generated test-only source oracle. Generated trace/oracle files exceed350 lines because they retain complete source programs and traces. Production never imports them.

## Contract mapping

All rows below map `SynchronizedOutputHandler` in the unchanged RenderService.ts to `renderer/service_sync.rs`. They are component contracts, not a claim about complete terminal integration.

| Input or host action | Required behavior | Shared programs |
| --- | --- | --- |
| Return undefined | Schedule again on another buffered range; skip cancellation | `undefined/flush`, `/dispose`, `/deliver` |
| Return null | Schedule again, but pass null unchanged when canceling | `null/flush`, `/dispose`, `/deliver` |
| Return another primitive, object or function | Do not reschedule until cleared; cancel the exact value without coercion | All other handle kinds, three paths each |
| Scheduling getter reenters buffering | Nested schedule completes before the outer call and assignment | `get-buffer` for four handle kinds |
| Scheduling call reenters buffering | Outer return overwrites the nested registration's handle | `call-buffer` for four handle kinds |
| Scheduling getter/call flushes or disposes | Retain source range and assignment order | `get-flush`, `call-flush`, `get-dispose`, `call-dispose` |
| Scheduling getter/call throws | Propagate the exact thrown value; do not perform the interrupted assignment | Eight throw actions for four handle kinds |
| Callback is delivered during scheduling | Disable synchronized output and render before outer handle assignment completes | `call-deliver` for four handle kinds |
| Cancellation getter replaces a timer | Read the replacement handle after the getter | `clear-get-replace` for four handle kinds |
| Cancellation call replaces a timer | Pass the pre-call handle, then clear the slot only after success | `clear-call-replace` for four handle kinds |
| Host retains a canceled or orphaned callback | Callback remains callable and performs source refresh behavior | Retained delivery in every program |

## Commands

Use an owned O: worktree, private target and TEMP/TMP. Obtain current compiler/browser resource grants and use Cargo-j2. Reuse installed esbuild, Playwright and matching wasm-bindgen0.2.100 through existing environment overrides.

```sh
node scripts/render-sync-cancel-source.mjs
cargo fmt -p xterm-web -- --check
cargo test --locked -p xterm-web --all-targets --all-features
cargo clippy --locked -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -p xterm-web --target wasm32-unknown-unknown --lib --test render_service_browser --all-features -- -D warnings
WASM_TEST=render_service_browser node scripts/browser-host-check.mjs
```

Receipts go in `results/render-sync-handles`. The Windows/MSVC Rust1.85.1 xterm-web all-target/all-feature native run passed415 tests across171 harnesses, with zero failed, ignored, measured or filtered tests. Warning-denied native all-target/all-feature Clippy and actual WASM library/render_service_browser Clippy passed. All five package formatting checks passed; the exact aggregate command failed with Windows OS206. Initial and restored-final Chromium/Firefox/WebKit runs each passed all six Rust functions with zero failed, ignored or filtered tests. The two new functions expand116 raw-handle source/Rust programs per engine; three older functions expand54 cancellation programs and one retains the original trace. Post-restoration strict native and actual WASM library/test Clippy also passed. The browser suite has six Rust functions, expanding54 cancellation and116 raw-handle programs plus the original RenderService trace. The directed control restored the exact accepted task511 typed-handle production file without changing assertions. Chromium reported four older functions passing and exactly the two new raw-handle functions failing. Production bytes were restored in finally before the final matrix; the restoration hashes match. The control compiler hash differs from the fixed build, while initial/final compiler, served WASM and binding-JS hashes match exactly. The light audit verified222 generated-stage pins,15 input pins,22 durable raw report/log copies and all776 reference hashes in both original and private copies. Generated runtime/compiler artifacts remain available through review, then are removed during owned cleanup. Raw verifier/auditor snapshots are execution receipts, not portable tooling.

## Remaining scope

This repair does not make the existing BrowserService window lookup fallible. It does not prove all root/public bindings, complete original browser/demo suites, renderer pixels, final workspace quality gates, native Linux/macOS runtime behavior, full mapping or comparisons. Full-port equivalence remains NOT PROVEN.
