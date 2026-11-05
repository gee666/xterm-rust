# Raw synchronized-output timer handles

Task522 extends accepted task511 in the actual Rust RenderService. Full renderer task318 and whole-port equivalence remain open.

The unchanged `source_repo/src/browser/services/RenderService.ts` stores the exact result of `window.setTimeout`. Scheduling uses `this._timeout ??= ...`; cancellation tests `this._timeout !== undefined`. A null result therefore allows another registration but is still passed to `clearTimeout`. Undefined allows another registration and is not canceled. False, zero, NaN, strings, symbols, BigInts and objects are retained without conversion. Returning an object must not invoke its conversion hooks.

The old `web_sys::Window` timer binding converted the result to i32. Rust now keeps a raw `JsValue`, resolves the host method and invokes it through `Reflect.apply`. No timer-slot borrow spans a host getter or call. A successful schedule assigns its result after reentry, just like the source assignment. A failed getter/call leaves the interrupted outer assignment undone. Cancellation retains task511's live post-getter handle read, exception identity and clear-only-after-success behavior. Callbacks retain task511's JS-owned closure lifetime and weak state capture.

## Tests

The existing RenderService fixture and unchanged-source generator are reused. The original RenderService assertion, its fixture and all54 cancellation program traces remain unchanged. Two additional Rust tests compare actual Rust, intact source executed in each browser and generated source traces.

`render_sync_handles.mjs` defines116 new programs. Sixteen handle kinds each run flush, dispose and retained-delivery paths, totaling48. Object, Symbol, null and undefined handles each run17 getter/call reentry or primitive/object-throw paths, totaling68. These check nullish rescheduling, exact cancellation identity, negative zero, absent conversion hooks, receiver identity, ignored own `.call`, callback delivery before assignment, callback survival after cancellation/errors, buffered ranges, render/viewport events and pending host work.

The source-only generator verifies eight unchanged source hashes. It produces `tests/fixtures/render-sync-handles.json` and extends the existing generated test-only source oracle. Generated trace/oracle files exceed350 lines because they retain complete source programs and traces. Production never imports them.

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

Receipts go in `results/render-sync-handles`. Source-only passes do not establish Rust behavior. Final native/browser execution results will be recorded after execution. The browser suite has six Rust functions, expanding54 cancellation and116 raw-handle programs plus the original RenderService trace. A directed control restores the exact accepted task511 typed-handle production file, leaving assertions intact; the two new functions must fail. Exact production bytes must be restored in finally before the final matrix.

## Remaining scope

This repair does not make the existing BrowserService window lookup fallible. It does not prove all root/public bindings, complete original browser/demo suites, renderer pixels, final workspace quality gates, native Linux/macOS runtime behavior, full mapping or comparisons. Full-port equivalence remains NOT PROVEN.
