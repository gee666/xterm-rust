# Search highlight decoration phases

Task458 follows445. Full search and port equivalence remain NOT PROVEN.

## Contract and implementation

The unchanged `source_repo/addons/addon-search/src/SearchAddon.ts` method `_highlightAllMatches` clears old decorations, collects matches, and calls `SearchResultTracker.updateResults` before reading `searchOptions.decorations`. A truthy eligibility value causes a separate second read as the `createHighlightDecorations` argument. The manager receiver is captured between those reads. Either getter may throw or reenter, including reactivation on another terminal. The eligibility object is not the call argument.

`crates/xterm-addons/src/search/find.rs` now passes the retained option source through the highlight pass instead of creating an eager combined snapshot. It reads matching flags separately, stores results, reads decoration eligibility, captures the current manager, and reads the argument. All terminal/cache/manager `RefCell` borrows end before a host callback. Ordinary find selection timing from445 stays unchanged.

`SearchOptionsSource::highlight_decorations` preserves the argument separately from eligibility. Native typed records retain their existing API. A native absent second value uses a private `MissingDecorations` source which returns a meaningful error only when a result reads a color. Empty result sets still clear old highlights without reading colors. Browser `JsOptions` retains the second raw JavaScript value even when falsy. `JsDecorations` uses non-optional color property reads, so nullish arguments throw the source TypeError while false, zero, empty string, NaN and Symbol are boxed normally. No matching or terminal behavior delegates to JavaScript. No new dependency, unsafe code or lint suppression.

## Verification scope

`crates/xterm-web/tests/search_cases/highlight_options.rs` adds five native functions expanding 34 scenarios across both directions, match/miss, eligibility short circuit, distinct argument colors, getter errors, nested search and real cell mutation, nullish absence, matching failures, and manager capture during reactivation onto another real terminal.

`crates/xterm-web/tests/fixtures/search-highlight-options.js` contains184 directed cases. The same assertions run against unchanged source classes through `scripts/search-call-args-source.mjs` and against actual Rust/WASM public bindings in one added `wasm_search_addon` function. It covers two directions, match/miss, two late getter phases, returned argument identity, short circuits, six thrown-value identities, nested searches, seven raw falsy arguments, invalid terms, engine error cutoff, and public marker destinations on terminal reactivation. Function counts and expanded case counts are distinct.

## Observed results

Raw commands, logs, source manifests and artifact identities are under `results/search-highlight-options/`; `summary.json` pins the retained files. Windows Node24.15.0 and Rust/Cargo1.85.1 were used.

- Native search integration passed53 functions, none ignored or filtered. Five new functions expand34 scenarios.
- Shared source184 cases passed on each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4,552 directed source cases in total.
- Actual Rust/WASM passed16 functions per engine,48 total, none ignored or filtered. The184 shared cases are inside one function per engine, not extra test functions.
- Final scoped native and WASM Clippy with `-D warnings` passed. Changed Rust formatting passed. Exact `cargo fmt --all -- --check` failed with Windows OS206, retained in `full-fmt.log`. This is not a workspace quality pass.
- A native negative control replaced only `find.rs` with its accepted445 version while retaining the new trait, error definitions and tests. All five new functions failed, zero passed,48 were filtered. Production bytes were restored in `finally`, hash-verified, and all53 functions passed again.
- The initial new source fixture incorrectly expected the outer search to select `foo` after nested search had replaced its cached term and selection with `FOO`. All three source runs stopped at that assertion. Their exact reports are retained as `source-attempt1.json`; no completed case totals are available for that failed attempt. The fixture now asserts source expansion from the nested selection. No original source or production behavior changed to resolve that fixture error.
- All776 original reference hashes matched before and after. The inherited445 archive's52 summary pins were independently checked, without replaying its effects.

Reproduce from a prepared checkout with the existing Node dependencies and matching wasm-bindgen0.2.100 CLI:

```sh
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
SEARCH_CASES=search-highlight-options node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
```

The existing runners accept `SOURCE_ROOT`, `SOURCE_DEPENDENCIES`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` for installed tools outside the owned checkout. Use an approved O: target and scratch while in this swarm. Removed native/WASM binaries are provenance records, not retained deliverables.

Full347 retains repeated matching-option getters and mutable raw term methods. Full322 retains complete search/serialization coverage, unchanged suite closure, disposal/GC and host contracts. No new unchanged original Search E2E run, random corpus, full workspace Clippy/tests, interactive demo or native Linux/macOS evidence is claimed here. The source inspection hypothesis about selection host capture after reactivation is separate from the tested highlight-manager destination contract.
