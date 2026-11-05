# Live built-in mouse service methods

Task 470 addresses a bounded part of task 305. The actual `Terminal._core.mouseStateService` had live native-backed mode properties but omitted `restrictMouseEvent` and `encodeMouseEvent`. Both methods now call the existing Rust raw-field policies through `RawMouseEvent`. They do not execute the JavaScript terminal engine.

`crates/xterm-web/src/terminal/mouse_api.rs` reads the active protocol or encoding from the same native `MouseStateService` used by parser mode changes and DOM mouse routing. The name-to-policy match only selects a shared algorithm. It does not store a second active mode or copy the encoding/restriction algorithm. Event fields remain live JavaScript properties. Getter/setter failures propagate as the original thrown value, including non-Error values. Encoded strings retain UTF-16 code units.

The active policy is selected before reading event fields. A getter can switch modes or recursively call the facade. The outer call finishes with its selected policy and later calls see the new native mode. No native state borrow crosses a JavaScript property read or coercion.

This binding supports the five built-in protocols and three built-in encodings already accepted by its mode setters. Raw custom registration, ordinary-object registry lookup, inherited keys, monkey-patched service fields, borrowed receivers and arbitrary custom return values remain task 305. A native-only custom registration is not silently projected into this raw built-in adapter. An unsupported name produces an explicit error. This is not full MouseStateService equivalence.

## Verification

`crates/xterm-web/tests/browser_mouse_builtin_cases.js` runs unchanged against the original browser Terminal and the actual Rust Terminal. Four cases cover the builtin protocol/encoding matrix, exact live-field read/write/coercion order, getter/setter thrown-value identity, recursive mode changes, parsed DECSET/DECRST, reset and calls after disposal. The failure matrix includes eight thrown values, including `undefined`, `null`, `NaN`, BigInt and Symbol. Encoded results compare UTF-16 code units rather than converting through Rust UTF-8.

Use an isolated worktree and current resource grants. With reference dependencies, matching wasm-bindgen 0.2.100 and the WASM target available:

```sh
MOUSE_CASE_FAMILY=builtin-methods XT_REFERENCE_ROOT=/path/to/source_repo node scripts/mouse-mode-source.mjs
WASM_TEST=wasm_terminal_mouse_builtins node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_mouse_modes node scripts/browser-host-check.mjs
cargo test -j2 --locked -p xterm-core --test mouse_callbacks --test mouse_custom --test mouse_state
```

Only generate new fixtures with `--generate` after inspecting the original source and reference hashes. The normal source run rejects drift. The runner closes servers and browsers through `finally`; compiler and browser commands have deadlines. Source captures and raw reports belong in `results/mouse-builtin-methods`.

On Windows with Rust/Cargo 1.85.1 and Node 24.15.0, all four source cases matched in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. All four actual Rust test functions passed per engine. The captures compare 260 protocol rows, 150 encoding rows, 636 failure/identity rows, eight ordered-property traces and 15 reentry/lifecycle trace entries. These are comparison rows, not additional Rust test functions.

Replacing only `mouse_api.rs` with its exact `32bf9544` version produced zero passes and four failures in Chromium because the methods were missing. A `finally` restored the production bytes exactly. The restored four tests and five existing mouse-mode tests passed all three engines. The 24 focused native mouse tests passed. Native all-target/all-feature checks and warning-denied Clippy passed; focused WASM library and both test-target Clippy passed with all features. Exact workspace formatting hit Windows OS206; every complete member formatting check passed. Raw logs and pinned compiler/browser/native artifacts are retained in `results/mouse-builtin-methods`.

The full workspace, native OS, complete unchanged-source/browser suites and all-addon gates remain separate. Overall port equivalence remains NOT PROVEN.
