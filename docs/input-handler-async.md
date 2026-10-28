# Live InputHandler asynchronous assertions

Task150 ports the final three cases in `source_repo/src/common/InputHandler.test.ts`. The required contracts are in `docs/criteria/input-handler-async.md`. This is not coverage for every InputHandler method or the rest of that test file.

## Original source baseline

`scripts/input-handler-async-baseline.mjs` bundles the complete unchanged test file and its dependencies with the reference's existing esbuild. Mocha selects `InputHandler - async handlers`; it does not extract or rewrite any assertion body. BeforeEach hooks remain intact. The original80-column/30-row setup, asynchronous cursor moves and emitted CPR replies run through source InputHandler/BufferService/CoreService.

Observed on native Windows before the disk-pressure pause:

- Discovered/executed/passed3, failed/skipped0.
- `results/input-handler-async/baseline.json` records exact case names, dependency hashes, Node/esbuild/Mocha versions and scoped counts.
- `results/input-handler-async/baseline.log` retains original output, including both CPR replies.

## Rust assertion mapping

`tests/input_handler_async.rs` uses the actual native QueuedTerminal, CoreTerminal engine, handlers, buffers and parser continuation. Every callback yields once through the native waker-driven pump, with no sleep or test replacement for terminal logic.

| Original case | Rust case |
| --- | --- |
| async CUP with CPR check | async_cup_with_cpr_check |
| async OSC between | async_osc_between |
| async DCS between | async_dcs_between |

The CUP callback records the original parameter vectors, then returns false after its asynchronous boundary so the already registered real CUP handler runs. This differs from the source test's direct call to private InputHandler.cursorPosition, not from the effect being asserted. The original exact CUP-versus-CPR assertion remains, with an additional check for `[3,4]` and `[6,8]`. OSC/DCS preserve payload, parameter and both intermediate/final line assertions unchanged.

`tests/input_handler_async_lifecycle.rs` adds public reset while a real handler is paused, and disposal of the actual terminal during a paused handler. Reset must preserve saved parse/decode state without replaying the prefix. Disposal must cancel outer callbacks and prevent resumed display writes.

`tests/input_handler_async_streaming.rs` varies every UTF-8 byte and UTF-16 unit boundary around real OSC/DCS streams containing wide and astral text plus a combining suffix. Each trace must invoke the handler once, retain exact payload/Params, complete two callbacks and print the input text once. These are additional Rust integration assertions, not extra original cases.

The small host/line helpers intentionally remain local to these standalone suites. Their setup and completion conditions differ, and introducing another generic test executor would obscure the actual pause/resume paths.

## Lifecycle differential traces

`scripts/input-handler-async-traces.mjs` is prepared to record six original real-terminal cases: handled, false bubbling, rejected future, reset while paused, disposal while paused and disposing a registration while paused. It preserves source exception records and logs before, during and after continuation. It has not run during the disk pause; no fixture or Rust parity result is claimed yet.

## Current execution status

The three original JavaScript tests passed. The nine Rust tests have been written and formatted but are UNRUN while the quality owner coordinates the shared-disk pause. The lifecycle differential script is also UNRUN. No test is ignored, weakened or replaced by a successful no-op.

After resource clearance, execute:

```text
node scripts/input-handler-async-baseline.mjs
node scripts/input-handler-async-traces.mjs
cargo test -p xterm-core --test input_handler_async --test input_handler_async_lifecycle --test input_handler_async_streaming
cargo clippy -p xterm-core --test input_handler_async --test input_handler_async_lifecycle --test input_handler_async_streaming -- -D warnings
```

Retain new failures rather than overwriting the original baseline or treating a source-only pass as Rust coverage. Add a Rust replay for the generated lifecycle fixture, independently review source/code and execute current gates before requesting task acceptance. Native Linux/macOS and browser execution are separate requirements. Overall equivalence is NOT PROVEN.
