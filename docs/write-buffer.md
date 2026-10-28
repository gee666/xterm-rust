# WriteBuffer port

`crates/xterm-core/src/input/write_buffer.rs` owns the FIFO state and host timer. `write_buffer_drain.rs` implements synchronous drains, timed slices and pending-handler continuation. `write_buffer_types.rs` defines the data, error and host contracts. The core adds no dependency, runtime or browser requirement.

## Callable contract

Construct `WriteBuffer::new(scheduler, action)`. The action receives the queue handle, borrowed `WriteData` and `Option<bool>` parser continuation. `None` is the source's synchronous action call without a promise result. Timed writes pass `Some(true)` initially and carry the resolved result through the entire continuation slice. The action returns `Result<WriteAction, WriteError>`.

- `WriteData::Text(Vec<u16>)` retains UTF-16 units, including lone surrogates. `Bytes(Vec<u8>)` owns native bytes. `External(Rc<dyn WritePayload>)` retains adapter-owned byte views until parsing, preserving JavaScript Uint8Array mutation, detachment and resizing. The queue rechecks dynamic length after parsing, even when a resized view makes the source pending count temporarily negative.
- `write(data, Option<WriteCallback<S>>)` schedules a zero-delay write unless the user-input fast path applies. Callback closures receive the queue for safe reentry.
- `write_sync(data, Option<f64>)` ports the deprecated source method, including strict recursion-limit comparison and dropped data when the limit rejects a call.
- `flush_sync()` ports the source's synchronous flush. Neither synchronous method waits for pending parser handlers.
- `handle_user_input()`, `on_write_parsed()` and `dispose()` map directly to source behavior. Disposal is explicit and idempotent; dropping the final owner cancels its timer.
- `WriteAction::from_parser(Option<HandlerResult>) -> Result<WriteAction, WriteError>` explicitly adapts every shared parser result, including immediate failures and fallible futures. The terminal action must retain its decoded chunk while pending, call the parser's resume method with the continuation result, and not decode the same chunk again.

`WriteScheduler` provides `request`, `cancel`, `now`, `spawn` and `report_error`. A requested timer stores the weak queue and clock/result arguments. When fired later, upgrade the weak queue and call `process(last_time, result)`. Requests and future polling must not call back inline while the scheduler is borrowed. Request failure returns `WriteError::Schedule`, propagated after releasing the scheduler borrow. The host reports uncaught asynchronous errors rather than discarding them. `WriteError::external(error)` retains an original typed parser/host error object by shared identity, and `external_ref` recovers that type without string conversion.

`spawn` executes local futures without requiring Send. A rejected handler reports its error and resumes with false. Dropped/disposed queues cannot be resumed by a completed future. Stored callbacks should use their queue argument, not capture a strong queue handle and create an ownership cycle.

## Preserved source quirks

The backing data and callback queues are separate. In synchronous loops an empty text chunk is falsy and terminates the loop before shifting its callback. An empty byte array is truthy and does not. A synchronous write leaves a sentinel offset and does not emit parsed. A synchronous flush clears remaining queued data/callbacks and emits parsed only when it processed a truthy chunk. These are not silently repaired.

The discard check compares already-pending units strictly greater than 50,000,000 before accepting a new chunk. Exactly 50,000,000 is accepted, as is a chunk that crosses the watermark. The following asynchronous write fails with the original message.

Synchronous failures retain the source's already-mutated state. In particular an action throw in a synchronous drain leaves the source's in-progress flag set. Returning an error is not transactional rollback.

Time slices yield at elapsed >=12ms. Processed chunks remain in the backing queue until a yielded offset is strictly greater than50. Parsed is emitted after rescheduling a remaining slice, not before it. Controlled-clock tests cover both strict thresholds.

## Original assertion mapping

The source file contains19 cases. All are executed unchanged by `scripts/write-buffer-oracle.mjs`; only imports/type-only names are transformed in temporary modules. The original done-callback timers remain part of the source baseline. Supplemental Rust tests do not use sleeps.

All Rust paths below are under `crates/xterm-core/tests/write_buffer_cases/`.

| Original case | Rust upstream.rs test |
| --- | --- |
| string | string |
| bytes | bytes |
| string/bytes mixed | string_bytes_mixed |
| write callback works for empty chunks | write_callback_for_empty_chunks |
| writeSync | write_sync |
| writeSync action recursion, issue3265 | write_sync_reentry_does_not_overflow_callstack |
| writeSync maxSubsequentCalls | write_sync_max_subsequent_calls |
| flushSync all pending | flush_sync_all_pending |
| flushSync no pending no-op | flush_sync_no_pending_is_noop |
| flushSync fires onWriteParsed | flush_sync_fires_parsed |
| flushSync empty does not fire | flush_sync_no_pending_does_not_fire_parsed |
| dispose cancels scheduled innerWrite | dispose_cancels_scheduled_inner_write |
| dispose no parsed | dispose_does_not_fire_parsed_for_pending |
| write after dispose | write_after_dispose_is_noop |
| dispose idempotent | dispose_is_idempotent |
| async handler continuation skipped after dispose | async_handler_continuation_skipped_after_dispose |
| handleUserInput fast path | handle_user_input_processes_first_chunk_synchronously |
| flushSync after dispose | flush_sync_after_dispose_is_noop |
| writeSync after dispose | write_sync_after_dispose_is_noop |

The original async-dispose case disposes before its timer invokes the handler. The faithful port retains that order. `timing.rs` adds a second test that actually starts and pauses a handler before disposal.

## Evidence and reproducible commands

```text
node scripts/write-buffer-oracle.mjs
cargo test -p xterm-core --test write_buffer
cargo clippy -p xterm-core --test write_buffer -- -D warnings
cargo fmt --all -- --check
```

The oracle records source SHA256 values, exact original outcomes, Node/platform, seed `0x5717eb0f`, 128 traces and8192 steps under `results/write-buffer/oracle.json`. Its retained fixture is `tests/fixtures/write-buffer-traces.tsv`. The replay compares action data and continuation values, callback order, parsed events, pending units, backing queue length, offsets, disposal and scheduled-task counts after every operation. Costs vary between0,3,6,12 and13ms per action.

`timing.rs` and `edges.rs` supplement that synchronous/timer corpus with exact watermark, UTF-16 lengths,50/51 compaction,12ms continuation, false and rejected results, disposal of an actually pending future, callback reentry, empty chunk truthiness, host request errors, source partial throw state and final-owner cleanup.

Observed native Windows checks: all19 original tests pass,8192 seeded steps match,42 Rust tests pass, and targeted Clippy is warning-free. `results/write-buffer/final-{oracle,tests,clippy}.log` retain these command outputs. The extra source assertions record rejection/error ordering plus actual mutable, detached and resizable JavaScript byte views. Earlier shared-build failures remain in the same results directory rather than being removed.

Observed checks run on native Windows. A clean queue result is not evidence of terminal integration or browser scheduling. Native adapter work belongs to Antigone; browser adapter work belongs to Xavier. The detailed criteria in `docs/criteria/write-buffer.md` retain these integration gates. Overall equivalence is NOT PROVEN. Native Linux/macOS and integrated browser/terminal checks remain unrun until their actual results are recorded.
