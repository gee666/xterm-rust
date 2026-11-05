# Search browser verification

Task290 restores the search browser test build and removes repeated lowercase conversion during highlighting. These results do not prove complete xterm.js equivalence.

## Changes

- `wasm_search_addon.rs` uses `Function.apply` with an argument array. The locked js-sys version has no `call4` method. CSS comes from the tracked crate asset, not generated `browser/xterm.css`.
- Search cache entries lazily retain lowercase UTF-16 text. Plain case-insensitive searches share that text instead of converting a long wrapped line once per match. Replacing or invalidating an entry, including its 15-second expiry, releases the derived text. Case-sensitive and regex searches continue using the original line. No buffer borrow spans a JavaScript callback.
- The retained Jeremiah reverse-search iterator correction is recovered. `next_back` selects the same bounded final match without walking the whole prefix.
- Real DOM tests use Ricardo's accepted root decoration producer through87fbb03. This is a dependency, not a search-only substitute renderer.

The cache unit tests cover lazy reuse, contextual final sigma, expanding lowercase for İ, a lone surrogate, row replacement, signed row keys, expiry and explicit invalidation. Existing seeded fixtures retain their expected values and assertions.

## Observed results

Windows, Node24.15.0, wasm-bindgen0.2.100, Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Exact source/build hashes, commands and process outcomes are in `results/search-browser/`.

| Check | Executed | Passed | Failed | Ignored/skipped |
| --- | ---: | ---: | ---: | ---: |
| New cache unit tests | 2 | 2 | 0 | 0 |
| Four native search integration targets | 34 | 34 | 0 | 0 |
| Actual-Terminal WASM search tests | 18 | 18 | 0 | 0 |
| Unchanged original search browser suite | 81 | 81 | 0 | 0 |

The native unit command filters unrelated library tests. The four integration targets and both browser suites run every discovered test in their stated scope. All81 original browser cases execute, with zero unrun cases. Scoped Clippy passes with warnings denied. This is not the full workspace quality matrix.

The initial executable browser checkpoint passed5/6 real-Terminal tests per engine. The DOM test timed out because the root did not construct BufferDecorationRenderer. Task296 fixes that root omission and its pre-open lifecycle.

Before lowercase caching, the original browser run executed81 cases, passed80 and timed out one Firefox `findNext` case with more than1,000 matches. An isolated rerun timed out both corresponding Firefox cases. Diagnostics using the exact original data, including literal backslash-n/backslash-r and Unicode15-graphemes, measured4,935ms and5,144ms for the two highlight-producing searches in debug Firefox WASM. The original10-second timeout was not changed. Final runs pass81/81 with the same debug build profile and unchanged source assertions. These single diagnostic timings are not a release performance comparison or a speedup claim.

## Reproduce

Set `CARGO_BUILD_JOBS=2`. Use an owned temporary directory for `TEMP`, `TMP` and `TMPDIR`. On Windows, expose Git and Cargo on `PATH`. Install or reuse wasm-bindgen CLI0.2.100 and the supplied reference's Playwright dependencies. `PLAYWRIGHT_MODULE` can point to an existing compatible installation for the component driver. Original-suite execution needs `source_repo/node_modules`, and the browser driver must find the matching CLI through `WASM_BINDGEN` or `PATH`.

```sh
cargo test --locked -j2 -p xterm-core --lib search::cache
cargo test --locked -j2 -p xterm-core --test search_differential --test search_cache --test search_integration --test search_ring
cargo clippy --locked -j2 -p xterm-core --lib --test search_differential --test search_cache --test search_integration --test search_ring -- -D warnings
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
node scripts/e2e-upstream.mjs --suite addon-search --timeout-ms 180000
```

The original test, its original configuration and TestUtils match the launch reference byte-for-byte. See `reference-integrity.json`. No original implementation, assertion, test timeout or expected fixture was edited.

## Remaining scope and cleanup

Task322 still owns the full search/serialization mapping and raw term/options compatibility audit. Truthy non-string search terms remain restricted, and accessor-backed options need full source read/error-order comparisons. Task310 owns the missing browser SerializeAddon. These are implementation gaps, not external blockers. The passing search cases do not establish overview rendering, other addons, all upstream unit assertions, native Linux/macOS runtime evidence or full distribution equivalence.

Raw reports retain the paths used during execution and artifact hashes. Generated WASM binaries, copied artifacts, temporary fixture/config files and browser processes are disposable. Only concise JSON/log evidence is retained here. The owned reference dependency junction must be unlinked before removing the O: worktree so cleanup cannot traverse into the original reference.
