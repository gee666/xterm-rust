# Cached search engine argument

Task 508 follows accepted 493. Full search 347/322 and full-port equivalence remain NOT PROVEN.

SearchAddon passes `this._state.cachedSearchTerm` as the third argument to `findNextWithSelection` or `findPreviousWithSelection`. JavaScript evaluates that argument before entering the engine. The engine guard, selection clear, cache initialization and matching getters can then run callbacks which replace the addon's cached term. Those mutations must not change the argument already passed to the outer engine call.

Rust previously cloned `cached_term` after the raw guard, option reads and matching text. A nested search could change whether the outer search continued from the old selection's start or end. Reverse search could incorrectly skip or add its forward-expansion attempt. Rust now retains that argument at entry to `find_selection`. It does not restore old addon state after callbacks. Successful outer completion still caches the outer term; a thrown callback leaves nested completion intact.

No dependency, unsafe code, JavaScript matching implementation or public API change is added. This repair does not address captured engine/cache ownership across reactivation, successful raw boxed/overridden matching methods or per-line matching getter semantics. Those remain 347/322.

## Verification scope

Two native test functions expand 60 scenarios. One covers 54 combinations of guard, flag or text reentry; forward and reverse search; absent, identical or different cached identity; and nested same-term, different-term or cache-clear operations. The other covers six throwing callbacks and checks that nested selection/cache changes survive. These use real terminal, selection, cache and virtual timer services.

The shared fixture has 90 cases inside one WASM function per browser, including 18 pre-engine decoration-getter controls which must still change the argument before engine entry. It covers both directions, absent/same/different cached strings, regex/caseSensitive/wholeWord getter reentry, same/different nested searches, cache clearing, exact thrown identity, before/after event ordering and follow-up outer/nested cache behavior. It runs unchanged against source classes and actual Rust/WASM public bindings. Existing source assertions are not edited.

Observed native Windows results are retained in `results/search-cached-argument/`:

- All 64 native functions passed, zero failed, ignored or filtered. The two new functions expand 60 scenarios.
- Swapping only `crates/xterm-addons/src/search/find.rs` with accepted 493 bytes produced one pass and one failure among the two new functions, with 62 retained functions filtered. Exact fixed bytes were restored in `finally`, then all 64 passed again.
- Scoped native library/search-test and actual WASM library/search-test Clippy passed with `-D warnings`. Changed Rust formatting and all five workspace member formatting checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS206, which is retained rather than reported as a full pass.
- The unchanged original source passed all 90 shared cases in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, 270 cases total.
- Actual Rust/WASM passed 20 functions in each engine, 60 functions total. All 90 shared cases are inside one function per engine, not 90 extra functions.
- Actual old-code WASM failed only the new function in Chromium, with 19 retained functions passing. The finally block restored the exact fixed source hash before all 20 functions passed in all three engines. Both variants require actual Cargo `fresh=false`, distinct WASM hashes and exact shared fixture bytes embedded in both compiler artifacts.

Both original and physical reference-copy manifests passed all 776 hashes before and after execution. The 54 accepted 493 evidence pins were checked against immutable Git objects before its exact four-commit private reuse. An initial shell export set TEMP to owned O: but expanded TMP from the inherited C: value. No known owned files remained there. Both drivers were corrected to set TEMP and TMP explicitly inside Node, then the scoped native gates and actual old/final compilations were rerun before the successful browser matrix. The correction is retained in `environment-correction.json`.

Full workspace Clippy/tests, original complete SearchAddon suites, seeded corpus, release, demo, GC and native Linux/macOS were not run for 508. Scoped passes do not establish full SearchAddon or full-port parity. Temporary native/WASM binaries are retained only until independent review and safe cleanup.

## Reproduce

Use Rust/Cargo 1.85.1, Node 24.15.0, wasm-bindgen CLI 0.2.100 and installed Chromium/Firefox/WebKit. Use an owned private O: target and O: TEMP/TMP; point SOURCE_DEPENDENCIES and PLAYWRIGHT_MODULE at the installed reference dependency tree.

```sh
cargo test --locked -j2 -p xterm-web --test search_addon
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon -- -D warnings
SEARCH_CASES=search-cached-argument node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
```

The private prerequisite is the exact four accepted 493 commits, copied onto main e64f4870 under coordinator approval post 652/3791. None belongs in the 508 pickup. Accepted 495 HTTP cleanup is already on main; no extra helper copy is required. Main, AGENTS.md, credentials and the original reference remain untouched.
