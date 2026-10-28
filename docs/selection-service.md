# Selection service port

The native service is under `crates/xterm-web/src/selection/`. `service.rs` owns coordinate state and synchronous events, `service_actions.rs` owns selection commands, `service_pointer.rs` owns click/drag updates, `word.rs` finds wrapped words using independent UTF-16 and cell-column indexes, and `text.rs` extracts normal/column selections through the real Rust buffer. `pointer.rs` contains modifier and drag-speed rules. Every file retains upstream MIT attribution.

The model accepts signed integer columns and lengths. This is required by actual search results inside combined cells, not just malformed public input. The first implementation used unsigned columns, so task69 was reopened and its source oracle extended before integration. No negative coordinate is clamped merely to fit a Rust type. Arithmetic and allocation failures return typed errors.

Text extraction retains JavaScript's cache-sensitive canonical translation path. Negative column reads yield blank cells as the source's undefined typed-array reads do. During this change, source trace269 exposed an extra blank before a wrapped continuation because explicit end arguments bypassed the canonical cache. The reproducer is retained at `results/selection-service/counterexamples/signed-cache-269.tsv`. The fix preserves absent start/end arguments for ordinary translation and uses explicit signed handling only where needed.

## Tests and accounting

```text
node scripts/selection-service-oracle.mjs
cargo test -p xterm-web --test selection_model --test selection_service --test selection_service_differential
cargo clippy -p xterm-web --test selection_model --test selection_service --test selection_service_differential --example selection_oracle -- -D warnings
```

The Node oracle uses the original installed TypeScript compiler to load read-only source modules. `scripts/source-loader.mjs` is reusable by other scoped oracles. Imports resolve to the actual original classes and Chai assertions; source files and assertion bodies are not edited. All 26 original SelectionService unit cases passed. As upstream itself does, its test subclass disables DOM interaction and refresh scheduling. This is unit baseline evidence, not a browser baseline.

Instrumentation delegates each original method and getter unchanged, recording initial buffer/cell/model state and final values. There are 201 operation/observation traces from the original suite. Membership and force-selection assertions have additional direct native tests. Another 4,096 seeded cases use actual original BufferLine, buffer and SelectionService classes, with wide cells, combined/astral strings, lone surrogates, NBSP, wrapping, signed ranges and normal/column text. Seed is `0x5e1ec710`.

The Rust fixture replay reconstructs real Rust BufferService/BufferLine state and compares raw model fields, final endpoints, hasSelection and full UTF-16 selected text. The pre-word-drag generation of 4,297 traces passed. The latest source fixture adds word-drag operations, and the retained cache counterexample is now replayed separately, giving 4,298 current native cases. This expanded replay has not executed during the coordinated build pause. Six additional executed native tests cover membership, modifier precedence/matrix, exact drag-speed bounds, refresh coalescing, callback order, reentrant model access and disposal. Together with the 23 SelectionModel tests, the combined command passed 30 Rust test functions, not 4,297 upstream tests. The warning-denying scoped Clippy command passed. A seventh native regression for rejected frame scheduling was added during the disk pause and is not yet executed. The expanded word-drag corpus has been regenerated and checked against the source, but awaits native execution after that pause. Raw commands, counts and limits are in `results/selection-service/verification.json`, with source hashes and original per-case outcomes in `oracle.json`.

To regenerate a fixture only after inspecting a real source change:

```text
node scripts/selection-service-oracle.mjs --generate
```

The normal command checks rather than silently overwrites the committed fixture. The 4,297-line TSV exceeds the normal source line limit because it is generated source-derived test data. Keep retained counterexamples even after a passing rerun.

Task273 found that the separate historical cache fixture on main contained only `undefined`, so case4297 failed its 15-field schema assertion before testing Rust behavior. Batu's validated `--legacy-cache-case` regeneration restores the original five-operation seeded case269. It checks the schema, known input and raw model before writing. The regenerated fixture matches Batu24c30bb byte-for-byte. Both debug and release replays now pass all4,298 cases, with23 model and7 service test functions also passing. No expected observation or native logic was weakened. Source26 assertions and the current4,297-case corpus match unchanged. Reproduction commands and limits are in `results/selection-service/fixture-repair.json`. This scoped native repair does not establish browser selection equivalence.

## DOM and terminal integration

`selection/dom.rs` owns DOM listeners and subscriptions. Its companion modules own mousedown/move/up/right-click behavior, requestAnimationFrame redraw coalescing and drag intervals. Callbacks capture weak state; disposal removes document-level listeners, cancels jobs and releases subscriptions. Resize, buffer activation, trims and actual user-input events update the same native SelectionService. Trims avoid borrowing the buffer while its mutation is dispatching.

`terminal/selection_services.rs` implements SelectionHost for WeakBrowserServices. It uses the opened terminal's shared selection, buffer, core, linkifier and renderer. Link selection takes priority over word lookup. Linux primary selection sets the actual textarea with lossless UTF-16, focuses and selects it. Alt-click invokes the Rust MoveToCell implementation and the native input path, without locally echoing text. Scroll requests use the central browser viewport path rather than bypassing smooth scrolling.

The coordinator must retain an Rc<SelectionDom> and clone it before invoking callbacks, releasing its RefCell borrow first. Accessibility, search, input and public APIs must use the same Rc<SelectionService>. No second selection model is created for an addon. Public and internal input-scroll events both refresh selection and synchronize the viewport.

Two concrete browser-boundary gaps are tracked separately, rather than hidden in a passing native suite. Criterion420/task189 covers JavaScript negative-zero endpoint identity and finite integer Numbers beyond i64. Criterion424/task192 covers source-timed wordSeparator.indexOf calls, dynamic method/error behavior and reentrant callbacks. The current typed UTF-16 separator path does not establish those JavaScript object contracts.

`tests/wasm_selection.rs` contains component DOM assertions for document drag, redraw coalescing, selected text, word/link priority, user input, trimming, buffer activation, modifier/column policies, alt-cursor callbacks and cleanup. These tests have not yet executed. The latest attempted WASM build failed in concurrent root protocol/input/host integration, retained in `wasm-build.log`. Disk pressure then paused builds across the swarm. No Chromium, Firefox or WebKit pass is claimed here. Full Terminal E2E and all remaining edge cases in `docs/selection-acceptance.md` remain required before task70/34 acceptance.
