# InputHandler diagnostics acceptance and contracts

Task 194 owns `terminal/diagnostics.rs`, its integration and native/browser differential assertions. Source is `source_repo/src/common/InputHandler.ts`, especially constructor fallback handlers, parse, keypad handlers and SGR diagnostics. This is real terminal behavior, not a service-only logging test. Core LogService policy remains shared with the browser adapter.

## Parse diagnostics

P1. Preserve the outer Debug-or-more-verbose gate before formatting the input. The message has two spaces before its opening quote: `parsing data  "..."`. Text contributes its original UTF-16 units. Bytes contribute one code unit per byte, not UTF-8 decoding. Include all 256 byte values, empty input, NUL, lone surrogates and split Unicode sequences.

P2. Re-read the current log level after Debug returns. Trace is emitted only when the level is exactly Trace. Its message is `parsing data (codes)`. Text passes an eagerly built array of UTF-16 code units, not a lazy callable. Byte input passes the original Uint8Array, preserving type, object identity, byteOffset and the live underlying buffer.

P3. Debug formatting occurs before its logger method getter/call. The trace argument is evaluated after Debug returns but before the trace method getter. Decode happens after both logging phases. A Debug or Trace callback may mutate the original byte view, change log level/logger, reset/dispose the terminal, or throw. Decode must observe permitted byte mutations; a thrown logger error must prevent decode, dirty-range clearing and later input mutations. Preserve previously retained UTF-8/UTF-16 decoder state after a failed call.

P4. Capture the cursor-start values before the logging callback. Dirty-range reset occurs after logging and only for a non-resumed parse. Source continuation first resumes the paused parser. If it pauses again, do not emit parse diagnostics yet. If it completes, emit diagnostics again using the original input and current log options. Small resumed input is not decoded twice. Large input uses the original 131,072-unit chunk boundaries and preserves the paused input position.

## Fallback and error diagnostics

F1. Unknown CSI logs Debug message `Unknown CSI code: ` with one ordered object, fields `identifier`, `params`. Identifier is the parser's string representation. Params faithfully represents primary and subparameter arrays.

F2. Unknown ESC uses `Unknown ESC code: ` and `{identifier}`. Unknown EXECUTE uses `Unknown EXECUTE code: ` and `{code}`. Do not invent diagnostics for ignored control characters that do not invoke a source fallback.

F3. OSC logs `Unknown OSC code: ` and ordered `{identifier, action, data}` for START/PUT/END. Identifier retains the source number domain; data retains its action-dependent undefined/string/boolean representation and exact UTF-16 chunks.

F4. DCS logs `Unknown DCS code: ` and ordered `{identifier, action, payload}`. HOOK converts Params to ordinary nested arrays before logging. PUT is UTF-16 text and UNHOOK is boolean. APC uses `Unknown APC code: ` with the corresponding START/PUT/END action and payload types. Preserve zero-length, idle/reset and failed-terminator calls exactly as the underlying parsers do.

F5. Parser errors log Error message `Parsing error: ` with the original ordered diagnostic state: position, code, currentState, collect, params and abort=false. The source returns that state unchanged; logging does not turn an otherwise recoverable parser error into an abort. A logger throw does abort immediately, retaining the original cause and the exact partial parser state.

F6. Fallible checked fallback/error callbacks are required to stop at the throw point, not at a later action boundary after another character mutates the buffer. Existing infallible parser APIs remain compatible wrappers. Max owns outer-parser additions; Amy owns OSC/APC and Sargon owns DCS additions. Every checked path needs a focused callback-throw state test.

## Command diagnostics

C1. ESC = logs `Serial port requested application keypad.` before changing current applicationKeypad and before its scrollbar notification. ESC > logs `Switching back to normal keypad.` at the same phase. A callback reset affects which current mode object is subsequently written. A throw leaves both mode and notification unchanged.

C2. DEC mode 66 and unsupported DECSET/DECRST 1005/1015 messages remain exact and in source order. Sophie owns their callback-aware mode handlers. Do not duplicate them in a second dispatch path.

C3. Unknown SGR logs `Unknown SGR attribute: %d.` plus the numeric parameter, without pre-formatting the placeholder for a custom logger. Prior attribute changes are committed before the callback; later parameters do not run if it throws. Existing SgrCursor/dispatch code already has this hook. Verify it rather than replacing its state-identity handling.

## Slow async warning

A1. Source arms an observer only when the level at pause is at most Warn. The fixed source limit is 5,000ms. Settlement clears the timer. A still-pending handler produces one direct console.warn message `async parser handler taking longer than 5000 ms`; it bypasses custom logger selection and the xterm.js prefix. Later log-level changes do not change that already armed decision.

A2. Warning does not cancel or replace the handler result. Each source pause can arm its own observer. Resolve/reject before the limit, exact-deadline ordering, multiple pauses, reset/disposal and a still-pending handler need deterministic host-clock tests. Do not add sleeps or an OS executor dependency to the core.

A3. The source's rejection sentinel is the exact string `#SLOW_TIMEOUT`. A handler itself rejecting that string triggers the same warning immediately. Other rejections propagate unchanged. A boxed String is not the sentinel. Console selection happens at warning time, and a console exception retains its original host failure behavior. Browser Promise/microtask ordering is required, not merely a native timer smoke test.

## Integration and evidence

A portable input source must expose diagnostics before making the final decode copy. The existing browser write queue retains the original byte view as WriteData::External; copying it before Debug/Trace is not acceptable. Native slice APIs may own a diagnostic byte snapshot, but browser trace arguments must be the original JS value. Exact shared interfaces will be agreed with Coleman, Antigone and Xavier before edits to their files.

Structured log arguments must preserve field insertion order and nested arrays. Reusing an alphabetically sorted options map would change observable Object.keys order. Source-derived synchronous and seeded operation traces must compare raw records and complete terminal state after both success and thrown callbacks. Save seeds and failure prefixes. Browser tests must run the actual Rust-backed Terminal in all three supported engines. Count new diagnostic assertions separately from the existing original InputHandler suite, which has no dedicated diagnostic assertions in this reference.

Execution and implementation are pending. No diagnostics coverage or equivalence is claimed from this planning document.
