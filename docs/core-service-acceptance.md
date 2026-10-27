# Core and log service acceptance

Willie owns `crates/xterm-core/src/core_service.rs`, `log_service.rs`, their tests, oracle and this document. Source references are `source_repo/src/common/services/CoreService.ts`, `CoreService.test.ts`, `LogService.ts`, and `Services.ts`. Other common services and public adapters have separate tasks. UnicodeApi belongs to task 23. MouseStateService belongs to task 57.

## State and reset

- Port all three upstream CoreService test cases and their four assertions without changing expected values: default cursor initialization is false, showCursorImmediately true initializes it, and reset preserves both true and false cursor initialization.
- Raw showCursorImmediately uses the source nullish fallback, not boolean replacement. Null/undefined become false; all other primitive/object identities survive until the first typed boolean write. Rust CursorInitialization.value exposes that exact value; get uses truthiness for engine visibility. Option changes and reset must not overwrite it.
- Initial cursor-hidden is false. Defaults are insert false; application cursor/keypad, bracketed paste, color-scheme updates, origin, reverse wraparound, send focus, synchronized output and win32 input false; wraparound true; cursor blink/style absent.
- Kitty state starts with flags/mainFlags/altFlags zero and independent empty main/alternate stacks. Reset clears every mutated mode, override, flag and stack. Reset does not change either cursor flag and does not replace event subscriptions.
- Instances must not share mutable modes or stack storage. Unknown Kitty flag bits remain representable.

## Data, binary and callbacks

- disableStdin blocks data, binary, scroll requests, user-input events and logging. Read it on each call, including after live option updates.
- Data sends fire scroll-to-bottom only for user input with scrollOnUserInput truthy and ybase different from ydisp. Test the full boolean truth table, equal viewport positions, both position orderings and live option changes.
- Data event ordering is scroll request, user input, debug log, trace log, data. Non-user data skips scroll/user-input. Binary sends only debug, trace and binary, never scroll/user-input.
- Empty text still emits. UTF-16 strings, embedded NUL, non-BMP characters and lone surrogates survive data/binary events exactly. Trace code arrays use UTF-16 code units, not Unicode scalars or UTF-8 bytes.
- A synchronous listener may update options, dispose subscriptions, dispose the service or trigger another send. No internal borrow or lock may cross callbacks. Preserve the upstream read points: disableStdin is checked once at entry; scroll options before scroll; logger before data.
- Dispatch failures propagate as errors and stop later callbacks, rather than silently continuing. Disposal clears all four event listener lists. Dropping the Rust owner performs the same cleanup. Reset does not dispose events.

## Logging

- Preserve trace/debug/info/warn/error/off thresholds and numeric ordering. Each level admits messages at or above it. Off admits none. Test all 30 level/message combinations.
- Read initial logLevel and respond synchronously to live logLevel option changes. Disposal unregisters that listener and freezes its last value. Invalid raw levels must match upstream suppression rather than assuming a valid level.
- Evaluate lazy optional parameters only for admitted messages, exactly once, left to right. Never evaluate for filtered levels. Keep typed UTF-16 messages and parameter values. A fallible lazy parameter returns its original error before later parameters or the sink execute.
- A custom logger receives no prefix. Default console routing uses `xterm.js: ` with trace/debug mapped to log, info to info, warn to warn, error to error. A portable Rust sink receives the route explicitly; browser/native adapters implement the actual console endpoint.
- Logger replacement is observed at send time. No borrowed mutable state may span a logger callback. Record whether custom logger callbacks and default sink bindings have been exercised independently of the terminal integration.

## Evidence and integration gate

A deterministic upstream oracle must compare complete mode/reset snapshots and ordered event/log traces, including malformed UTF-16, every input/scroll combination and live option changes. Record seed, case counts, commands and retained logs. Run focused Rust tests, rustfmt and warning-denied Clippy. An independent reviewer must inspect code and execute the checks. Terminal integration remains unproven until the terminal engine actually uses these services. Local service tests alone cannot close the full terminal criterion.
