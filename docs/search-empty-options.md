# Empty search option phases

Task357 repairs an eager matching-option read in the callable SearchAddon. Before this change, `findNext('', options)` and `findPrevious('', options)` read `caseSensitive`, `regex` and `wholeWord` even when source search never matches a line. A throwing getter therefore prevented selection clearing and the after-search event.

The empty-string path now reads decoration eligibility first, clears the selection and decorations, reads decoration eligibility again for result notification, caches the empty term and emits after-search. It does not inspect matching flags unless an empty term is already cached and decorations are truthy. That case preserves the source comparison of the newly stored options object to itself, including strict equality, left/right getter order, short circuits and thrown value identity. A decoration getter can start a nested search without a Rust buffer, cache or state borrow spanning JavaScript.

Write/resize invalidation also checks the cached term before reading options. An empty cached term therefore does not evaluate decoration or matching getters when new output arrives. For nonempty cached terms, invalidation reads decoration eligibility only. Matching snapshots remain in the existing refresh callback and engine.

`SearchOptionsSource::decorations` and `flags_changed` separate these reads from matching snapshots. Existing typed native callers retain default implementations; the JS adapter implements raw property access and strict comparison. The engine, cache, selection and decoration services remain Rust implementations. There are no new dependencies or unsafe operations.

## Reproduction and checks

Use Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100. Set `CARGO_BUILD_JOBS=2`, and set TEMP, TMP and TMPDIR explicitly to an owned O: scratch directory. `SOURCE_DEPENDENCIES` can point at an existing unchanged reference checkout. `PLAYWRIGHT_MODULE` can point at that checkout's `node_modules/playwright/index.mjs`; `WASM_BINDGEN` can identify the matching installed CLI. No dependency junction is needed.

```sh
SEARCH_CASES=search-empty-options node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-web --test search_addon -- -D warnings
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt --all -- --check
```

The same `fixtures/search-empty-options.js` assertions run against unchanged source classes and the actual Rust Terminal/SearchAddon exports. Its54 directed cases per engine exercise both directions, falsy/truthy decoration values, poisoned matching getters, cached empty requests, strict object/NaN comparisons, flag short circuits, throwing first/final decoration reads, nested searches and a subsequent write. It adds no randomized cases. Original source files and assertions are not edited.

## Observed Windows results

| Check | Executed | Passed | Failed | Skipped/unrun |
| --- | ---: | ---: | ---: | ---: |
| Shared source-directed cases, three engines | 162 | 162 | 0 | 0 |
| Baseline Rust WASM functions, Chromium only | 1 confirmed | 0 claimed | 1 confirmed | 7 outcomes unreported |
| Fixed native search functions | 30 | 30 | 0 | 0 |
| Fixed Rust WASM functions, three engines | 24 | 24 | 0 | 0 |
| Shared empty-term cases inside fixed WASM functions | 162 | 162 | 0 | 0 |

The last row is contained in the24-function count, not an additional162 test functions. The eight WASM functions per engine also retain the earlier81-case callable-argument fixture. Browser versions were Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Scoped native and WASM warning-denied Clippy and exact full workspace formatting passed. A later lightweight run of `node --test scripts/browser-artifact-server.test.mjs scripts/upstream-baseline.test.mjs` passed7/7 runner helper tests. It does not add terminal behavior coverage.

The baseline uses unchanged producer files from main4a38b522 with only the new WASM regression fixture/test added. It fails the new function on the first poisoned matching getter, preserving the thrown `emptySearchGetterFailure` object. Eight functions were discovered, but the runtime abort prevents a summary. The other seven outcomes are unreported and are not counted as passes. Fixed production files were restored in a `finally` block before the passing run. The original source manifest verifies all776 entries both before and after the source run.

`results/search-empty-options/` retains source, baseline, final browser reports, native/lint logs and a hash-pinned summary. Browser/compiler binaries referenced by reports are temporary build outputs, not committed distribution artifacts. The bounded runners close their own browsers and servers. Native Windows results are not native Linux/macOS runtime evidence.

## Independent review

Task357 accepted commit05ae2d3. The isolated reviewer independently reran native30/30, scoped warning-denied Clippy and full workspace formatting. It verified all nine implementation/test hashes, all776 reference-manifest entries and retained baseline/fixed compiler, WASM and binding hashes. It inspected browser evidence without starting a new matrix. No tracked files changed during review.

## Canonical integration

The canonical helper integrated357 as mainbb6eada9 on base5ca9f593. It passed changed addon/web builds and all-target native tests, affected-crate/default workspace checks and applicable WASM library checks. The log contains491 passing tests across141 summaries, zero failures or ignored tests. These are the helper's scoped gates, not the final all-feature/release/platform matrix. All nine reviewed implementation/test hashes remained unchanged after rebase. Exact full formatting passed again after integration. The helper removed the private Cargo target. Browser reports remain the pre-batch scoped runs described above, not a fresh integrated browser matrix.

`results/search-empty-options/canonical.json` and `canonical.log` retain the receipt.

## Scope still open

This does not close raw-search347 or full-family322. Truthy non-string and boxed/overridden terms, term validity/error precedence, identity-cached object terms, per-line repeated matching getters, null matching options and refresh object spread still need source-phased matching. Matching-flag getters that replace the stored request through nested search remain part of347; this change verifies nested decoration getters, not that larger flag-replacement contract. The unchanged original81-case browser suite was not rerun for357. Its earlier344 receipt is separate; the fresh eight-function suite is not a substitute. Overall equivalence remains NOT PROVEN.
