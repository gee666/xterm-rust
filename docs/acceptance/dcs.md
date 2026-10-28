# DCS acceptance checklist

Scope is every method and branch in `source_repo/src/common/parser/DcsParser.ts` and all 20 cases in `DcsParser.test.ts`. Full terminal/WASM integration belongs to separate criteria. A finite subsystem suite does not prove whole-project equivalence.

## Registration and callback mutation

- A new parser has no registered or active handlers, identifier zero, no fallback callback and no paused dispatch.
- `registerHandler` appends without deduplicating. Hook, put and unhook traverse newest first. A dropped registration handle does not unregister. Explicit disposal removes the first matching object identity, with repeated disposal removing duplicate registrations one at a time.
- `clearHandler` removes only the lookup entry. A previously hooked list remains active. Later registration after clearing creates a separate list. Registration without clearing appends to the shared active list and participates in subsequent payload/unhook calls.
- Disposing one handler during payload updates the active list, not a detached snapshot. Bounds are observed at each callback lookup.
- If a callback removes entries needed by an ongoing reverse traversal, upstream throws a TypeError. Rust returns `DispatchError::ActiveHandlerRemoved` from hook, put, unhook or reset rather than silently skipping callbacks. Partial state remains available for reset/recovery.
- `dispose` removes registration lookup and fallback, detaches active handlers without invoking unhook, and does not silently dispose registration handles. Already queued asynchronous cleanup remains caller-drainable.

Evidence: Rust registration tests, duplicate identity and clear-during-payload cases, four callback-mutation regressions, and all reports in the generated mutation traces.

## Streaming and fallback

- Hook always resets leftover handlers first, aborting each with false in reverse order. It then selects exactly the requested packed identifier and passes all Params values/subparameters to raw streaming handlers.
- With no active handlers, hook reports its identifier and params, put reports UTF-16 payload, and unhook reports its success flag. There is no fallback after all registered handlers decline unhook.
- Put preserves every selected UTF-32 value, chunk boundary and empty chunk. Conversion includes supplementary pairs, lone surrogates, NUL and upstream truncation behavior for malformed u32 values.
- Idle put and idle unhook use identifier zero. After ordinary completion and reset the identifier returns to zero. Explicit dispose alone retains the source's current identifier until another state transition.
- Multiple handler hooks and payload calls preserve exact newest-first ordering. An older handler is not skipped merely because a newer one will eventually handle the sequence.

Evidence: five original registration cases, fallback traces, UTF-32 edge corpus and direct idle/dispose checks.

## Unhook, async continuation and reset

- False handler results bubble with the original success flag. The first true result switches every older handler to false cleanup. Caller abort always passes false.
- A pending handler pauses before invoking the next handler. Its resolved false value continues bubbling; true starts cleanup. A pending cleanup resumes cleanup regardless of its resolved boolean.
- Both sync/async/sync and async/sync/async ordering retain the original callback order, including asynchronous cleanup before older handlers.
- Reset during payload aborts active handlers and permits subsequent reuse. Reset while paused cleans only handlers before the paused index, never re-invokes already-ended handlers, and discards the old dispatch continuation.
- Rust's explicit executor binding retains pending reset cleanup. `take_cleanup` transfers each pending result exactly once. Hook's implicit reset uses the same queue. Futures previously returned by unhook remain caller-owned.
- The buffered handler retains accumulation until successful future resolution; reset/start may clear it earlier, just as upstream allows. Future scheduling is explicit and documented. Dropping an unpolled future is cancellation, not successful execution.

Evidence: four original mixed-async cases, four async callback factory cases, paused reset and cleanup-drain tests, scripted async traces, and shared executor documentation in `docs/dcs.md`.

## Buffered UTF-16 payload and Params

- Every hook clears prior accumulation and overflow state. Empty or single-zero params canonicalize to `[0]`, including subparameters attached to that single zero. Nonzero or multi-value params are cloned so parser mutation cannot change the retained snapshot.
- Whole-payload callbacks run exactly once only on successful, non-overflowed unhook. False unhook never invokes them. A false callback result bubbles to the next older buffered handler with its independently accumulated complete payload.
- The default payload limit is exactly 10,000,000 UTF-16 units. Per-instance test limits exercise the same code path. Exactly limit units are accepted; limit plus one clears accumulation, latches rejection and suppresses callback until the next hook.
- Supplementary values count as two UTF-16 units, lone surrogates as one. Chunks arriving after overflow are ignored. The next hook clears that latch and accepts a fresh payload.
- Empty payloads are valid. Params and data are reset after sync completion and after async completion. Async callback futures own any data they retain, rather than borrowing parser storage across an await.

Evidence: all six original sync factory cases, four async factory cases, cloned/default Params assertions, surrogate/limit checks and a reuse-after-overflow test.

## Evidence and quality

- Run every original DCS test body, assertion and hook without changing the source. Record discovered, executed, passed, failed, skipped and unrun counts. Keep source SHA-256 hashes and exact runtime/tool versions.
- Replay 512 seeded traces at seed `0xdcc512`, four sequences per trace and five put chunks per sequence. Compare every ordered report, not just completion. Retain generated fixture hash and do not hand-edit expected reports.
- Include explicit upstream-confirmed callback-mutation regressions. Fail regeneration or verification if a source case changes, a test is omitted, a count changes or a corpus differs.
- Keep source files under 350 lines, preserve MIT attribution and use safe Rust. Pass owned-file rustfmt checks, focused warning-free Clippy and all DCS Rust tests. Record real command logs under `results/dcs/`.
- Independent review reads implementation and source, checks binding changes, and runs the verification commands. Do not count a teammate's assertion as an executed check.
- Native Windows evidence is not Linux/macOS evidence. DCS unit coverage is not a terminal integration or browser result. Record these exclusions rather than expanding the claim.

Commands: `node scripts/verify-dcs.mjs`; regeneration only through `node scripts/check-dcs.mjs --generate`. Detailed command outcomes and per-case results are under `results/dcs/`.
