# Delayed search refresh options

Task369 fixes the options copy used by SearchAddon after write or resize. Source clears the cached term, then calls findPrevious with `{...lastSearchOptions, incremental:true}` and `noScroll:true`. Rust previously read a typed snapshot before clearing the cache. That skipped unrelated own getters and symbol keys, read inherited/non-enumerable flags, and left the old cache visible during copy callbacks.

`SearchOptionsSource::for_refresh` now separates this copy from matching reads. Native requests copy their typed options and set incremental. The JS adapter performs only the language-level object spread in a static imported helper. It retains the copied raw request for the normal Rust search path. Matching, navigation, results, selection, decorations and timer ownership remain Rust. No dependency or unsafe code was added.

The cache clears before copying options. No addon-state borrow spans the copy, so getters can search recursively, dispose the addon or throw their original JS value. A copied getter becomes a plain data property. Later refreshes use the copy rather than re-reading the original getter. The spread includes own enumerable symbol keys, honors numeric/string/symbol enumeration order, ignores inherited and non-enumerable keys, and checks each live descriptor after snapshotting the keys.

## Verification

Use Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100. Set `CARGO_BUILD_JOBS=2`, and TEMP/TMP/TMPDIR to owned O: scratch. Point `SOURCE_DEPENDENCIES` at the existing unchanged reference checkout and `PLAYWRIGHT_MODULE` at its Playwright module. `WASM_BINDGEN` selects the matching CLI. Source-only dependencies are reused, not installed again.

```sh
SEARCH_CASES=search-refresh-options node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt --all -- --check
node scripts/check-search.mjs
node scripts/e2e-upstream.mjs --suite addon-search --timeout-ms 180000
```

The last two runners require reference dependencies under the physical reference copy. This run created one owned `source_repo/node_modules` junction and unlinked it in a finally block after both commands. The directed source/WASM runners use explicit dependency paths. All browser runners bound execution and close browsers/servers. The fixture intercepts only the 200ms debounce timer and invokes it explicitly. It does not sleep or change source assertions/timeouts.

The shared fixture has14 directed cases per engine. It covers write and resize refresh, own getter/symbol order before search events, deleted/new keys during copying, prototype/non-enumerable flags, proxy trap order, five throwing copy operations with exact error identity, cache clearing before exceptions and reentry, replacement of accessors by copied data, and disposal during copying. Three native virtual-clock tests cover the shared contract and independent incremental snapshot. No new randomized cases were added.

| Windows check | Executed | Passed | Failed | Skipped/unrun |
| --- | ---: | ---: | ---: | ---: |
| Shared unchanged-source cases, three engines | 42 | 42 | 0 | 0 |
| Native addon functions | 35 | 35 | 0 | 0 |
| Actual Rust WASM functions, three engines | 30 | 30 | 0 | 0 |
| Unchanged source SearchEngine/Cache unit assertions | 86 | 86 | 0 | 0 |
| Portable core search functions | 34 | 34 | 0 | 0 |
| Unchanged original Search browser suite | 81 | 81 | 0 | 0 |

The42 shared cases are inside three of the30 WASM functions, not42 extra Rust functions. Existing callable-argument, empty-option and invalid-term fixtures also rerun. Source86 is JavaScript baseline execution, not86 Rust tests. The retained core corpus matches2200 engine calls,6116 wrapped translations and1024 state rows, using existing seed0x5ea2c001. Browsers are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Exact full formatting and both scoped warning-denied Clippy commands pass.

The negative control restored the two old producers from baseaf04745 while keeping new tests. Chromium catches the missing own getters: actual `[before,after]` versus expected `[1,2,extra,incremental,Symbol(extra),before,after]`. The WASM test panic aborted without final counts. Its ten discovered functions are not reported as ten executed functions or nine passes. Producer bytes were restored in finally, then the final three-engine matrix passed. The baseline hashes/report and final source/WASM/native/original-suite reports/logs are in `results/search-refresh-options/`. `summary.json` pins producer and retained evidence hashes. Compiler/WASM/binding paths in raw receipts are temporary verification outputs, not delivered distribution artifacts.

## Independent review

Task369 accepted d69d5ac against baseaf04745. The isolated reviewer inspected source copy/cache ordering, callback borrow boundaries and both adapters. It independently checked both JavaScript syntax checks, all24 summary hashes, retained WASM artifact hashes, both negative-control producer hashes and all776 reference-manifest entries. It inspected the native/source/WASM/original-suite receipts without rerunning Cargo or browsers because the local grant had been released. No changes were requested. Clean rebase onto main5051454 preserves the source/evidence hashes as1affb84 plus intent receiptbd2accf.

## Remaining work

This closes only delayed refresh copying within347. Valid boxed/overridden string matching, null matching options, repeated per-line flag reads, flag-getter reentry replacing stored options, and raw length/method mutation between matching phases remain347/322. Refresh after an intervening cache clear still needs the source undefined-term representation audited. Complete source-module mapping, full raw-option randomized coverage, native Linux/macOS and full workspace/end-to-end evidence are not supplied by this task. Overall equivalence is NOT PROVEN.
