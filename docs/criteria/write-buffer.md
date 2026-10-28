# WriteBuffer acceptance

Reference: `source_repo/src/common/input/WriteBuffer.ts` and every assertion in `WriteBuffer.test.ts`. Scope is the portable queue plus real host/terminal integration, not an isolated successful queue smoke test.

## Queue, synchronous writes and failures

- Preserve FIFO text/byte/mixed chunk order and callback order, including empty text and empty byte arrays. Text length is UTF-16 units; byte length is bytes. Browser byte views retain identity until parsing, so mutations, detachment and resizable-buffer length changes remain visible. Dynamic length is read at enqueue and after parsing, preserving temporarily negative pending counts when a view grows.
- Reject asynchronous writes only when already-pending data is strictly greater than 50,000,000 units before adding the new chunk. Preserve exact error text. Equality is accepted; the crossing chunk is accepted; the next write is rejected. No policy substitution or hidden truncation.
- Start one zero-delay task only on transition from an empty backing queue. User input makes the first new write run immediately, consumes that flag only on the empty transition, and preserves older queued work.
- `write_sync` drains synchronously and allows action/callback reentry without growing the native stack. Match the strict recursion-count comparison, reset behavior and maximum-subsequent-calls data loss from the deprecated source API.
- Preserve the JavaScript truthiness distinction: a synchronous loop stops at an empty text chunk, while an empty Uint8Array remains truthy. Preserve the source's independent callback queue and offset state rather than silently fixing this quirk.
- `flush_sync` ignores calls during a synchronous drain, fires parsed once only if it processed a truthy chunk, clears queue/callback state, and leaves a scheduled timer harmless through the sentinel offset. `write_sync` does not fire parsed.
- Synchronous action, callback and parsed-listener failures propagate. State changes already committed before a throw remain observable. Do not catch and turn failures into successful no-ops.

## Scheduling, asynchronous handlers and disposal

- Record the start clock once per slice, carry it across fast pending-handler continuations and yield at elapsed >=12ms. Continuations resolved at or after the boundary request a zero-delay host task; earlier resolution resumes immediately.
- Paused handlers keep the current chunk/offset and do not decrement pending units or invoke its callback. Resume forwards the Boolean result to the action, with default true on ordinary timer dispatch. Multiple pauses in one chunk preserve order.
- A rejected handler reports its original typed error object through the host's microtask error path before false continuation runs. It must not stringify/swallow the error or abort all queued input. Immediate parser failures propagate synchronously; all HandlerResult variants are handled explicitly.
- After each completed chunk, call the callback before incrementing offset and subtracting that chunk's units. Preserve action/callback reentrant writes and source order.
- If work remains after a budget break, retain processed chunks until offset >50, compact exactly then, request the next task and fire parsed after scheduling. When exhausted, clear both queues/counters and fire parsed once.
- Disposal cancels the scheduled task, clears queues/pending units/offset and disposes parsed subscriptions. All writes/flushes after disposal do nothing. Pending future completion after disposal neither resumes the action nor schedules work. Repeated disposal is safe. Dropping the final Rust owner cancels scheduled host resources without an ownership cycle.
- Host scheduling never invokes the queue inline while the scheduler is borrowed. Host callbacks use weak queue handles. The core does not depend on an operating system, browser or executor.
- Connect the queue to CoreTerminal and the browser/native adapter with real scheduling and pending-future execution. An explicit scheduler trait without a working caller is not integrated terminal parity.

## Verification

- Execute all 19 original WriteBuffer cases unchanged as JavaScript baseline and port each assertion, including the 10,000-step stack safety and recursion limit last='11'. Confirm the discovered case count from source, do not assume this paragraph's count if upstream changes.
- Use a controlled clock and scheduler, not sleeps, to test exact12ms and50-chunk boundaries, continuation resolve/reject, callback/event reentry, disposal during pending work, empty-string quirks and exception partial state.
- Retain seeded JavaScript/Rust operation traces and complete queue/event/action/callback observations with source hashes, seed, counts and exact commands.
- Run focused tests and warning-free formatting/Clippy, then terminal write/resize integration and actual browser scheduling checks. Independent review must execute checks and inspect source. Record native Windows separately from unrun native Linux/macOS evidence.

Overall xterm equivalence remains NOT PROVEN while integration, source mapping, browser checks or platform evidence is missing.
