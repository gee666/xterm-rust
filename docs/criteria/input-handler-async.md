# Live InputHandler asynchronous acceptance

The reference is the final `InputHandler - async handlers` describe in `source_repo/src/common/InputHandler.test.ts`. It contains three cases. The surrounding synchronous/control suites are separate obligations, not counted as covered by these three cases.

## Faithful original assertions

- Execute the three original cases unchanged through the actual source InputHandler, including its real BufferService/CoreService and original beforeEach setup at80 columns and30 rows. Preserve bodies/assertions and promise continuation order.
- Port `async CUP with CPR check`: record both parameter arrays from late asynchronous CUP handlers and both CPR replies after the cursor moves. For `aaa ESC[3;4H ESC[6n bbb ESC[6;8H ESC[6n`, assert CUP parameter vectors equal parsed CPR vectors exactly. Use the actual Rust terminal engine and queue, not a parser-only model or a test substitute for cursor movement.
- Port `async OSC between`: after the callback's promise boundary but before it resolves, assert the first two lines are `hello world!` and empty, and payload is `some data`. After completion assert `hello world!` and `second line`.
- Port `async DCS between`: preserve the same intermediate/final line assertions, exact payload and parameter vector `[1,2]` for `ESC P1;2a ... ESC \\`.
- Rust public callback registration may delegate a false asynchronous CUP result to the already registered real built-in CUP handler instead of invoking private InputHandler.cursorPosition directly. This is a binding-level call difference only. The late cursor effect, callback ordering and all original assertions remain mandatory and must be documented.

## Continuation and integration boundaries

- Through a real QueuedTerminal, no bytes after a pending CSI/OSC/DCS handler and no later queued chunk may parse until it resolves. Completion callbacks and batched parsed events must follow resumed parsing in source order.
- Preserve true handling versus false bubbling into earlier handlers. Rejection reports the original typed error before false continuation, and a synchronous callback failure propagates without pretending completion.
- Reset while pending, disposing a handler/terminal while pending and async reset cleanup must preserve saved parser/input stack rules. Poll owned cleanup futures independently, forward their errors and retain counterexamples. Never leave silently dropped futures or successful no-ops behind an accepted test.
- Retain UTF-16 payload/Params ownership across await. Add split-byte/string boundaries around introducers, payload and terminators and prove no chunk is decoded twice on resume.
- Run deterministic source/native ordered traces for at least pending resolution/rejection and lifecycle cases. Record discovered/mapped/executed/passed/failed/skipped/unrun separately; unresolved source differences remain failures, not rewritten expectations.
- Keep every production module independent of browser and OS. Tests must execute actual Rust InputHandler-equivalent handlers, buffer/cell mutations, replies and public/native queue callbacks. Run warning-free focused formatting/Clippy and independent source review. Native Windows is not Linux/macOS evidence.

Overall xterm equivalence remains NOT PROVEN until all other functionality and verification gates pass.
