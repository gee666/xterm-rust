# Raw invalid search terms

Task363 corrects callable SearchAddon argument validation. The previous binding converted falsy values to one empty UTF16 string and rejected every truthy non-string before checking activation or firing before-search. Source accepts many of those values as invalid terms and returns false. It reads `!!(term && term.length > 0)` during highlighting and selection, after the before-search event and initial decoration getter.

The native addon now retains one `SearchTermHandle` as its cache authority. `SearchTermSource` separates validity, raw truthiness, identity and matching text. Native UTF16 callers keep their existing methods and borrow matching text without another allocation. The JS adapter retains the raw argument, uses JavaScript property/relational semantics for validity, and uses strict equality for cache comparisons. Undefined remains the source cache's absence sentinel. NaN does not equal itself. Other invalid terms, including null, numbers, symbols, empty arrays and empty boxed strings, retain their actual identity.

The common search path reads decoration eligibility, checks raw cache identity and validates at the source highlight/find points. Invalid terms clear selection and decorations without inspecting engine flags or internal noScroll, then read decoration eligibility again, notify results, cache the raw term and fire after-search. Cached same-term highlighted searches still perform the source option comparison. Write invalidation tests cached raw truthiness rather than UTF16 length. A truthy invalid object therefore reaches the decoration eligibility getter on a later write, while a falsy cached term does not.

No addon-state, buffer or cache borrow spans a length getter or its numeric coercion. Such callbacks can throw their original JS value or start nested searches. Engine traversal, matching, results, selection and decorations remain Rust. The static JS helper only evaluates the source argument validity expression. There are no dependencies, unsafe operations or JS terminal-engine delegation.

## Commands and evidence

Use Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100. Set `CARGO_BUILD_JOBS=2` and TEMP, TMP and TMPDIR to the same owned O: scratch directory. `SOURCE_DEPENDENCIES` and `PLAYWRIGHT_MODULE` can point to existing original dependencies, while `WASM_BINDGEN` names the matching installed CLI.

```sh
SEARCH_CASES=search-invalid-terms node scripts/search-call-args-source.mjs
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
node scripts/e2e-upstream.mjs --suite addon-search --timeout-ms 180000
cargo fmt --all -- --check
```

The E2E runner needs reference dependencies under its physical reference copy. This run used one explicitly owned node_modules junction to the unchanged original dependencies, unlinked immediately after the runner. The directed source and WASM runners need no junction. All runners bound their work and close their own browsers and local servers.

`fixtures/search-invalid-terms.js` has240 directed cases per engine. The same assertions run on unchanged source classes and actual Rust exports. They cover both directions,22 invalid raw values, repeated raw cache identity,10 invalid length values, getter/coercion throws, Symbol length errors, nested search, differing invalid objects, raw write invalidation, pre-activation precedence, before-search exceptions and before-search mutation. There are no new randomized cases. Original source/assertions/timeouts are not modified. All776 reference-manifest entries pass before and after the source run.

| Windows check | Executed | Passed | Failed | Skipped/unrun |
| --- | ---: | ---: | ---: | ---: |
| Shared source-directed cases, three engines | 720 | 720 | 0 | 0 |
| Negative-control WASM functions, Chromium | 9 | 8 | 1 | 0 |
| Fixed native search functions | 32 | 32 | 0 | 0 |
| Fixed Rust WASM functions, three engines | 27 | 27 | 0 | 0 |
| Shared invalid-term cases within fixed WASM functions | 720 | 720 | 0 | 0 |
| Unchanged original Search browser suite, all original projects | 81 | 81 | 0 | 0 |

The720-case WASM subtotal is inside27 functions, not720 additional test functions. Nine functions per engine retain the earlier81-call-argument and54-empty-term shared fixtures too. Browsers are Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Native and WASM scoped warning-denied Clippy and exact full formatting pass. Native Windows evidence is not Linux/macOS runtime evidence or the full workspace quality matrix.

The negative control restores all six old producer files from mainbb6eada with only the new regression test and fixture added. It fails on repeated undefined caching. Old code collapses undefined into a cached empty string and wrongly invokes matching flag getters. The runner reports eight other functions passed and one failed, unlike the earlier task357 baseline that aborted without a summary. Production files were restored in a finally block before final verification. The raw report and baseline producer hashes remain under `results/search-invalid-terms/`.

Raw source, baseline, final WASM and unchanged-suite reports, logs and hash-pinned summary are in that directory. Compiler/WASM/binding binaries referenced by receipts are temporary verification outputs, not delivered distribution artifacts. They remain only through independent review and are then removed with owned temporary work.

## Independent review

Task363 accepted source91f8106 and evidenceba4705b/b910edf against basea2a1252. The isolated reviewer independently reran native32/32, scoped warning-denied Clippy and exact full formatting. It verified53 hashes across implementation, evidence, baseline producers, reference files, runners and retained browser artifacts. It inspected the720-case source/Rust receipts and unchanged81/81 suite without launching browsers. No files changed during review and no concrete problems were found.

## Canonical integration

The canonical helper fast-forwarded actual main to d88c4b9bdb44ddef032e72a1af299bad003e08b3 without bypasses. It built and ran all-target tests for changed xterm-addons/xterm-web, checked affected native all-target dependents and the default workspace, and checked applicable WASM libraries. The raw log records496 passing test invocations across143 summaries, zero failures and zero ignored tests. These invocation counts are not the complete workspace/platform matrix. The helper removed the private target. Exact formatting on that integrated revision also passes. `canonical-merge.log` and `canonical-summary.json` preserve the receipt.

## Remaining work

This is a bounded repair under347, not completion of347 or full-family322. Valid boxed strings and objects with overridden string methods still need source matching semantics. Their length can be valid while matching text is not a primitive JS string. Null matching options, per-line repeated flag reads, flag-getter reentry replacing stored options, raw term mutation between matching phases and refresh object-spread enumeration remain open. Source matching and navigation read raw lengths again after addon validity; this repair does not replace those engine phases. Complete source-unit mapping, randomized raw-option traces and cross-platform proof also remain. Overall equivalence is NOT PROVEN.
