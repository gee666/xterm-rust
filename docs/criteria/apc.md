# APC acceptance criteria

Scope is `source_repo/src/common/parser/ApcParser.ts`, its complete test suite, and its use by the outer escape parser. Each numbered requirement below is an assertion to demonstrate, not a statement that the current implementation already passes. `docs/apc-port.md` maps binding names. Full terminal equivalence still requires the other subsystem criteria.

## Registration and lifecycle

- R1. `register_handler` selects the exact packed identifier. Two identifiers keep separate lists. Registering another handler preserves earlier registrations.
- R2. Callback identity is shared. Registering the same object twice invokes it twice, unless dispatch short-circuits. Each explicit disposal removes the first matching identity. Repeated disposal after all matches are gone does nothing.
- R3. Dropping a registration token does not dispose it. Explicit token disposal removes the handler from the shared active list as well as future lookups.
- R4. Clearing a lookup does not detach its already active list. Registering that identifier after clear creates a new list for the next command, not the old active command.
- R5. Registration added to the selected shared list during payload participates in subsequent dispatch according to upstream reverse-loop lengths. Test insertion and removal between payload chunks.
- R6. Parser disposal clears registrations, fallback and active-list ownership without invoking `end`. Old tokens remain safe to dispose. Registration after disposal works as in the source.
- R7. Reset preserves registration and fallback callbacks, aborts remaining active handlers, restores the idle identifier to zero and permits reuse. Starting another command first performs that reset.
- R8. Registration disposal inside a callback follows mutable-list indexing, not a frozen snapshot. If a needed index disappears, return `DispatchError::ActiveHandlerRemoved` at the same point upstream throws TypeError. Preserve preceding callbacks and partial parser state. Do not silently skip, retry or complete successfully.

Evidence must include distinct-identifier, duplicate-identity, repeated-disposal, token-drop, clear-during-active, registration-during-active and self-removal regressions. Compare callback traces with the read-only source for mutable-list cases.

## Dispatch, payload and fallback

- D1. `start(ident)` takes the packed identifier from the outer parser. It does not parse OSC numeric prefixes or consume APC payload as an identifier.
- D2. For each start and payload chunk, invoke the selected handlers newest first. Forward even empty payload chunks. Slice boundaries correspond exactly to the original data/start/end triple.
- D3. When no handlers are selected, fallback receives identifier and typed Start, Put and End actions in the same order as upstream. No implicit Start is added to an idle Put or End call.
- D4. Idle Put and End use identifier zero. Reset and completed End return to this idle behavior. An unmatched identifier stays active until completion.
- D5. Fallback Put converts UTF-32 with the same JavaScript UTF-16 semantics as the shared decoder, including supplementary values, lone surrogates, zero and out-of-range u32 values. Do not replace lone surrogates with U+FFFD.
- D6. `end(true)` invokes handlers newest first until one returns true. Every older handler receives `end(false)` cleanup. If all return false, every handler receives true. The fallback is not invoked just because registered handlers return false.
- D7. `end(false)` invokes abort cleanup in source order. A boolean true still changes only bubbling/cleanup state, not the success flag. All remaining callbacks observe false.
- D8. Completed end releases active-list ownership and clears identifier/pause state. Repeated idle end calls remain observable fallback events as upstream does.
- D9. Missing-index errors during start, payload, ordinary end or reset are recoverable typed errors, not successful no-ops or panics. Preserve source partial-state behavior before error propagation.

Evidence must compare full callback and fallback logs, including identifiers, payload code units and success flags, not only callback counts or final text.

## Asynchronous continuation and reset

- A1. An asynchronous end result pauses before calling older handlers. The caller owns and polls that future. Resume with the same success flag and the resolved boolean.
- A2. Resolved false continues normal bubbling. Resolved true starts cleanup for older handlers. During cleanup, an asynchronous false must not restore normal bubbling.
- A3. Multiple pending callbacks resume at the saved next index. Never reinvoke the callback that produced the pending result. Test both sync/async/sync and async/sync/async layouts with first-handled and all-false outcomes.
- A4. Cleanup callbacks preserve order even when both the handled callback and an older cleanup callback are asynchronous.
- A5. Reset while paused skips the already returned callback and all newer callbacks. It aborts only remaining older handlers. The previously returned future remains caller-owned and may complete.
- A6. Since Rust futures are lazy, asynchronous cleanup initiated by reset or implicit reset inside start must be retained for `take_cleanup`. Transfer each exactly once in reverse invocation order. Do not silently drop or claim it executed without polling.
- A7. A pending cleanup queue remains available through parser disposal. The outer integration must drive it on its executor. Document eager callback prefixes and future ownership rather than introducing a hidden global runtime.
- A8. The buffered callback keeps accumulated payload alive until asynchronous completion, clears it on resolution, and resets it on the next start. A dropped future must not corrupt subsequent command setup.
- A9. A synchronous callback failure or rejected asynchronous callback is not a false return. Propagate a typed callback error without invoking older handlers or resuming subsequent input. Keep the pending state until explicit reset. Retained payload clears on the next start, not on failed completion.

Required evidence includes the four original mixed asynchronous cases, async callback-factory cases, reset before completion, reset during pause, multiple async cleanup callbacks and the outer parser's resume/abort integration tests. No sleeps are needed.

## Whole-payload callback and limits

- B1. `ApcHandler` invokes its callback once at successful end with all chunks concatenated in order. Empty successful payload invokes it with an empty UTF-16 vector.
- B2. Aborted commands and cleanup calls do not invoke the payload callback. Reset and restart clear previous accumulation.
- B3. The default limit is exactly 10,000,000 UTF-16 units from ParserConstants, not the stale 50 MB comment elsewhere in source. An instance-specific test limit replaces upstream's private static test mutation.
- B4. A payload exactly at the limit succeeds. The first append taking length above the limit clears accumulation and suppresses the entire command callback. Subsequent puts cannot reactivate it.
- B5. Supplementary code points count as two UTF-16 units. Lone surrogates stay one unit and remain unchanged in callback data. Check exact-limit and one-unit-overflow boundaries with both ASCII and supplementary input.
- B6. After overflow or abort, a new command succeeds with clean data. No payload leaks across commands.
- B7. Sync and async callback true/false results participate in identical parser bubbling. Removing the latest callback handler allows the earlier callback to receive the next command.

## Verification and integration

- V1. Discover and execute all 20 original `ApcParser.test.ts` test bodies with assertions unchanged. Await asynchronous cases and preserve nested beforeEach/afterEach hooks. Record exact passed, failed, skipped and unrun counts, source hashes and runtime version.
- V2. Map all 20 cases to Rust assertions. Parameterized Rust tests may combine repeated cases, but each expected event sequence, payload and limit assertion must remain represented.
- V3. Retain and replay 256 deterministic upstream-generated traces of 80 operations each, seed `0xa9c2025`. Compare every callback and fallback record. Include registration, disposal, reset, clear, restart, completion, abort, empty chunks and UTF-16 boundary inputs. Retain full failing traces.
- V4. Add source-oracle traces for callback-triggered active-list removal, including successful index shifts and the abrupt-error case. Check Rust error kind and partial callback log, not only that an error exists.
- V5. Run focused Rust unit/differential tests, warning-free Clippy and formatting checks. Retain exact commands and raw logs under results/apc. Independent review must inspect the shared OSC dispatch used by APC, not just the adapter.
- V6. Through the actual outer escape parser, test ESC _ entry, APC identifier/intermediates, payload chunk boundaries, ST termination, CAN/SUB abort, reset and pending-handler resume. Verify that the outer parser forwards errors and drives retained cleanup futures. Subparser-only tests do not satisfy this item.
- V7. Exercise the APC implementation through native terminal and Rust-backed browser integration when those APIs exist. Track unavailable platform/browser execution explicitly. No subsystem smoke test may be presented as full terminal equivalence.
