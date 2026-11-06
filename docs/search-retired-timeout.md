# Retired search cache timeout setup

Task 544 repairs the scheduling-error part of 527. Full search and full-port equivalence remain NOT PROVEN.

## Failure and repair

An ordinary search captures its engine before evaluating the engine's raw term-length guard. That getter can activate the addon elsewhere or dispose it. The captured engine still continues on the original terminal, including cache initialization.

Source `SearchLineCache.initLinesCache` invokes `disposableTimeout` even after its `MutableDisposable` has been disposed. The setter ignores the returned handle, but cannot suppress an error from creating that handle. Rust's retired engine returned before timeout creation. A throwing `window.setTimeout` was therefore skipped, and a later matching-option error incorrectly won.

`EngineContext.init` now attempts timeout creation in that branch. Setup errors propagate through the existing host error representation before matching or raw text conversion. A successfully returned retired timeout is still cancelled immediately for Rust ownership safety.

Initial installation also uses the source's full 15,000 ms TTL. The old helper subtracted a second clock read, potentially shortening it to 14,999 ms. Expiry callbacks now read the clock once and reuse that timestamp for expiry and extension calculations. No dependency, unsafe operation, public API or JavaScript matching implementation was added.

## Scope and remaining work

Task 527 remains open for orphan listener/timer registration counts, callback liveness after disposal and a compatible ownership solution or explicit approved divergence. These tests do not approve that divergence. Their successful-setup cases prove invocation and subsequent matching-error order only. Native pending-timer assertions check Rust cleanup, not source orphan liveness.

The raw term deliberately throws through the first matching-option getter. Successful boxed/overridden term matching, per-line getter phases, cell mutation and arbitrary searches after addon disposal remain 347/322. Guard disposal here concerns an already captured engine, not all later calls on a disposed addon.

## Tests and actual results

Evidence is under `results/search-retired-timeout/`, on native Windows with Rust/Cargo 1.85.1, Node 24.15.0 and wasm-bindgen CLI 0.2.100.

- All 72 native functions pass, zero failed, ignored or filtered. One new function expands 24 direction/cache/lifecycle/setup scenarios. The second fires a refreshed live cache timeout with an advancing clock, checks one callback clock read and the exact 5,000 ms extension, then verifies final expiry without rescheduling.
- Exact accepted-518 engine bytes fail the designated native setup-error function at the intended retained-error identity assertion, zero passes, one failure and 71 filtered. A finally block restores the fixed source hash, then all 72 pass again.
- The identical shared fixture passes all 56 cases against unchanged source in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. It checks both directions, warm/cold caches, guard activation/disposal, frozen-object/null/symbol throw identities, successful setup followed by a distinct matching error, and strict-zero guards that must not schedule.
- Actual old-code WASM fails only the new function in Chromium, 21 passes and one failure. The diagnostic shows guard then matching, with timeout setup missing. After exact source restoration, actual current WASM passes all 22 functions in each of the three engines, zero failed, ignored or filtered. The 56 shared cases are inside one function per engine, not 56 extra functions.
- Both completed WASM control variants are actual rebuilds, `fresh=false`, with distinct compiler and served-WASM hashes and the exact shared fixture embedded. Final compiler hash is `cc35b89d5ece2a4e2873e95094ad6d6055895a9b46daf2198c3a8f6cfef8ba79`. Bindings, served WASM and test-source identity were checked.
- Final scoped native and WASM library/search-test Clippy pass with `-D warnings`. All five whole-member formatting checks pass; xterm-web was checked again after the WASM helper correction. Full-workspace quality/release, complete original Search suites, seeded corpus, demo/GC and native Linux/macOS were not run for 544.
- All 1,872 current tracked crate/build inputs, ten retained runtime artifact descriptors and both 776-file original/physical source manifests were checked. Runtime binaries are temporary verification artifacts, not deliverables.

The fixture cancels successfully installed source timers in finally because the disposed source owner cannot own them. It also restores window methods, disposes terminals and removes DOM nodes. Source registrations are bounded by the outer terminal lifetime. Rust never deliberately leaks them.

## Retained failures and independent review

The first native run passed 70 and failed the new total clock-count expectation. Warm matching has an existing separate `run_engine` clock read. The new assertion now measures the cache phase relative to completed warmup. The raw failure is retained; no original assertion changed.

The first actual old-WASM attempt reached the intended identity failure, but the shared helper's `JsFuture.unwrap` then triggered a secondary wasm-test ScopedTLS panic. That run discovered 22 functions but produced no completed count summary, so it is not reported as 21/1. Its report, console trace, artifacts and finally restoration receipt remain retained. The new test now returns `Result<(), JsValue>` directly to wasm-bindgen-test through a fallible helper. Existing tests retain their unwrap wrapper. Assertions, fixture bytes, feature selection and timeout limits did not change. The corrected old-code run produces the completed 21/1 result above; final 22-by-three execution uses that same corrected harness.

The first isolated review of `ae8a8e62` independently ran the then-current 71 native functions, checked all 1,872 inputs, native pins, compiler hash and embedded fixture, and found no production defect. It correctly requested browser execution and a discriminating expiry-clock regression. The added second native function and the actual browser results address those findings. Resumed review is recorded separately.

## Reproduce and integration

Use an owned O: target and literal O: TEMP/TMP/TMPDIR, Cargo `-j2`, existing reference dependencies and an explicitly coordinated heavy/browser allocation. Set SOURCE_DEPENDENCIES to the installed reference root, PLAYWRIGHT_MODULE to its Playwright entry and WASM_BINDGEN to CLI 0.2.100.

```sh
cargo test --locked -j2 -p xterm-web --test search_addon --all-features
cargo clippy --locked -j2 -p xterm-addons -p xterm-web --lib --test search_addon --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_search_addon --all-features -- -D warnings
SEARCH_CASES=search-retired-timeout node scripts/search-call-args-source.mjs
WASM_TEST=wasm_search_addon CARGO_FEATURES=acceptance-hooks node scripts/browser-host-check.mjs
```

Main remained read-only. Approved private prerequisites are exactly seven accepted 518 copies on main0442, patch-byte audited against the originals. They are excluded from the 544 pickup. No private 508 copies were included. Original assertions, AGENTS.md, credentials and unrelated user work are preserved. Historical 518 evidence is not current 544 proof.
