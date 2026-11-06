# Live geometry in selection event deduplication

`SelectionService.setSelection` clears its model, runs drag teardown, assigns the requested start and length, refreshes, and checks whether the selection changed. Source `SelectionModel.finalSelectionEnd` reads current terminal columns, rows and active-buffer scrollback at that last step.

Rust passed dimensions captured before teardown to `fire_if_changed`. A host removal callback can call `Terminal.resize`. Although the first event exposes the correct public selection, Rust stored the old wrapped endpoint in its deduplication state. An identical second `select` therefore emitted an extra selection-change event. A synchronous refresh callback could cause the same stale snapshot.

The repair changes only the final `fire_if_changed` argument in `crates/xterm-web/src/selection/service_actions.rs`. It reads `buffer_dimensions` after teardown and refresh callbacks. It does not hold a model or buffer borrow across either callback. The existing explicit dimensions argument remains useful to the refresh API and preserves the Rust calling contract. No new dependency, unsafe code or JavaScript terminal engine is used.

## Checks

Four custom method cases load unchanged source SelectionService and SelectionModel methods. Teardown and refresh each change columns from 20 to 10 or 32. Both widths alter the wrapped endpoint for start 8, length 15. The first select emits one event; an identical second select emits none. These are custom differential cases, not four upstream test definitions.

One native test function runs the same four cases using actual BufferService resizing and SelectionService callbacks. Its old-code control fails at the intended count, two events instead of one. The fixed focused native run also retains the prior selection teardown, service, model and unchanged-corpus replay assertions.

The shared public trace opens an actual Terminal, writes through its parser, starts a real DOM drag and intercepts mousemove or mouseup listener removal. It shrinks or grows the terminal, optionally throws an exact object or Symbol after resizing, and repeats the same selection. Eight cases compare selected text, endpoints, dimensions, partial state, throw identity and every selection-change snapshot. The source runner verifies exact embedded source-map text and input hashes. The Rust test runs the same program on the actual Rust/WASM Terminal.

The old WASM control fails at the intended whole-trace equality, with duplicate events in the nonthrowing cases. Throwing controls retain the cleared model and resized terminal; the next select succeeds. Final Rust results match unchanged source in Chromium, Firefox and WebKit. One Rust test function contains eight cases per engine. Retained `wasm_selection_teardown`, `wasm_selection` and `wasm_terminal_selection` also run unchanged.

## Commands

Run from an isolated worktree. Use the current team compiler/browser allocation, Cargo `-j2`, an owned O: target and O: `TEMP`, `TMP`, `TMPDIR`. Set `SOURCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to the existing original reference and locked tools.

```sh
node scripts/selection-live-geometry-methods.mjs
node scripts/selection-live-geometry-source.mjs
cargo test --locked -j2 -p xterm-web --test selection_live_geometry --test selection_teardown --test selection_service --test selection_model --test selection_service_differential
cargo test --locked -j2 -p xterm-web --all-targets --all-features
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_selection_live_geometry --test wasm_selection_teardown --test wasm_selection --test wasm_terminal_selection --all-features -- -D warnings
WASM_TEST=wasm_selection_live_geometry node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection_teardown node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_selection node scripts/browser-host-check.mjs
```

`--generate` on the source runner creates the initial fixture only. Do not regenerate it to hide a Rust failure. The source runner is separate from the frozen task545 producer so its retained eighteen-case proof remains reproducible.

## Scope and evidence

Raw reports are under `results/selection-live-geometry`. Strict native and scoped WASM Clippy passed. Exact workspace formatting still fails with Windows OS206, and whole xterm-web member formatting passes. These Windows Rust 1.85.1 checks are not the final workspace, stable, release, native Linux/macOS or full upstream browser matrices.

Task545 is an accepted prerequisite. This worktree privately copied its original four commits onto main224d1b33. Canonical integration must pick only task557's own commits after the original task545 source, never these private copies.

Full task321 selection/link behavior remains open. DOM removal ordering, stable repeated listener handles, timer lifetime, arbitrary live option sampling, raw numeric representations, complete suite/demo coverage and GC/disposal proof are not closed here. Search527 orphan registration ownership remains unchanged and open. The ordinary image/Fit matrix failures inspected before this repair remain recorded by task327. Full equivalence is NOT PROVEN.
