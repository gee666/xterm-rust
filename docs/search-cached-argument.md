# Cached search engine argument

Task 508 follows accepted 493. Full search 347/322 and full-port equivalence remain NOT PROVEN.

SearchAddon passes `this._state.cachedSearchTerm` as the third argument to `findNextWithSelection` or `findPreviousWithSelection`. JavaScript evaluates that argument before entering the engine. The engine guard, selection clear, cache initialization and matching getters can then run callbacks which replace the addon's cached term. Those mutations must not change the argument already passed to the outer engine call.

Rust previously cloned `cached_term` after the raw guard, option reads and matching text. A nested search could change whether the outer search continued from the old selection's start or end. Reverse search could incorrectly skip or add its forward-expansion attempt. Rust now retains that argument at entry to `find_selection`. It does not restore old addon state after callbacks. Successful outer completion still caches the outer term; a thrown callback leaves nested completion intact.

No dependency, unsafe code, JavaScript matching implementation or public API change is added. This repair does not address captured engine/cache ownership across reactivation, successful raw boxed/overridden matching methods or per-line matching getter semantics. Those remain 347/322.

## Verification scope

Two native test functions expand 60 scenarios. One covers 54 combinations of guard, flag or text reentry; forward and reverse search; absent, identical or different cached identity; and nested same-term, different-term or cache-clear operations. The other covers six throwing callbacks and checks that nested selection/cache changes survive. These use real terminal, selection, cache and virtual timer services.

The shared fixture has 72 cases inside one WASM function per browser. It covers both directions, absent/same/different cached strings, regex/caseSensitive/wholeWord getter reentry, same/different nested searches, cache clearing, exact thrown identity, before/after event ordering and follow-up outer/nested cache behavior. It runs unchanged against source classes and actual Rust/WASM public bindings. Existing source assertions are not edited.

Results are pending. Durable commands, logs, negative controls and exact source restores will be recorded in `results/search-cached-argument/`. The old-code control swaps only `crates/xterm-addons/src/search/find.rs` with accepted 493 bytes and restores exact fixed bytes in `finally`.

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
