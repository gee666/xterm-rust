# Scheduling and platform acceptance

This checklist controls task #58. Broad implementation criteria are not a replacement for these checks. Sources are read-only `source_repo/src/common/{TaskQueue,Platform,WindowsMode,Version}.ts` and `WindowsMode.test.ts`. `Types.ts` is an interface inventory, not a runtime implementation. Its contracts belong to the owning parser, input, buffer, service and terminal types; `TestUtils.test.ts` provides test mocks rather than shipped APIs. Track that distinction in the compatibility inventory instead of adding dead runtime types.

## Queue order and cancellation

- Enqueue schedules one future callback, not synchronous execution. Multiple enqueues before dispatch coalesce to one request.
- FIFO is strict. Returning true repeats the current task; false advances. Completed tasks are not rerun across deadlines.
- Flush runs all remaining repetitions synchronously, including tasks appended by callbacks. Flush of an empty queue is valid. A nonterminating task also makes upstream flush nonterminating; no hidden repeat limit is added.
- Clear cancels exactly the outstanding callback, drops all tasks, resets the index and can be repeated. Completed/cleared work never executes on future dispatch. A fresh enqueue schedules again.
- Cancel identifiers and request kinds are retained in differential traces. Host dispatch consumes its callback token before running work.

## Deadlines and adapters

- A batch always executes its first available task even at zero deadline.
- Measure each duration from two clock readings and clamp it to at least 1 ms. A clock moving backwards therefore uses 1 ms. NaN propagates like JS Math.max.
- Track the longest duration across this batch only. Yield exactly when longest times 1.5 is greater than the latest time remaining. Equality does not yield.
- Query the deadline initially and after every task. The queue reschedules on a tight deadline even when that was the final task; the next empty dispatch clears it.
- Warn only if previous time remaining minus the just-finished duration is strictly less than -20 ms. Preserve JS rounding at half boundaries and exact warning magnitude.
- Priority callbacks create a 16 ms deadline at dispatch time and clamp remaining time at zero. Idle callbacks use the supplied host deadline. Missing idle support selects priority callbacks.
- The portable scheduler cannot itself observe a browser event loop. A separate browser integration must wire timer/idle APIs, cancellation, live dispatch and logging before browser scheduling equivalence can be claimed. A deterministic fake scheduler proves the algorithm only.

## Reentrancy, debounce and lifetime

- No queue-state borrow is held across tasks. Enqueue and clear during callbacks reproduce upstream index changes, including skipping an appended task when clear then enqueue is followed by a false return.
- Recursive flush is permitted with caller-controlled termination. Callbacks can mutate captured Cell state. Task closures receive the queue to avoid strong reference cycles.
- Debounced set cancels previous scheduled work, keeps only the latest callback and preserves callback repetition. Flush executes the latest task. Dispose cancels pending work; a later set is accepted, as upstream permits.
- Last-handle drop cancels any host token. Cloning a handle preserves shared state and must not cancel before the last handle drops.

## Platform and version

- Node requires process.title presence and either absent navigator or a Node.js/ user agent. A process shim plus a real browser navigator is not Node.
- Browser flags retain case-sensitive Firefox/Chrome/Edge substring rules. Safari uses the actual start-anchored case-insensitive regex: the first safari must occur before any line terminator and before a chrome or android substring. Chrome after Safari does not disqualify the match.
- Mac and Windows use the exact upstream platform allowlists; Linux uses the case-sensitive Linux substring. CrOS requires ASCII word boundaries. Node forces platform/userAgent to node and all browser/OS flags false.
- Safari major version uses the first case-sensitive Version/ followed by ASCII digits. Missing/invalid version returns zero; non-Safari always returns zero.
- Zoom factor is 1 regardless of device pixel ratio because that is the actual source contract.
- Missing navigator outside Node is a recoverable detection error, not a guessed OS. Version equals upstream XTERM_VERSION 6.0.0.

## Windows wrapping

- Port the three unchanged upstream assertions: a final non-space character marks wrapped, final U+0020 clears it, final null cell clears it.
- An absent previous line does not alter the target. A missing or negative cell acts as null and clears wrapped, because legacy BufferLine.get always returns an array with bitwise-coerced content. Previous index is ybase+y-1, target is ybase+y. Include first-row signed ring indexing, rotated storage, nonzero scrollback base, absent target, zero cols, combined cells, non-ASCII, tab and existing wrapped=true cases. An empty combined string yields NaN from charCodeAt and therefore marks wrapped.
- Only null and U+0020 are excluded. Other Unicode whitespace and zero-width combined content use the actual final codepoint contract.
- Run the helper on real BufferLine storage and integrate it with the real BufferService/CoreTerminal paths. Isolated helper tests do not prove that integration.
- CoreTerminal.ts enables the heuristic only when windowsPty is present, backend is conpty, buildNumber is defined and is less than 21376 under JS numeric comparison. Check undefined/null/NaN and 21375/21376 boundaries, option toggles and raw-object mutations separately.
- Enabling registers both a linefeed listener and a CSI H handler. The CSI H hook updates the current row before cursor-position handling and returns false so dispatch continues. Preserve reverse handler precedence, registration time, public listener observations and option-change disposal. Repeated enabling must not duplicate registrations. Disable/dispose removes both registrations. Test resize/redraw/copy effects in the actual terminal.

## Evidence

`node scripts/task-platform-oracle.mjs` must execute unchanged applicable upstream assertions and deterministic JS/Rust traces, record exact counts, seed, source hashes, environment and commands under `results/task-platform/`, and preserve counterexamples on mismatch. Run focused Rust tests, rustfmt and Clippy. Native Windows evidence is local; Linux/macOS and live browser adapter checks remain explicitly unrun until observed. Independent review must inspect implementation and rerun checks.
