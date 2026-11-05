# Search selection decoration access

Task445 separates ordinary engine matching flags from the later selection decoration getter. Upstream SearchAddon evaluates `searchOptions?.decorations` after `findNextWithSelection` or `findPreviousWithSelection` returns, even for a miss, and before reading internal `noScroll` or mutating the selected result. Rust previously read decorations while snapshotting matching flags, before the engine could finish or throw.

`SearchOptionsSource::matching_options` supplies the engine flags. Its default preserves typed Rust records. Browser `JsOptions` overrides it without accessing decorations. After the engine returns and releases terminal/cache borrows, the addon reads selection decorations, then internal scroll options, then selects. The original combined `read` operation remains for highlight snapshots and native records. This scoped fix does not close repeated per-line flags, matching short circuits, boxed terms or mutable raw term methods in task347. No new dependency, unsafe code or JavaScript terminal engine is introduced.

The change is based on integrated main3aa04692 plus accepted436. The five436 prerequisites were rebased privately, not replayed onto main. The new source commit is884a04ac. Canonical integration must pick that445 commit only after436, not the rebased private prerequisite chain.

## Verification

| Check | Executed | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: | ---: |
| Native search functions after restoration | 48 | 48 | 0 | 0 |
| Shared unchanged-source directed cases across three engines | 1080 | 1080 | 0 | 0 |
| Fixed Rust WASM functions across three engines | 45 | 45 | 0 | 0 |
| Final restored Rust WASM functions across three engines | 45 | 45 | 0 | 0 |
| Unchanged original Search E2E across all original projects | 81 | 81 | 0 | 0 |
| Old-timing native negative control | 2 | 0 | 2 | 0 |

Each browser executes360 directed cases inside one of15 WASM test functions. Do not add expanded case counts to function counts. Both native regression functions expand to10 directed scenarios over real terminal/selection/decoration services. Existing46 native functions and14 browser functions remain intact. The fixture covers both directions, matches/misses, existing/no selection, returning/throwing/reentrant decoration getters, six thrown-value identities including undefined and NaN, noScroll failures, engine failures and invalid-term short circuits. Initial eligibility and final result decoration reads remain distinct. Native callbacks also mutate real cells after nested search to prove borrow release, and verify that the late returned decoration is installed.

Native and WASM scoped warning-denied Clippy pass. Every changed Rust file passes rustfmt. The exact required `cargo fmt --all -- --check` command still fails with Windows OS206, with its raw diagnostic retained. This is not a full workspace quality pass. Tools are native Windows Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100. Browsers are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. No new randomized corpus or Linux/macOS run is claimed.

The unchanged original suite discovers and executes81 tests with zero failed, skipped, unrun or retried cases, two workers and a freshly built debug Rust distribution whose source/artifact identity is verified by the existing runner. All776 reference entries are intact before/after. An owned temporary node_modules junction supplies existing dependencies and is unlinked in finally. No original source or assertion changes.

## Negative control and retained failure

The negative experiment restores the historical pre-engine decoration snapshot in find.rs only. The new trait method remains compiled but is unused on that path. Both native regression functions execute and fail. In Chromium,15 functions are discovered and the new fixture reports the exact phase mismatch `before,d1,d2` versus expected `before,d1,d2,scroll,d3,after`. The asynchronous wasm-bindgen panic hook then aborts without a final test summary. Browser passed/failed totals are therefore unreported, not15 failures or a complete negative matrix. The report retains the console assertion, stack, compiler/binding hashes and absence of summary.

Two experiment-driver assertions were corrected or diagnosed. The first expected exit1 rather than Cargo's101 despite a valid2-test failure. The second assumed browser totals existed after the actual phase assertion and panic-hook abort. Neither changed production behavior. Each finally block restored find.rs and verified its SHA-256. After restoration the complete native48 and three-browser45 suites pass again. Original81 passes after the final restore.

Durable results and source/fixture/evidence hashes are in `results/search-selection-options/summary.json`. Temporary paths in reports identify verification artifacts, not distributed deliverables. Independent review accepted445 atc0596614 after correcting one normalized Markdown checksum. The reviewer independently ran the pinned retained native executable with48passed, checked52pins, all776reference entries, changed Rust formatting, JavaScript syntax and final WASM/compiler/bindings/CSS hashes. It found no behavioral defect. No independent rebuild, Clippy or browser rerun was authorized or performed. Review transcription is retained in review.txt; owned cleanup follows acceptance.

## Reproduction

Set CARGO_BUILD_JOBS=2, a regular private CARGO_TARGET_DIR and TEMP/TMP under the owned O: worktree. SOURCE_DEPENDENCIES names the reference with installed Node dependencies; PLAYWRIGHT_MODULE names its playwright/index.mjs and WASM_BINDGEN the matching0.2.100 CLI. Use the unchanged reference copy checked against docs/baseline/reference-manifest.json.

```sh
SEARCH_CASES=search-selection-options node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
node scripts/e2e-upstream.mjs --suite addon-search --timeout-ms 180000
cargo fmt --all -- --check
```

The original runner needs existing Node dependencies visible at the private reference's node_modules path. Unlink an owned dependency junction before any whole-worktree removal. Full tasks322/347, complete source mapping, full original suites/demo matrices, stable quality gates and native Windows/Linux/macOS proof remain open. Full equivalence is NOT PROVEN.
