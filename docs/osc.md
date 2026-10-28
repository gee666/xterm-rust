# OSC parser contracts and evidence

This document covers `source_repo/src/common/parser/OscParser.ts` and all 23 cases in `OscParser.test.ts`. It does not establish terminal, browser, or addon equivalence. The implementation is `crates/xterm-core/src/parser/osc.rs`, the shared result/error contract is `parser/handler.rs`, and the payload adapter is `parser/osc_handler.rs`.

## Acceptance checklist

### Identifier and streaming paths

- An idle parser ignores `put` and `end`. `start` resets leftovers before entering identifier collection.
- Identifier digits may span any number of chunks. Leading zeros do not change identity. Only ASCII 0 through 9 are accepted before semicolon. Any other value aborts the command until reset or another start; no fallback or handler callbacks occur for the aborted identifier.
- Preserve the source operation order `id * 10 + ASCII - 48` using IEEE-754 double arithmetic, including rounding beyond 2^53 and infinity for long identifiers. Empty identifier remains -1, not zero.
- Semicolon announces start exactly once, including a semicolon-only chunk. Forward only nonempty post-semicolon payload chunks unchanged as UTF-32 slices. Fallback converts those slices to UTF-16 units, including astral pairs and lone surrogates.
- Ending during identifier collection still announces start then end, with no synthetic put. Repeated end after completed dispatch has no effect.
- Incoming slices replace JavaScript's data/start/end triple. Bounds are validated by Rust slice construction, not unchecked indexing inside the parser.

### Registration and mutable traversal

- Registrations are ordered by insertion, dispatched newest first for start, put and end. Token drop is not disposal. Explicit token disposal removes the first equal handler identity, matching JavaScript's `indexOf`. Disposing the same token again removes the next matching registration when one identity was registered more than once; after all matches are gone it has no effect.
- Multiple registrations of the same identity remain separate list entries. Clearing an identifier detaches its registry entry without changing a command already using the old list. New registrations after clear form a new list.
- `registrar()` supplies a cloneable weak registration handle for callbacks that must register or clear without reborrowing the parser. It returns None only after the owning parser drops, not after explicit disposal. Registration and disposal tokens use the shared generic registry with ESC/CSI/DCS. Weak token captures deliberately avoid a callback capturing its own token from creating a Rust reference cycle. Registry and active lists own callbacks; detached tokens do not retain otherwise unreachable callbacks.
- No registered handlers means fallback receives identifier, action and action-specific value. An emptied active list uses fallback on subsequent puts/end. Disposal clears registry, fallback and active list without sending end callbacks.
- Do not hold a handler-list borrow across callbacks. Callbacks may dispose registration tokens. Preserve live-list traversal rather than taking a snapshot. If mutation removes an index the ongoing traversal needs, return `DispatchError::ActiveHandlerRemoved`, the recoverable Rust equivalent of upstream's TypeError. Do not skip it silently or panic. Preserve partial state on error.

### Bubbling, futures and reset

- End sends its success flag newest first until one handler returns true. All older handlers still receive end(false) for cleanup. False continues bubbling, including when command success is false.
- A failed synchronous callback is not a false result. `HandlerResult::Failed` returns the typed DispatchError immediately, retaining already-committed state and skipping lower handlers. Event callback errors retain the actual EmitError cause, including a native or browser-owned error object.
- A pending handler pauses traversal. The caller awaits its future and resumes using the same success flag and resolved boolean. Resume starts one index below the pending handler, never repeats it. Pending cleanup resumes cleanup regardless of its result.
- Fallible asynchronous callbacks use `PendingFallible`, preserving rejected promises without converting failure to false. `resolve()` normalizes both future forms into Result. Resume only on Ok. On Err the parser remains paused until explicit reset; no trailing input or older callbacks execute. Whole-payload data clears on successful resolution but remains on error.
- Test sync/async/sync and async/sync/async orders for first-handler-wins and all-false bubbling. Include pending results in both execution and cleanup passes.
- Reset during payload aborts every unfinished handler newest first. Reset while paused skips the already-called pending handler and cleans only lower handlers. Reset during identifier/start/abort has no payload cleanup. New start first performs reset cleanup.
- Rust futures are lazy, unlike JavaScript promises. `take_cleanup()` transfers pending reset cleanup futures to the integration executor. The executor must schedule every returned future even though its result no longer affects dispatch, including cleanup queued before another callback fails. Previously queued cleanup must be polled before the current pending callback when emulating JavaScript microtask ordering. Previously returned pending end futures remain owned by their caller. No executor, sleeps, global mutable state, unsafe code or swallowed callback failures belong in the core.

### Whole-payload adapter

- Start clears previous accumulation and overflow state. Each put appends UTF-16 units from UTF-32 input. Count UTF-16 units, not Unicode scalars or UTF-8 bytes.
- The default protocol limit is 10,000,000 units. A constructor-supplied limit replaces the upstream tests' global private-field mutation. Exactly the limit succeeds. Limit plus one clears data, permanently rejects remaining chunks in that command, and suppresses callback invocation.
- End(false), overflow, and older handled commands do not invoke the whole-payload callback. A successful command invokes it exactly once, preserving true/false results and async results.
- Synchronous end clears data immediately. Async end retains accumulation until completion; completion clears shared state, matching the upstream then callback. Next start always clears leftover data. Test overflow recovery, zero/custom limits, astral width and lone surrogates.

### Evidence and integration gate

- Run unchanged original assertions, with only import/module/type syntax adaptation, and record every discovered/passed/failed/skipped case and hashes of all inputs. Any assertion failure fails the command.
- Faithfully port all original expected callback orders and values. Grouping cases into Rust tests must not remove an assertion path. Keep the mapping below.
- Generate the deterministic corpus with seed 0x05c2019, 512 traces, 96 operations each. Replay every trace against Rust and compare the complete callback trace. Include start/put/end/reset, registration/clear/disposal, malformed identifiers, empty chunks, UTF-16 edge cases and numeric overflow. Retain any failure with case number and operation trace.
- Run warning-free focused Clippy and formatting, plus workspace gates after integration. Passing these parser checks is not proof that terminal OSC handlers or browser APIs are integrated. Those remain the escape parser, terminal API and browser owners' gates.

## Outer escape parser integration acceptance

Task91 and criterion238 cover real `EscapeSequenceParser`, not an OSC helper replacement. `scripts/escape-osc-oracle.mjs` loads unchanged source modules with private module scopes preserved. `tests/escape_osc.rs` invokes the actual native parser over every generated operation.

- Both ESC ] and C1 OSC introducers combine with BEL, C1 ST, ESC backslash, CAN, SUB and ESC followed by a non-ST final. Verify surrounding text and escaped-final fallback, not merely payload callbacks.
- Use valid, zero-padded, payloadless, identifierless, malformed and Unicode payloads. Include ignored C0 values, DEL, BOM, astral codepoints and lone surrogates.
- For every sequence run whole, split at every boundary including empty chunks, and one-codepoint chunks. Test fallback plus four registered synchronous/asynchronous/bubbling configurations. Compare every callback, print/execute/ESC/CSI fallback, parser state, preceding join state, collect and parameters at each executor turn.
- Reset during identifiers and payloads, restart an OSC during another OSC, reset while a handler is paused, and resume the same input chunk according to upstream continuation rules. Drive background cleanup before a current pending future and at explicitly matched chunk microtask boundaries. The harness yields at the same boundaries in JS and Rust, rather than suppressing cleanup or silently changing expected output.
- Reentrant removal errors propagate through real parse/reset; failed callbacks and rejected futures stop before trailing text and older handlers. Explicit reset permits documented continuation afterward.
- Seed0x05c91 adds128 traces of32 operations. The current fixture contains6598 deterministic plus128 seeded traces,3149 pending callbacks and253 resets. Input/source hashes and synthetic mutation errors are in `results/osc/escape-oracle.json`. The retained restart cleanup counterexample is `results/osc/restart-cleanup-counterexample.json`. This parser-level evidence is not a browser or terminal title/clipboard integration claim.

## Original assertion mapping

| Upstream cases | Rust test in `tests/osc.rs` |
| --- | --- |
| Identifier no report, no payload, with payload | `identifier_parsing` |
| setOscHandler, clearOscHandler, dispose handlers | `registration_clear_and_disposal` |
| addOscHandler, addOscHandler return false; four mixed sync/async cases | `bubbling_and_mixed_sync_async_cleanup` |
| Factory successful end, aborted end, disposable, false result, synchronous and asynchronous variants | `factory_success_abort_dispose_and_bubble` |
| Factory exact payload limit and limit+1 | `factory_exact_limit_and_limit_plus_one` |
| Reset abort and subsequent complete command | `reset_payload_and_resume_after_reset` |

Additional tests in `tests/osc_edges.rs` cover UTF-16 limits, lone surrogates, malformed identifiers, float arithmetic, paused reset and mutation errors. `tests/osc_failures.rs` covers synchronous/future failures and typed cause preservation. `tests/osc_registry.rs` covers reentrant registration, explicit disposal, weak ownership and self-token cycles. `tests/osc_differential.rs` replays the generated corpus; shared support modules avoid duplicate callback/future implementations.

## Commands and observed results

```sh
node scripts/check-osc.mjs --generate
node scripts/check-osc.mjs
node scripts/escape-osc-oracle.mjs
cargo test -p xterm-core --test osc --test osc_edges --test osc_failures --test osc_registry --test osc_differential --test escape_osc
cargo clippy -p xterm-core --lib --test osc --test osc_edges --test osc_failures --test osc_registry --test osc_differential --test escape_osc -- -D warnings
cargo fmt --all -- --check
```

`--generate` intentionally updates the fixture. Normal verification asserts that generated bytes match the checked-in corpus. `results/osc/oracle.json` stores original test outcomes and source/fixture hashes. The baseline observed Node v24.15.0 on Windows, with 23 discovered, 23 passed, zero failed and zero skipped tests. Rust1.85.1 passed29 tests: seven grouped upstream, eight edge, six callback-failure, three registration, one512-trace differential, and four real outer-parser integration tests. The latter replays6726 traces. Original synthetic checks also verify seven mutation/reentry and five callback failure paths. Raw output is `results/osc/rust-checks.log`; independent review logs use the `boleslav-` prefix. Focused formatting passes; see the latest retained Clippy log for the current shared-workspace result. Native Linux/macOS and browser integration are not established by this evidence. The full workspace is still under concurrent development; its gate results must be recorded separately from focused parser checks.
