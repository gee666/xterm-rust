# Live mouse-service disposal

Task 479 addresses one missing binding under task 305. The real `Terminal._core.mouseStateService` lacked `dispose()`, although its native `MouseStateService` already implements the source lifecycle. `crates/xterm-web/src/terminal/mouse_api.rs` now delegates to that existing implementation. It creates no new mode, registry, callback store or lifecycle authority.

Source `MouseStateService` registers only its protocol emitter for disposal. Direct disposal is idempotent and stops future notifications, but retains active protocol and encoding, restriction and encoding methods, and wheel hooks. Parsed DECSET/DECRST and reset can still change retained state. The terminal remains usable. Disposing a retained subscription is still valid, and subscribing after disposal does not append a handle to an optional collection.

Synchronous disposal during notification keeps the existing source emitter semantics. Clearing the array during a multi-listener dispatch raises a TypeError when dispatch reaches a removed entry. An interrupted reset has already committed `NONE` but has not yet reset its encoding. Adding or removing a listener first creates another array; disposal clears that current array without clearing the snapshot already being dispatched. A listener's own thrown value propagates with its original identity. The binding does not catch these errors or replace them with success. The shared cases compare the TypeError class and committed state, not engine-specific TypeError wording. Error-message text equivalence is not established by this scope.

## Verification commands

Run from an isolated worktree with a current resource grant, Cargo `-j2`, matching wasm-bindgen 0.2.100, the WASM target and existing reference browser dependencies:

```sh
MOUSE_CASE_FAMILY=disposal XT_REFERENCE_ROOT=/path/to/source_repo node scripts/mouse-mode-source.mjs
WASM_TEST=wasm_terminal_mouse_disposal node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_mouse_modes node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_mouse_builtins node scripts/browser-host-check.mjs
cargo test -j2 --locked -p xterm-core --test mouse_callbacks --test mouse_custom --test mouse_state
```

The eight shared cases in `crates/xterm-web/tests/browser_mouse_dispose_cases.js` run against the original browser Terminal and the actual Rust Terminal. They compare complete JSON captures for direct disposal, a recursive single-listener reset, interrupted multi-listener reset, removed and added listener snapshots, callback throw identity, parser mode changes and real DOM routing with a retained wheel callback. These are source-derived differential cases, not the complete unchanged upstream suite. No original source or assertion is edited. Use `--generate` only after inspecting source behavior and captures.

## Observed results

On Windows, Rust/Cargo 1.85.1 and Node 24.15.0, all eight source cases matched across Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Both the initial and the final actual Rust matrices passed eight test functions per engine. After exact production restoration, the existing five mouse-mode and four raw-builtin test functions also passed all three engines. The 24 focused native regressions passed. Scoped native all-target/all-feature Clippy and WASM library/test all-feature Clippy passed. Exact workspace formatting failed with Windows OS206; all five complete member formatting checks passed. These are scoped gates, not a full workspace or native OS matrix.

The exact pre-change `mouse_api.rs` produced eight missing-method panic logs when the full Rust test target ran in Chromium, then wasm-bindgen-test hit an externref-table trap before producing a reliable suite summary. That diagnostic is retained in `negative.json`; it is not counted as eight completed failing tests. One unchanged synchronous assertion then ran alone on the same old binary. It discovered eight tests, selected and executed one, reported zero passes and one expected failure, and left seven unrun. Its captured console identifies the missing `mouse.dispose` method. The first isolated verification script expected that message in the output rather than the console and rejected the receipt; both receipts are retained. No product code or test assertion changed to obtain the final control result.

A `finally` restored the production file byte-for-byte before the final passing matrices. `restoration.json` records identical before/after SHA-256 hashes. Raw reports, generated final browser bindings and WASM, all four compiler binaries, three native executables, command logs, pin manifests and original-reference checks are under `results/mouse-disposal`. The `.mjs.txt` files are audit snapshots of the temporary orchestration scripts, not standalone user commands. The original 776 reference hashes were unchanged after testing.

Raw custom registration, ordinary-object registries, arbitrary borrowed receivers and monkey-patched service internals remain task 305. The native and browser implementation still need the broader full-suite and native OS acceptance gates. Overall equivalence remains NOT PROVEN.
