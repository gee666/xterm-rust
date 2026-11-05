# Recursive selection mouse events

`SelectionDom` registered its mousedown and document drag listeners as `FnMut` closures. The source handlers permit synchronous recursive DOM dispatch. An `onSelectionChange` listener can therefore select all again and dispatch a second mousedown before the first one returns. Rust's mutable callback rejected that second dispatch with `closure invoked recursively or after being dropped`.

The selection listeners now use the existing `DomListener::new_reentrant` adapter. No shared selection model, host boundary, error routing or native algorithm was replaced. DOM listeners still capture weak component state and retain the existing disposed checks and removal ownership.

## Reproduction and results

The new `wasm_terminal_selection` regression opens the exported Rust Terminal, writes `abcdefghij` through Rust, selects all, and installs an `onSelectionChange` callback. While the first mousedown clears the selection, the callback selects all and dispatches a nested mousedown at column four. A subsequent real DOM drag to column seven must select `efg`, with four selection-change events. There are no timing sleeps or synthetic selection-service replacements.

`scripts/selection-reentry-source.mjs` runs the same directed scenario through the original production Terminal bundle in real browsers. It verifies the bundle's embedded SelectionService source against the read-only TypeScript file and records both hashes. This is a new source-derived regression, not an unchanged upstream test.

Observed on Windows with Rust/Cargo 1.85.1, Node 24.15.0 and wasm-bindgen 0.2.100:

- The original Terminal returned `efg`, four events and a prevented nested mousedown in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4.
- Before the fix, the Rust Chromium regression failed. It reported the recursive-closure error and retained the select-all text instead of `efg`. Firefox and WebKit were not run before the fix.
- After the fix, the new real Terminal regression passed 1/1 in each engine. The existing SelectionDom component suite passed 8/8 in each engine before and after the fix.
- The unchanged original SelectionService suite passed 26/26 and its existing 4,297-case source corpus matched. Native selection targets passed 31 Rust test functions, including replay of 4,298 recorded cases counting the separate retained cache regression.
- The coordinate-model oracle passed all 18 original assertion bodies and 61,310 comparisons, including 20,000 seeded cases with seed `0x051ec710`. The service corpus uses seed `0x5e1ec710`.
- Scoped WASM library/test Clippy passed with warnings denied. Changed-file rustfmt checks and Node syntax checks passed.

Independent review accepted source commit `c5ed709`. The reviewer inspected the callback ownership and regression, verified retained WASM/binding/test-source hashes and original embedded source identity, and reran the unchanged 26-case source suite with the 4,297-case corpus, Node syntax, changed-file rustfmt and whitespace checks. It found no concrete problems. Native Cargo and browser execution were not independently rerun because the shared runtime allocation remained unapproved; the retained three-engine runtime results above are producer executions.

Concise reports, the before-fix failure, source hashes and logs are under `results/selection-service/reentry/`. The latest coordinate report is `results/selection-model/oracle.json`. Browser reports contain hashes and paths of the exact generated artifacts used at execution time. Disposable binaries are not retained after cleanup.

## Commands

Set `SOURCE_ROOT` to the read-only original tree containing the built `lib/xterm.mjs` and its source map. Set `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` when the existing dependencies and locked CLI are outside this checkout. `NODE_PATH` may point at the original Node dependencies for the unit oracle.

```sh
node scripts/selection-reentry-source.mjs
WASM_TEST=wasm_terminal_selection node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection node scripts/browser-host-check.mjs
node scripts/selection-service-oracle.mjs
node scripts/selection-model-oracle.mjs
cargo test --locked -j2 -p xterm-web --test selection_model --test selection_service --test selection_service_differential
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_terminal_selection --test wasm_selection -- -D warnings
```

The model oracle invokes Cargo internally. Set `CARGO_BUILD_JOBS=2` and allocate a compiler slot before running it. Keep build targets and temporary process files in the owned O: tree for this run.

This repair does not establish complete selection/link equivalence. Live JavaScript link object mutation, provider/cache getter reentry, raw word-separator calls, Number values outside the native selection representation, complete unchanged browser suites, and native Linux/macOS execution remain outside this scoped evidence. See `docs/selection-acceptance.md` and `docs/live-link-acceptance.md`. Full family closure remains unfinished.
