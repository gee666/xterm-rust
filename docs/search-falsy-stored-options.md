# Falsy stored search options

Task430 repairs the initial short circuits in SearchState.didOptionsChange. Source assigns the incoming request before reading decoration eligibility, but that getter can start a nested search and replace the stored request again. The outer search must inspect the current stored request, not assume it is still its incoming argument.

When the stored request is falsy, source returns true before comparing flags. When the stored request is truthy and the incoming request is falsy, it returns false before comparing flags. Cache absence and a different cached term short circuit earlier still. Primitive falsy incoming requests can reach this path through inherited decoration getters.

SearchOptionsSource now exposes infallible request truthiness. Typed Rust records default to true; the browser adapter checks the actual retained JsValue without invoking getters or coercion. The portable search checks the current stored handle after decoration access, with no state borrow held across its truthiness method. It preserves the existing flag comparison for truthy requests. No dependency, unsafe code, lint suppression or JavaScript terminal-engine delegation is added. Matching, result tracking and decoration state remain Rust.

## Reproduction and verification

The source commit is f6cc79de on main785a7781, which already contains accepted412 and422. This repair does not replay either. Run with Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100 on native Windows. All new targets, browser artifacts and scratch stay under the owned O: worktree. Set CARGO_BUILD_JOBS=2 and TEMP, TMP and TMPDIR to its tmp directory. SOURCE_DEPENDENCIES points to the unchanged original reference checkout; PLAYWRIGHT_MODULE names its installed playwright/index.mjs. WASM_BINDGEN names the matching installed CLI.

```sh
SEARCH_CASES=search-falsy-stored-options node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt --all -- --check
node scripts/e2e-upstream.mjs --suite addon-search --timeout-ms 180000
```

The shared84-case fixture runs the same assertions on unchanged source and actual Rust bindings. It covers both outer and nested directions, eight falsy replacement values, six truthy controls, undefined/null distinction, thrown-value identity, before/after events, nested selection, earlier cache-absence/changed-term short circuits, primitive prototype getters, and truthy stored replacement during a falsy incoming request. Every modified prototype descriptor is restored in finally. Four new native functions exercise both branches, nested replacement, real highlight creation and the unchanged truthy comparison failure. No new randomized corpus is claimed.

| Check | Executed | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: | ---: |
| Shared unchanged-source cases, three engines | 252 | 252 | 0 | 0 |
| Native search functions after restoring fixed code | 44 | 44 | 0 | 0 |
| Fixed actual Rust WASM functions, three engines | 39 | 39 | 0 | 0 |
| Unchanged original Search E2E, all original projects | 81 | 81 | 0 | 0 |
| Old-find native negative control | 4 | 1 | 3 | 0 |

The unchanged original Search suite runs against a real debug Rust browser distribution with source/build identity verification, two workers and no retries. All81 discovered cases execute and pass with zero skipped or unrun cases. The runner closes its server. An owned node_modules junction under the physical reference copy points to unchanged installed dependencies only for that run and is unlinked in finally. Its exact Playwright report and build identity are retained.

The252 directed WASM cases are inside three of the39 functions, not252 extra functions. Each engine executes13 functions, including the earlier raw-argument, empty/invalid/null-option and delayed-refresh fixtures. Browser versions are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. All776 reference-manifest entries match before and after the source run. Native and WASM scoped warning-denied Clippy and affected-file formatting pass.

The negative controls replace only find.rs with its exact main785a7781 bytes while retaining the additive truthiness methods and new tests. The old native producer executes four new functions, with three failing on the unwanted comparison and one unchanged truthy control passing. The old Chromium WASM producer fails the new thrown-sentinel identity assertion, then the test runtime aborts with an externref-table error. It discovers13 functions but has no final summary; this is not counted as13 executed failures. The raw mismatch and runtime error are retained. Both controls restore fixed find.rs in finally and verify its hash. The fixed native44 and WASM39 are rerun after restoration.

Raw reports, compiler and binding hashes, source fixture hashes, bounded command records and concise logs are retained in results/search-falsy-stored-options. The build/compiler/binding binaries named in reports are temporary verification artifacts, not delivered distribution artifacts. The scoped reports do not establish final integrated workspace quality or native Linux/macOS runtime evidence. Exact cargo fmt --all -- --check fails with Windows OS206 and its raw diagnostic is retained, not relabeled as a full formatting pass.

## Independent review

Task430 was accepted at sourcef6cc79de plus evidence4af9d7b6 against785a7781. The isolated reviewer independently reran native44, scoped warning-denied native Clippy and affected-file formatting, all passing. It validated all46 then-pinned hashes, old/fixed compiler and binding artifacts, original-suite build artifacts, and776 reference-manifest entries. It inspected raw browser, original-suite and negative-control receipts without launching browsers. No files changed and no concrete findings were raised. The retained review transcription adds the47th final pin after review.

## Remaining work

Full347 and322 stay open. Truthy option replacement during each individual comparison getter, matching-phase repeated flags and short circuits, valid boxed/overridden search terms, mutable term lengths/methods, complete source-unit mapping and seeded raw traces still need implementation and proof. No current full-workspace, native Linux/macOS or complete demo matrix is claimed. Full equivalence is NOT PROVEN.
