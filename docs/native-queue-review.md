# Independent native queue review

Task147 reviews task128, owned by Antigone. Reviewer Epictetus inspected `headless/native_queue.rs`, `queued.rs`, `queue_bridge.rs`, their tests, and the source queue/parser/core resize contracts. This is native Windows evidence, not whole-terminal or cross-platform equivalence.

## Findings resolved

1. The initial native pump took all timers out of live storage before dispatch. Cancelling a later timer from an earlier callback could not remove that later callback from the local batch. The pump now snapshots timer IDs, removes each timer from live storage immediately before dispatch, and skips cancelled entries. `earlier_timer_can_cancel_a_later_timer_in_the_same_host_poll` passes.
2. The initial queued wrapper substituted true for an absent synchronous continuation while a parser was paused. Source `EscapeSequenceParser.parse` rejects an undefined result in that state. The wrapper now returns the original `ImproperContinuation` error before clearing pending state. `sync_flush_during_paused_handler_preserves_improper_continuation_error` passes.
3. Queued resize originally had no source-style pre-resize flush. The wrapper now flushes pending writes before changing dimensions, while preserving the headless source's early return for unchanged dimensions. Both changed/unchanged resize regressions pass.
4. Reviewed pending handler, cleanup and error paths retain typed exceptions and execute owned cleanup futures. The native pump does not busy-poll unresolved futures, does not hold a RefCell borrow across callbacks/polls, and drops owned futures outside the scheduler borrow. No unsafe code was introduced.

A suspected Bell callback visibility issue was checked against the owner's source/native reentry probes. Those probes already observe the pre-following-print buffer at the control-time callback. This review does not report that suspicion as an unresolved defect.

## Independent real-source traces

`scripts/native-queue-oracle.mjs` bundles the unchanged source headless public Terminal with the reference's existing esbuild dependency. It records every bundled source hash. It controls only timer dispatch and clock, never terminal engine logic. Temporary bundles stay in `tmp/native-queue-oracle/`.

The five scenarios cover24 operation steps:

- Deferred FIFO strings, byte chunks, writeln CRLF and callback/parsed ordering.
- A queued11-column line followed by a shrink to5 columns, flush/reflow, a wide Unicode character, CRLF and scrolling.
- Unchanged headless resize preserving deferred input until the timer runs.
- User input causing the first write to parse immediately, then disabled stdin preventing that fast path.
- Split UTF-8 astral bytes, combining text and split UTF-16 surrogate pairs.

`tests/native_queue_differential.rs` replays those operations through the actual Rust QueuedTerminal. After every operation it compares dimensions, cursor, buffer base/viewport/length, every line's translated UTF-16 text, pending units, queue offset/backing length, scheduled-write presence and the complete callback/parsed/data/resize log. It does not claim full packed-cell or mode comparison; the separate terminal-engine differential suite owns that broader obligation.

Source timer dispatch explicitly targets the write timer. Unrelated buffer-cleanup host tasks are outside this queue corpus. The source's own terminal disposal still must cancel its retained write timer.

## Executed commands and results

```text
node scripts/native-queue-oracle.mjs
cargo test -p xterm-core --test headless_queue --test headless_queue_edges --test native_queue_differential
cargo clippy -p xterm-core --test headless_queue --test headless_queue_edges --test native_queue_differential -- -D warnings
```

- `results/native-queue-review/oracle.json` records Node24.15.0, Windows, esbuild0.28.1, source hashes,5 scenarios,24 steps and fixture SHA256.
- `results/native-queue-review/review-tests.log` records10 original native integration tests,4 source-edge regressions and1 independent differential replay passed. Zero failures or ignored tests.
- `results/native-queue-review/review-clippy.log` records warning-free scoped Clippy.
- Initial failed compilation attempts remain in this directory. They identify in-flight SGR module exports and a Win32 reference-type edit, not hidden successes. The final commands above ran after owners repaired those integrations.

The native implementation task128 is accepted on this scoped evidence. Task147's additional replay artifacts require their own independent review. Browser adapter task118, native Linux/macOS runs, randomized full-terminal coverage and project-wide quality gates are separate, unproven requirements. Overall xterm equivalence remains NOT PROVEN.
