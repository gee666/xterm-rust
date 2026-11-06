# Selection assignment after drag teardown

`SelectionService.setSelection` clears its model before removing drag listeners and assigns the requested start and length only after teardown returns. The previous Rust implementation assigned the target before firing its drag-stop event.

DOM teardown calls host methods. A replaced `document.removeEventListener` can inspect the selection, call another public selection method or throw a value. The source exposes an empty model during that callback. A throw prevents target assignment. A successful callback can leave a selection end or select-all flag that the outer method preserves. Replacing the entire model after teardown would lose those fields, so Rust writes only the start and length after the callback.

The change is in `crates/xterm-web/src/selection/service_actions.rs`. It does not hold a model or buffer borrow across drag-stop, refresh or selection-change callbacks.

## Verification scope

The eight method cases in `scripts/selection-teardown-methods.mjs` invoke the unchanged original methods with a real source SelectionModel. They cover observation, exact thrown-value identity, nested `setSelection`, all four combinations of reentrant selection-end and select-all mutations, and mutation followed by failure. The method observer replaces only DOM teardown and refresh delivery, not the assignment or selection-change logic.

Five native test functions in `crates/xterm-web/tests/selection_teardown.rs` cover the same cleared-state, failure, retained-field and nested-assignment contracts. The existing service/model/corpus assertions remain unchanged.

The shared public trace in `scripts/selection-teardown-trace.js` opens an actual Terminal, writes through its parser, starts a real DOM selection drag and intercepts either mousemove or mouseup listener removal. Nine actions at each interception point inspect state, throw an exact object or Symbol marker, select a nested target, select all, select lines or clear selection. The nested, select-all and select-lines actions also run with a subsequent throw, preserving their partial state. Both the original bundle and actual Rust/WASM Terminal run this same trace. No private selection-service fixture or JavaScript terminal fallback is used. Browser fixture generation is explicit, and the source runner verifies exact embedded source-map text before execution.

The expected fixture is `crates/xterm-web/tests/fixtures/selection-teardown.json`. Custom method/public cases are not unchanged upstream test definitions. The retained original SelectionService baseline is 26 unchanged test definitions and 4,297 source operation/observation traces, including 4,096 randomized cases with seed `0x5e1ec710`. The native replay also retains the separate signed-cache counterexample.

## Commands

Use an isolated worktree, existing tool installations and explicit team compiler/browser grants. Set `SOURCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to the available original reference and locked tools. Cargo targets and `TEMP`/`TMP`/`TMPDIR` must use the authorized owned O: worktree.

```sh
node scripts/selection-teardown-methods.mjs
node scripts/selection-service-oracle.mjs
node scripts/selection-teardown-source.mjs
cargo test --locked -j2 -p xterm-web --test selection_teardown --test selection_service --test selection_model --test selection_service_differential
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_selection_teardown --all-features -- -D warnings
WASM_TEST=wasm_selection_teardown node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_selection node scripts/browser-host-check.mjs
```

## Current evidence

Eight source method cases and the unchanged original SelectionService suite passed, with 26 test definitions and 4,297 operation/observation traces. The native focused command passed 36 test functions before and after the control. Native xterm-web all-target/all-feature tests passed 424 functions across 178 summaries, with no failed, ignored, measured or filtered tests. Strict native Clippy passed. Restoring the exact main0442 production file made all five new native functions fail. The fixed file was restored in `finally`, its SHA checked, and the 36-function focused run passed again. Full workspace formatting failed with Windows OS206; xterm-web member formatting passed. These are scoped Windows/MSRV checks, not the final stable/all-platform workspace matrix.

The unchanged original Terminal passed all 18 shared public cases in each Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 with no page errors. Exact source-map content and input hashes were checked before and after the matrix. The original-source phase took 9.01 seconds including browser startup and cleanup. Actual Rust/WASM execution remains pending grants. Raw results are under `results/selection-teardown`; Rust browser receipts will also use the existing browser-host result paths.

This repair does not establish full selection or link equivalence. Arbitrary DOM teardown getter/order/repeated-removal and listener lifetime behavior, live option sampling and geometry after teardown reentry, general public raw numeric representations, complete upstream/E2E coverage and native Linux/macOS runtime evidence remain separate full-family obligations under task321 and the project verification requirements.
