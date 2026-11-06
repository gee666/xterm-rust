# Selection assignment after drag teardown

`SelectionService.setSelection` clears its model before removing drag listeners and assigns the requested start and length only after teardown returns. The previous Rust implementation assigned the target before firing its drag-stop event.

DOM teardown calls host methods. A replaced `document.removeEventListener` can inspect the selection, call another public selection method or throw a value. The source exposes an empty model during that callback. A throw prevents target assignment. A successful callback can leave a selection end or select-all flag that the outer method preserves. Replacing the entire model after teardown would lose those fields, so Rust writes only the start and length after the callback.

The change is in `crates/xterm-web/src/selection/service_actions.rs`. It does not hold a model or buffer borrow across drag-stop, refresh or selection-change callbacks.

## Verification scope

The eight method cases in `scripts/selection-teardown-methods.mjs` invoke the unchanged original methods with a real source SelectionModel. They cover observation, exact thrown-value identity, nested `setSelection` and all four combinations of reentrant selection-end and select-all mutations. The method observer replaces only DOM teardown and refresh delivery, not the assignment or selection-change logic.

Five native test functions in `crates/xterm-web/tests/selection_teardown.rs` cover the same cleared-state, failure, retained-field and nested-assignment contracts. The existing service/model/corpus assertions remain unchanged.

The shared public trace in `scripts/selection-teardown-trace.js` opens an actual Terminal, writes through its parser, starts a real DOM selection drag and intercepts either mousemove or mouseup listener removal. Nine actions at each interception point inspect state, throw an exact marker, select a nested target, select all, select lines or clear selection. Both the original bundle and actual Rust/WASM Terminal run this same trace. No private selection-service fixture or JavaScript terminal fallback is used. Browser fixture generation is explicit, and the source runner verifies exact embedded source-map text before execution.

The expected fixture is `crates/xterm-web/tests/fixtures/selection-teardown.json`. Custom method/public cases are not unchanged upstream test definitions. The retained original SelectionService baseline is 26 unchanged test definitions and 4,297 source operation/observation traces, including 4,096 randomized cases with seed `0x5e1ec710`. The native replay also retains the separate signed-cache counterexample.

## Commands

Use an isolated worktree, existing tool installations and explicit team compiler/browser grants. Set `SOURCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to the available original reference and locked tools. Cargo targets and `TEMP`/`TMP`/`TMPDIR` must use the authorized owned O: worktree.

```sh
node scripts/selection-teardown-methods.mjs
node scripts/selection-teardown-source.mjs
cargo test --locked -j2 -p xterm-web --test selection_teardown --test selection_service --test selection_model --test selection_service_differential
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_selection_teardown --all-features -- -D warnings
WASM_TEST=wasm_selection_teardown node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_selection node scripts/browser-host-check.mjs
```

## Current evidence

Source methods8 and the unchanged original SelectionService26 with its4,297-case corpus passed during preparation. Native and browser execution remain pending resource grants. Raw results will be retained under `results/selection-teardown` and the existing browser-host result paths.

This repair does not establish full selection or link equivalence. Arbitrary DOM teardown getter/order/repeated-removal and listener lifetime behavior, live option sampling, general public raw numeric representations, complete upstream/E2E coverage and native Linux/macOS runtime evidence remain separate full-family obligations under task321 and the project verification requirements.
