# Live stored search flags

Task436 repairs the comparisons in upstream SearchState.didOptionsChange. Each comparison reads the stored property first, then the incoming property. The stored request is resolved again before each later property because either getter can synchronously replace it through a nested search. Comparing the incoming request with itself or holding one stored snapshot across all properties loses this behavior.

SearchOptionsSource.flags_changed now receives a live request resolver. The native default reads typed records in case-sensitive, regex, whole-word order and stops at the first mismatch. Absence after a nested search is SearchAddonError::MissingOptions for native clients. The browser adapter retains raw JsValue operands for strict equality and uses non-optional property access on the stored request. A null or undefined replacement during comparison must throw rather than be treated as an empty request. Custom JS thrown values retain their identity. The existing430 truthiness checks occur once before these comparisons, not after every getter.

The raw adapter follows the existing SearchTermSource Any pattern to identify JS option records. For a typed Rust record used with that adapter, its optional boolean flags map to JS undefined or boolean values. No browser dependency enters the shared addon, no callback runs under an addon/buffer RefCell borrow, and no JS terminal engine implements Rust matching or state. There is no new dependency or unsafe code.

## Verification scope

The shared464-case fixture checks actual exported Rust Terminal/SearchAddon bindings against the unchanged original implementation. It covers both outer and nested search directions, replacement in either operand before later regex/whole-word reads, eight replacement kinds including null and undefined, strict raw equality for16 value pairs per flag, six thrown-value kinds at each operand/property, short circuits, before/after/result events and selection state at failure. Two native test functions expand into16 directed scenarios over the real terminal services. Their typed callbacks test stored-first access, first mismatch and reentry that removes the request before the next property.

Source0cc7446b plus the result-event assertion a698c945 were tested on integrated main dcde3e9c. Before rebasing, git cherry verified all four430 commits already patch-equivalent. Those prerequisites were dropped, not replayed. Scoped native/WASM warning-denied Clippy and affected-file rustfmt pass. Aggregate fmt fails with Windows OS206 and its raw diagnostic is retained. No full-workspace pass is claimed.

| Check | Executed | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: | ---: |
| Shared unchanged-source cases, three engines | 1392 | 1392 | 0 | 0 |
| Final restored native search functions | 46 | 46 | 0 | 0 |
| Fixed Rust WASM functions, three engines | 42 | 42 | 0 | 0 |
| Final restored Rust WASM functions, three engines | 42 | 42 | 0 | 0 |
| Unchanged original Search E2E | 81 | 81 | 0 | 0 |
| Old-comparison native negative control | 2 | 0 | 2 | 0 |
| Old-comparison Chromium WASM control | 14 | 13 | 1 | 0 |

The1392 directed Rust cases are inside three of the42 functions in each matrix, not1392 extra functions. The original suite discovers and executes81 cases with zero failed, skipped or unrun cases across all original browser projects. It uses a newly built actual debug Rust distribution, two workers and no retries. The source and original-suite runs verify all776 reference-manifest entries before and after. An owned node_modules junction supplies existing dependencies for the original runner and is unlinked in finally. No original assertion changes.

Negative controls replace only the native default or browser flag-comparison body with the historical self/self algorithm, retaining the new resolver signature so the targeted old behavior compiles. They are not exact whole historical revisions. Both native regression functions fail, with44 other functions filtered out. The old browser producer reports13 passes and one failure. Its new trace is incoming.caseSensitive, incoming.caseSensitive, incoming.regex rather than stored.caseSensitive, incoming.caseSensitive. Both overridden files are restored in finally and hash-verified. The full fixed native suite and three-engine WASM matrix pass again after restoration. Browser versions are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.

Raw results, bounded command records, source/fixture/artifact hashes and concise logs are in results/search-live-stored-flags. The summary pins the owned source and evidence. No new random corpus is claimed. Built-in null/undefined failures assert TypeError class and failure phase, not exact engine-specific stack text; custom thrown values assert identity.

## Independent review

Pending isolated review. No acceptance is claimed yet.

## Reproduction

Use Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100. Set CARGO_BUILD_JOBS=2, a regular private CARGO_TARGET_DIR and TEMP/TMP inside the owned O: worktree. SOURCE_DEPENDENCIES names the supplied reference checkout with existing Node dependencies. PLAYWRIGHT_MODULE names its installed playwright/index.mjs and WASM_BINDGEN names the matching CLI. Source files are copied physically and checked against the retained reference manifest, not changed for these tests.

```sh
SEARCH_CASES=search-live-stored-flags node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
cargo fmt --all -- --check
```

The browser harness discovers14 test functions per engine, one containing the464 directed cases. Expanded cases and test functions must not be added together as if they measured the same thing. Existing callable/empty/invalid/null/refresh/falsy-request fixtures remain intact. Generated compiler/binding/distribution paths in reports are verification provenance, not retained deliverables.

## Remaining work

Task347 still owns raw term methods and mutable length, matching-phase repeated flags and short circuits. Task322 still owns complete search/serialization source mapping and differential closure. This scoped repair does not establish native Linux/macOS runtime evidence, full workspace quality, complete browser/demo behavior or speed/size/LoC comparisons. Full equivalence is NOT PROVEN.
