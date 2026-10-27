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

- Registrations are ordered by insertion, dispatched newest first for start, put and end. Token drop is not disposal. Explicit token disposal is idempotent and removes the first equal handler identity, matching JavaScript's `indexOf`.
- Multiple registrations of the same identity remain separate list entries. Clearing an identifier detaches its registry entry without changing a command already using the old list. New registrations after clear form a new list.
- No registered handlers means fallback receives identifier, action and action-specific value. An emptied active list uses fallback on subsequent puts/end. Disposal clears registry, fallback and active list without sending end callbacks.
- Do not hold a handler-list borrow across callbacks. Callbacks may dispose registration tokens. Preserve live-list traversal rather than taking a snapshot. If mutation removes an index the ongoing traversal needs, return `DispatchError::ActiveHandlerRemoved`, the recoverable Rust equivalent of upstream's TypeError. Do not skip it silently or panic. Preserve partial state on error.

### Bubbling, futures and reset

- End sends its success flag newest first until one handler returns true. All older handlers still receive end(false) for cleanup. False continues bubbling, including when command success is false.
- A pending handler pauses traversal. The caller awaits its future and resumes using the same success flag and resolved boolean. Resume starts one index below the pending handler, never repeats it. Pending cleanup resumes cleanup regardless of its result.
- Test sync/async/sync and async/sync/async orders for first-handler-wins and all-false bubbling. Include pending results in both execution and cleanup passes.
- Reset during payload aborts every unfinished handler newest first. Reset while paused skips the already-called pending handler and cleans only lower handlers. Reset during identifier/start/abort has no payload cleanup. New start first performs reset cleanup.
- Rust futures are lazy, unlike JavaScript promises. `take_cleanup()` transfers pending reset cleanup futures to the integration executor. The executor must drive every returned future even though its result no longer affects dispatch. Previously returned pending end futures remain owned by their caller. No executor, sleeps, global mutable state, unsafe code or swallowed callback failures belong in the core.

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

## Original assertion mapping

| Upstream cases | Rust test in `tests/osc.rs` |
| --- | --- |
| Identifier no report, no payload, with payload | `identifier_parsing` |
| setOscHandler, clearOscHandler, dispose handlers | `registration_clear_and_disposal` |
| addOscHandler, addOscHandler return false; four mixed sync/async cases | `bubbling_and_mixed_sync_async_cleanup` |
| Factory successful end, aborted end, disposable, false result, synchronous and asynchronous variants | `factory_success_abort_dispose_and_bubble` |
| Factory exact payload limit and limit+1 | `factory_exact_limit_and_limit_plus_one` |
| Reset abort and subsequent complete command | `reset_payload_and_resume_after_reset` |

Additional tests in `tests/osc_edges.rs` cover UTF-16 payload limits, lone surrogates, empty/illegal identifiers, floating identifier arithmetic, reset during pause, and mutation errors. `tests/osc_differential.rs` replays the generated corpus. `tests/parser_support/mod.rs` shares callback reporting with APC.

## Commands and observed results

```sh
node scripts/check-osc.mjs --generate
node scripts/check-osc.mjs
cargo test -p xterm-core --test osc --test osc_edges --test osc_differential
cargo clippy -p xterm-core --lib --test osc --test osc_edges --test osc_differential -- -D warnings
cargo fmt --all -- --check
```

`--generate` intentionally updates the fixture. Normal verification asserts that generated bytes match the checked-in corpus. `results/osc/oracle.json` stores original test outcomes and source/fixture hashes. The baseline observed Node v24.15.0 on Windows, with 23 discovered, 23 passed, zero failed and zero skipped tests. Rust1.85.1 passed seven grouped upstream tests, seven edge tests and one differential test replaying all512 traces. Focused Clippy and formatting passed. Raw output is `results/osc/rust-checks.log`. Native Linux/macOS and browser integration are not established by this evidence. The full workspace is still under concurrent development; its gate results must be recorded separately from focused parser checks.
