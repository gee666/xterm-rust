# Captured search engine receiver

Task 518 follows accepted 508. Full search tasks 347/322 and full-port equivalence remain NOT PROVEN.

The source addon evaluates its engine receiver before calling `findNextWithSelection`, `findPreviousWithSelection` or each highlight `find`. An engine retains the terminal and line cache supplied at construction. Activating the addon on terminal B from an engine guard or matching getter replaces the addon's engine, but the already-running call still searches terminal A. Selection after that call uses the addon's live terminal B. Subsequent highlight calls capture the new engine independently.

Rust previously captured only the host used to clear selection. Cache initialization and the final matching operation looked up the current addon host again. A callback could switch the outer match to B or populate B's cache from the wrong engine state.

A private `EngineContext` now owns the host, line cache and cache listener/timer handles for one activation. Each engine call retains an `Rc` before callbacks. Reactivation disposes the old context's subscriptions and timeout, without changing a captured call's receiver. The old context can still initialize and use its cache after disposal, matching the source's disposed cache behavior without reinstalling jobs. Event/timer callbacks retain only a weak context. Drop cancels owned resources. No public API, dependency, unsafe code or JavaScript matching implementation is added.

Addon-level selection, decoration manager, result tracker and delayed match refresh still use live state. Cache lifecycle is separate because activation replaces the engine and cache together in the source, not merely its host pointer. Existing line-cache algorithms remain in the portable core.

## Tests

Three native functions expand 42 scenarios. They cover successful raw guard, matching flag and matching-text activation; both directions; warm/cold caches; nested searches on B; thrown errors; independently captured highlight calls; old/new host event invalidation; cache TTL and disposal. These use real terminal, selection, decoration and virtual timer services. Successful raw guard/text callbacks are native-only until the remaining raw matching methods in 347 are implemented.

The shared JavaScript fixture has 112 cases. It runs without assertion changes against original source classes and actual Rust/WASM bindings. It covers regex, caseSensitive and wholeWord getter activation, both directions, warm/cold cache, nested B searches, hits/misses, exact frozen-object/null/symbol throw identity, event ordering and follow-up B cache isolation. It deliberately does not assert per-line getter counts or successful boxed/overridden term methods. Those remain 347.

Verification is pending. No test pass or equivalence claim is made in this initial change.

## Reproduce

Use an owned O: target, TEMP and TMP, Rust/Cargo 1.85.1, Node 24.15.0, wasm-bindgen CLI 0.2.100 and installed Chromium/Firefox/WebKit. Set SOURCE_DEPENDENCIES and PLAYWRIGHT_MODULE to the existing reference dependency installation.

```sh
cargo test --locked -j2 -p xterm-web --test search_addon --all-features
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon --all-features -- -D warnings
SEARCH_CASES=search-engine-receiver node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon node scripts/browser-host-check.mjs
```

The private prerequisite is exactly the four accepted 508 own commits copied onto main 1106d480 with coordinator approval in post 409. Private copies c968ad6b, 43109076, 84ba7a19 and 416ddf96 are excluded from the 518 pickup. No private 493 ancestors were copied. All 56 immutable 508 evidence pins and both 776-file original/physical reference manifests were verified before execution. Main, source_repo assertions, AGENTS.md and credentials are preserved.
