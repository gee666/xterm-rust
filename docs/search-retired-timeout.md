# Retired search cache timeout setup

Task 544 repairs one part of 527. Full search and full-port equivalence remain NOT PROVEN.

## Failure and repair

An ordinary search captures its engine before evaluating the engine's raw term-length guard. The guard can activate the addon on a different terminal or dispose the addon. The captured engine then continues on the original terminal, including cache initialization.

The source `SearchLineCache.initLinesCache` still invokes `disposableTimeout` after its `MutableDisposable` has been disposed. That setter ignores the returned handle, but it cannot suppress errors from creating the timeout. Rust's retired engine returned before evaluating timeout creation. Consequently, a throwing `window.setTimeout` was never called and a later matching-option error incorrectly won.

`EngineContext.init` now attempts timeout creation in that retired branch. Errors propagate through the existing host error representation, before any matching option or raw text conversion. A returned timeout is still cancelled immediately by the existing RAII branch. Initial installation uses the source's full 15,000 ms TTL, rather than subtracting a second clock read from the deadline. Expiry callbacks read the clock once and reuse that timestamp when calculating an extension. No new dependency, unsafe operation, public API or JavaScript matching implementation is introduced.

## Deliberate scope

This is not approval of a source/Rust lifetime divergence. Task 527 remains open for the source's orphan listener/timer liveness, observable registration counts, timer callbacks after disposal and a compatible ownership solution or explicit approved divergence. A successful timer setup in these tests proves invocation and subsequent matching-error order only. Native pending-timer assertions check Rust cleanup, not source liveness equivalence. Calls after explicit addon disposal are covered only where the raw guard disposes an already captured engine, not arbitrary later searches.

The fixture uses a raw term whose second length read retires the captured engine. It deliberately throws from the first matching-option getter, so successful boxed or overridden term matching is neither required nor claimed. Full raw matching, repeated per-line getter order and cell mutation phases remain 347 and 322.

## Controls

The shared source/actual-WASM fixture has 56 scenarios. It covers both search directions, warm/cold caches, guard activation/disposal, thrown frozen-object/null/symbol identities, successful timeout setup followed by a distinct matching failure, and strict-zero engine guards that must not schedule at all. Successful source timers are explicitly cancelled in the fixture's finally block, since source disposal intentionally cannot own those returned handles. Window methods, terminals and DOM nodes are restored or disposed in finally. The same fixture and assertions run against unchanged source classes and real Rust/WASM bindings.

One native test expands 24 scenarios over the same direction, cache and lifecycle combinations. A second native test fires a refreshed live cache's first timeout with an advancing clock, checks one callback clock read and the exact 5,000 ms extension, then checks final expiry without rescheduling. Its clock advances on every read to catch shortened initial delays and extra clock reads. It compares retained error identity and exact guard/timeout/matching phase order, verifies noScroll is reached only for an empty engine result, and checks Rust timers are not left pending.

## Verification

The pre-review native Windows Rust/Cargo 1.85.1 execution passed all 71 functions, zero failed, ignored or filtered. The independent reviewer reran that pinned executable, verified all 1,872 inputs and runtime pins, and found no production defect. Review correctly requested the browser gates and a discriminating expiry-callback clock test. That added test raises the current native count to 72 and needs fresh execution; the 71-function result is historical, not a current final pass. Exact accepted-518 engine bytes failed only the new filtered native function, zero passes, one failure and 70 filtered, at the intended retained-error identity assertion. The finally block restored the fixed source hash, then all 71 passed again. Scoped native and WASM library/search-test Clippy passed with `-D warnings`. All five whole-member formatting checks passed. All-feature WASM compilation produced a fresh test artifact embedding the exact shared fixture. Browser execution, complete original suites, full-workspace quality/release and native Linux/macOS are still UNRUN for 544.

The initial native run passed 70 and failed the new clock-count assertion. Its total-count expectation overlooked the existing `run_engine` clock read during warmup. The corrected assertion measures the cache phase relative to the completed warmup. Both raw failure and corrected passes are retained; no original assertion changed.

Historical 518 results are prerequisites, not current 544 passes. Its seven approved private prerequisite copies are excluded from the eventual 544 pickup, and all four private 508 copies are excluded entirely. Main is read-only throughout producer work. Commands, raw logs, 1,872 tracked build-input hashes, native runtime descriptors and actual compiler/fixture identity are under `results/search-retired-timeout/`.
