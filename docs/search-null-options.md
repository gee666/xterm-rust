# Null search matching options

Task412 repairs a bounded SearchAddon matching boundary under347. The browser binding previously discarded an explicit `null` request as though it were `undefined`. Source performs optional decoration reads at addon level, but its engine reads `searchOptions.regex` without optional chaining. A valid term with null options therefore throws TypeError, after obtaining and clearing selection and initializing the line cache. An invalid term still returns false without matching access. Undefined and other primitive options keep their ordinary property-access behavior.

The binding now retains null. The JS adapter distinguishes optional addon reads from non-optional matching reads and begins matching flags with a named `regex` access. This named access preserves the property label in WebKit's engine-generated TypeError too. Rust controls selection, cache initialization, matching, traversal, events, and results. Static JS helpers only implement property access; no terminal engine delegates to JavaScript. No dependency or unsafe code was added.

The portable addon captures and clears selection and initializes its cache before reading matching options. Every highlighting engine invocation still initializes the cache once. No state, buffer, cache, or selection borrow spans an options getter. A getter may select through a nested search and throw its original value; the outer failure does not clear that nested selection or emit its after/results events.

## Verification

Base source is main51112351598d2bf81bf252185981653eaf2d16a0. The run uses native Windows, Node24.15.0, Rust/Cargo1.85.1, wasm-bindgen0.2.100, Chromium148.0.7778.96, Firefox150.0.2, and WebKit26.4. All new build and scratch output is in the owned O: worktree. Existing reference dependencies and the matching wasm-bindgen CLI are reused without modifying them.

```sh
SEARCH_CASES=search-null-options node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt -p xterm-addons -p xterm-web -- --check
```

Set TEMP, TMP, and TMPDIR to owned O: scratch and CARGO_BUILD_JOBS=2. SOURCE_DEPENDENCIES points to the unchanged original checkout, PLAYWRIGHT_MODULE to its Playwright module, and WASM_BINDGEN to the installed matching CLI. The runners bound commands and close their servers and browsers. The shared directed fixture uses no sleeps or new random seed. All776 reference-manifest entries match before and after its source run.

| Check | Executed | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: | ---: |
| Shared unchanged-source directed cases, three browsers | 138 | 138 | 0 | 0 |
| Native addon test functions | 37 | 37 | 0 | 0 |
| Fixed WASM test functions, three browsers | 33 | 33 | 0 | 0 |
| Shared null-option cases inside three WASM functions | 138 | 138 | 0 | 0 |
| Old native producer negative control | 2 | 1 | 1 | 0 |

The138-case WASM subtotal is inside33 test functions, not138 additional functions. Existing callable argument, empty/invalid-term and delayed refresh assertions also execute. The46 new cases per engine cover both directions, present/absent matches, existing/no selection, null requests with eight invalid terms, seven other primitive request values, activation and before-event precedence, regex/caseSensitive getter exceptions, exact thrown identity, and nested real selection.

The old native producer fails the cleared-selection assertion. The old WASM producers fail the new null-TypeError assertion in Chromium, then the wasm-bindgen test runtime aborts with an externref-table error. The11 discovered functions in that attempt do not have a final summary and are not counted as11 executed tests. Exact producer bytes are restored in a finally block and hash-checked after both negative controls.

An intermediate fixed run passed11 functions each in Chromium and Firefox but failed one of11 in WebKit. The computed-key property helper produced a TypeError without the `regex` label. The named property read corrects that engine-visible detail. The fixture assertions are unchanged between that failure and the final three-browser pass. Raw failed and final reports remain under `results/search-null-options/`. Browser/compiler artifacts identified by those reports are temporary verification output, not distribution artifacts.

Scoped warning-denied Clippy and formatting pass. Exact `cargo fmt --all -- --check` still fails on Windows OS206 in this physical short worktree. This is retained, not relabeled as a full formatting pass or waived. Final integrated workspace quality remains304. This task supplies no Linux/macOS native runtime or current full-workspace proof.

## Remaining work

This is not completion of347 or full-family322. Valid boxed/overridden matching methods, per-line repeated flags, source short circuits between flag reads, mutable raw terms, raw lengths across phases, and option replacement during callbacks remain open. Decorated matching still snapshots request flags too early; this repair only claims the undecorated/null matching failure boundary. Refresh after an intervening cache clear still needs the source undefined-term representation audited. Full source-module mapping and randomized raw traces remain required. Overall equivalence is NOT PROVEN.
