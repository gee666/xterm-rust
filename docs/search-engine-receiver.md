# Captured search engine receiver

Task 518 follows accepted 508. Full search tasks 347/322 and full-port equivalence remain NOT PROVEN.

The source addon evaluates its engine receiver before calling `findNextWithSelection`, `findPreviousWithSelection` or each highlight `find`. An engine retains the terminal and line cache supplied at construction. Activating the addon on terminal B from an engine guard or matching getter replaces the addon's engine, but the already-running call still searches terminal A. Selection after that call uses the addon's live terminal B. Subsequent highlight calls capture the new engine independently.

Rust previously captured only the host used to clear selection. Cache initialization and the final matching operation looked up the current addon host again. A callback could switch the outer match to B or populate B's cache from the wrong engine state.

A private `EngineContext` now owns the host, line cache and cache listener/timer handles for one activation. Each engine call retains an `Rc` before callbacks. Reactivation disposes the old context's subscriptions and timeout, without changing a captured call's receiver. A captured call can still use its retired context without reinstalling jobs. This resource-safety rule is not full post-disposal equivalence, as explained below. Event/timer callbacks retain only a weak context. Drop cancels owned resources. No public API, dependency, unsafe code or JavaScript matching implementation is added.

Addon-level selection, decoration manager, result tracker and delayed match refresh still use live state. The addon host remains separate from the engine host because the source updates its terminal before replacing its cache/engine. A disposal callback can reenter activation between those assignments. Cache lifecycle is separate because activation replaces the engine and cache together in the source, not merely its host pointer. Existing line-cache algorithms remain in the portable core.

## Tests

Six native functions expand 45 scenarios. They cover successful raw guard, matching flag and matching-text activation; both directions; warm/cold caches; nested searches on B; thrown errors; independently captured highlight calls; old/new host event invalidation; cache TTL and disposal. A forwarding host also reactivates during timer installation and checks immediate cancellation before matching. Timer-disposal callbacks also reactivate without an internal state borrow held and verify the source's separate live addon host and outer engine receivers during nested activation. These use real terminal, selection, decoration and virtual timer services. Successful raw guard/text callbacks are native-only until the remaining raw matching methods in 347 are implemented.

The shared JavaScript fixture has 118 cases, including six highlight receiver controls with mixed A/B result lists. It runs without assertion changes against original source classes and actual Rust/WASM bindings. It covers regex, caseSensitive and wholeWord getter activation, both directions, warm/cold cache, nested B searches, hits/misses, exact frozen-object/null/symbol throw identity, event ordering and follow-up B cache isolation. It deliberately does not assert per-line getter counts, successful boxed/overridden term methods or cell mutation/resizing inside matching getters. Those access-phase behaviors remain 347.

## Remaining disposal gap

The original `common/Lifecycle.ts` MutableDisposable setter ignores new values after disposal. It does not immediately dispose them. Thus a captured engine whose raw guard activates B before old-cache initialization can create unowned source listeners/timers. Activation from a timer-installation callback can similarly leave the just-created source timer unowned. At the 518 checkpoint Rust cancelled returned handles and kept retired contexts inert rather than deliberately leaking registrations. Task 544 now attempts retired timeout setup so scheduling failures are not suppressed, while still cancelling successful returned timers. See `search-retired-timeout.md` for its separate current evidence. The 518 native timer-installation test checks Rust resource safety, not JavaScript equivalence for the source leak. Source registration counts, scheduling failures and successful raw-object matching across these phases remain 347/322. Calls which reactivate after explicit addon disposal also remain unproven. The shared browser fixture activates from matching getters after cache initialization, so it does not exercise or conceal this gap.

## Observed verification

Results are retained in `results/search-engine-receiver/` on native Windows with Rust/Cargo 1.85.1, Node 24.15.0 and wasm-bindgen CLI 0.2.100.

- All 70 native functions passed, zero failed, ignored or filtered. The six new functions expand 45 scenarios.
- Swapping only addon.rs and find.rs to accepted 508 bytes failed two of three receiver functions and two of three cache functions. Each directed run passed one retained control and filtered 67 functions. Exact fixed hashes were restored in finally before all 70 passed again.
- The unchanged source passed all 118 shared cases in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4, 354 cases total.
- Actual Rust/WASM passed 21 functions in each engine after correcting the new test initializer. The 118 cases are inside one function per engine, not 118 extra functions. Actual old-code WASM failed only that new function in Chromium, with 20 retained functions passing, then the restored production bytes passed all 21 functions in all three engines.
- Both WASM variants were actual Cargo rebuilds, `fresh=false`, with distinct compiler/served-WASM hashes and the exact shared fixture embedded in both compiler artifacts. Initial and restored fixed artifacts were byte-identical before the formatting correction below.
- Final scoped native and WASM library/search-test Clippy passed with `-D warnings`. All five complete workspace member formatting checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS206; its raw result is retained, not reported as a full formatting pass.
- All 56 inherited immutable 508 evidence pins, both 776-file reference manifests, the native executable and all four produced WASM/binding artifact sets were verified. The input manifest records 1,817 tracked crate/build inputs.

Two test-development failures are preserved. The first native run expected column 4 after a nested warm-cache search on B, but the captured cached-term argument correctly advances that nested search to column 6. Only the new expectation was corrected, and the shared assertions then confirmed it against unchanged source classes. The first WASM attempt used an invalid escaped raw JSON initializer, so its new function failed before any shared cases ran. Correcting that initializer changed no assertion. A later member-format check required its shorter invocation to fit on one line. That formatting-only rebuild changed compiler and served-WASM bytes, so the earlier browser pass is not silently reused as final stable-revision evidence. The final formatted-harness old-code control again failed only the intended receiver assertion in Chromium, 20 passes and one failure. After finally restoring both production file hashes, all 21 functions passed in each browser on the final formatted artifact. Its compiler hash exactly matches the independent formatting-only rebuild, `c3b83786b7e0f80355e754bd247b66b2986caa6393d44d0904e84409fa4da984`. Both final variants were actual Cargo rebuilds and embedded the unchanged shared fixture. `formatted-artifact-identity.json` and `compiler-controls.json` distinguish these hashes from the earlier formatting checkpoint.

## Independent review correction

The first isolated review of frozen `b2d546a0` independently ran all 70 native functions and verified 107 evidence pins, 1,817 inputs, 21 runtime pins and both 776-file reference manifests. It found a Rust-specific reentrancy panic. Assigning a new engine through `inner.engine.borrow_mut()` could drop a different nested engine while the slot remained borrowed. Cancelling that engine's live timer could reenter search and borrow the same slot.

Activation now moves the replaced `Rc` out with `RefCell::replace` and drops it after the borrow ends. The existing nested-disposal scenario now performs a nested search to install C's timer, then reenters search when that timer is cancelled. Exact pre-review addon bytes fail that strengthened test with the intended RefCell panic, zero passes, one failure and 69 filtered functions. A finally block restores the fixed source hash, then all 70 native functions and scoped native/WASM Clippy pass. The earlier native 416 controls are retained as historical pre-review evidence, not misrepresented as the strengthened current test.

The post-review production change was rebuilt and all 21 WASM functions passed in each of Chromium, Firefox and WebKit, zero failures, ignored or filtered functions. Cargo reported `fresh=false`; compiler `4a75e7a67c1a6eeddc6beaa17b1a55417646bec0d74ea62ab324bb81f5b522a3` embeds the exact shared fixture, and both served WASM/binding hashes were checked. The earlier `c3b83786` browser runs remain historical. The retained 416 receiver control uses the identical WASM harness and shared fixture; the new P2 panic was reproduced separately with exact pre-review addon bytes in native execution. No source or shared JavaScript assertions changed for the review correction.

The same isolated reviewer accepted frozen `b82a71c2` after the correction. It independently reran the current hash-pinned executable, all 70 passes, and checked all 129 then-current summary pins, 1,817 input pins, 24 runtime pins, both reference manifests and all 56 prerequisite pins. It inspected current WASM/Clippy evidence without compiling or launching browsers. The later receipt and cleanup preflight add metadata only. Temporary native/WASM binaries are verification artifacts, not deliverables, and are removed only after this review and source/evidence preservation.

Accepted source/test/proof commits are `9afc453b`, `05f59d87`, `3b6eb6a5`, `8cf831f1`, `b2d546a0` and `b82a71c2`, followed by a review/cleanup metadata-only tail. `archive/Danielle518-complete` preserves the complete own chain. None of the four private 508 copies belongs in the pickup. Do not append 518 to the already-frozen 531 batch.

Full workspace Clippy/tests/release, complete original SearchAddon suites, seeded search corpus, demo, GC and native Linux/macOS were not run for 518. Successful raw-object matching and per-line getter/mutation phases remain 347/322; disposed registration/scheduling behavior remains 527. Scoped passes do not establish full SearchAddon or full-port equivalence.

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
