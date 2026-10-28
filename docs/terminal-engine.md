# Terminal engine integration

`xterm_core::terminal::CoreTerminal` executes the Rust parser, streaming decoders, real normal/alternate buffers, Unicode providers, charset service, modes, SGR, OSC handlers and device/status replies. It never delegates emulation to JavaScript. `docs/engine-contract.md` is the acceptance checklist, not a completion claim.

## Host contract

Use `new(cols, rows, scrollback)` or `with_options(Rc<OptionsService>)`. `write_utf16` preserves lone surrogates. `write_bytes` preserves streaming UTF-8 prefixes. `write` is a Rust string convenience. A returned `TerminalError::Pending` means the current chunk paused on a registered async callback, not that it was discarded. Spawn futures from `take_cleanup()` before awaiting `take_pending()`, then call `resume(bool)` with the handler's result. Do not send another chunk until the current one finishes. The public headless/browser adapters use the separately ported WriteBuffer scheduler.

`state()` borrows live state, including BufferService and its shared BufferSet. Release that borrow before mutations. `state_handle()` supports synchronous host callbacks that need to inspect or reset state while parsing. `set_event_handler` delivers parser action events outside state/queue borrows. Without a handler, `drain_events` retrieves ordered owned notifications. Input is an outbound data event, never display echo. Parser replies and diagnostics invoke CoreService/LogService outside the parser callback's mutable state borrow.

`dispose()` returns errors from disposable buffer callbacks. Drop cannot return an error and retains any disposal failure in the shared state for `TerminalState::take_error()`. Explicit disposal is the error-reporting path.

## Observed checks

On native Windows, `cargo test -p xterm-core --test terminal_engine --test terminal_engine_differential` passed eight directed integration tests and the 1,114-snapshot differential test. The oracle has 172 deterministic/seeded traces, seed `0x4354524d`. It compares both buffers, every packed cell/content/fg/bg/UTF-16/extended attribute, cursor, margins, base/display positions, core modes and ordered bell/linefeed/cursor/scroll/render/title/data events after each operation. Inputs include chunked UTF-8/UTF-16, wide/combining characters, CSI edits, SGR, alternate screen, saved cursor, resize, scroll, clear and reset. Source hashes and exact fixture hash are in `results/terminal-engine/oracle.json`.

Reproduce the source fixture with `node scripts/terminal-engine-oracle.mjs`; normal mode rejects drift. Use `--generate` only after reviewing source changes. Original failing attempts, including unwired CSI and alternate-screen render ordering, remain in `results/terminal-engine/runs/`. The retained fixture inputs reproduce those counterexamples against earlier commits.

## Remaining scope

This corpus is not complete equivalence proof. All original InputHandler assertions, malformed/random stream families, callback/logger failure and reentrancy paths, async scheduler integration, browser report/color handling, every addon and native Linux/macOS evidence still need their separately tracked tasks. In particular, the current logger action staging has an identified mode/SGR callback-error ordering gap being corrected by the handler owners. Raw negative internal cursor preconditions from JavaScript source tests do not map to unsigned Buffer fields and remain explicitly unproven. Overall equivalence is NOT PROVEN.
