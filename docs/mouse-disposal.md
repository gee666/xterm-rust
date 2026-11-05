# Live mouse-service disposal

Task 479 addresses one missing binding under task 305. The real `Terminal._core.mouseStateService` lacked `dispose()`, although its native `MouseStateService` already implements the source lifecycle. `crates/xterm-web/src/terminal/mouse_api.rs` now delegates to that existing implementation. It creates no new mode, registry, callback store or lifecycle authority.

Source `MouseStateService` registers only its protocol emitter for disposal. Direct disposal is idempotent and stops future notifications, but retains active protocol and encoding, restriction and encoding methods, and wheel hooks. Parsed DECSET/DECRST and reset can still change retained state. The terminal remains usable. Disposing a retained subscription is still valid, and subscribing after disposal does not append a handle to an optional collection.

Synchronous disposal during notification keeps the existing source emitter semantics. Clearing the array during a multi-listener dispatch raises a TypeError when dispatch reaches a removed entry. An interrupted reset has already committed `NONE` but has not yet reset its encoding. Adding or removing a listener first creates another array; disposal clears that current array without clearing the snapshot already being dispatched. A listener's own thrown value propagates with its original identity. The binding does not catch these errors or replace them with success.

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

Raw custom registration, ordinary-object registries, arbitrary borrowed receivers and monkey-patched service internals remain task 305. The native and browser implementation still need the broader full-suite and native OS acceptance gates. Overall equivalence remains NOT PROVEN.
