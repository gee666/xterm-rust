# Terminal engine integration

`xterm_core::terminal::CoreTerminal` executes the Rust parser, streaming decoders, real normal/alternate buffers, Unicode providers, charset service, modes, SGR, OSC handlers and device/status replies. It never delegates emulation to JavaScript. `docs/engine-contract.md` is the acceptance checklist, not a completion claim.

## Host contract

Use `new(cols, rows, scrollback)` or `with_options(Rc<OptionsService>)`. `write_utf16` preserves lone surrogates. `write_bytes` preserves streaming UTF-8 prefixes. `write` is a Rust string convenience. A returned `TerminalError::Pending` means the current chunk paused on a registered async callback, not that it was discarded. Spawn futures from `take_cleanup()` before awaiting `take_pending()`, then call `resume(bool)` with the handler's result. Do not send another chunk until the current one finishes. The public headless/browser adapters use the separately ported WriteBuffer scheduler.

`state()` borrows live state, including BufferService and its shared BufferSet. Release that borrow before mutations. `state_handle()` supports synchronous host callbacks that need to inspect or reset state while parsing. `set_event_handler` delivers parser action events outside state/queue borrows. Without a handler, `drain_events` retrieves ordered owned notifications. Input is an outbound data event, never display echo. Parser replies and diagnostics invoke CoreService/LogService outside the parser callback's mutable state borrow.

`dispose()` returns errors from disposable buffer callbacks. Drop cannot return an error and retains any disposal failure in the shared state for `TerminalState::take_error()`. Explicit disposal is the error-reporting path.

## Observed checks

The earlier native Windows checkpoint passed eight directed integration tests and the original 1,114-snapshot differential test. The expanded Windows run now passes 9,306 snapshots across 684 traces, seed `0x4354524d`. The original 172 traces and all 1,114 snapshot rows remain an unchanged byte-for-byte prefix of the fixture. Another 512 seeded traces contain 16 operations each, mixing malformed UTF-8 byte chunks, arbitrary UTF-16 units including lone surrogates, alternate-screen modes, large edit counts, reset, clear, scrolling and resizing. It compares both buffers, every packed cell/content/fg/bg/UTF-16/extended attribute, cursor, margins, base/display positions, core modes and ordered bell/linefeed/cursor/scroll/render/title/data events after each operation. Inputs include chunked UTF-8/UTF-16, wide/combining characters, CSI edits, SGR, alternate screen, saved cursor, resize, scroll, clear and reset. Source hashes and exact fixture hash are in `results/terminal-engine/oracle.json`.

`node scripts/engine-replies-oracle.mjs` also passed all 20 selected unchanged original InputHandler cases and their 20 Rust counterparts. They cover save/restore origin/wrap, cursor-style values, every original window-option case, XTVERSION default/zero/nonzero branches and six DECSTR reset cases. Per-attempt reports and source hashes are under `results/terminal-engine/replies/`. Other InputHandler suites remain separately tracked.

Reproduce the source fixture with `node scripts/terminal-engine-oracle.mjs`; normal mode rejects drift. Use `--generate` only after reviewing source changes. Original failing attempts, including unwired CSI and alternate-screen render ordering, remain in `results/terminal-engine/runs/`. The retained fixture inputs reproduce those counterexamples against earlier commits.

## Expanded corpus verification

The native replay deliberately matches the oracle's public headless entry points. In particular, public `Terminal.resize` returns before its core call when both dimensions are unchanged. Calling raw `CoreTerminal.resize` instead resets margins and clamps a pending wrap cursor. The extended corpus exposed this adapter error in `extended-480`. The replay now performs the same public boundary check. No expected source state or event was changed. `results/terminal-engine/expanded-before-fix.txt` retains all ten mismatched snapshots from the previous adapter.

Observed with Node 24.15.0 and Rust 1.85.1 on native Windows MSVC:

```text
node scripts/terminal-engine-oracle.mjs --generate
cargo test -p xterm-core -j2 --test terminal_engine_differential --test headless_queue --test headless_queue_edges --test native_queue_differential
cargo clippy -p xterm-core -j2 --test terminal_engine_differential -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/tests/terminal_engine_differential.rs
node --test scripts/oracle-fixture.test.mjs
```

The focused native command passed 16 test functions, including the 9,306-snapshot full-state replay and the separate 24-step queue fixture. The fixture helper passed five tests. The focused Clippy and formatting checks passed. This is not the full workspace quality matrix. The TSV exceeds the normal file-length limit because it is a generated, unchanged-source behavioral fixture, not implementation code.

The generator continues the original xorshift32 state after the first 128 seeded traces. It does not regenerate those traces from the expanded token list. Short random chunks exercise streaming decoder state across subsequent operations. They do not cover every byte sequence or every chunk boundary. The source report records 684 discovered/executed traces and 9,306 generated snapshots; the native summary records 9,306 passed and zero failed, skipped or unrun snapshots for this corpus only. Source hashes, fixture hash and counts are retained in `results/terminal-engine/oracle.json` and `rust-summary.txt`.

A larger exploratory source run exceeded its 180-second process deadline. The bounded 512-trace extension completed under a 240-second subprocess deadline. This was not treated as a passing attempt or used to remove failing source assertions. Normal verification without `--generate` still rejects fixture drift. New temporary source bundles and Cargo outputs used the owned O: worktree; cleanup is separate from durable source/fixture/results retention.

## Remaining scope

This corpus is not complete equivalence proof. All original InputHandler assertions, malformed/random stream families, callback/logger failure and reentrancy paths, async scheduler integration, browser report/color handling, every addon and native Linux/macOS evidence still need their separately tracked tasks. In particular, the handler owners are correcting mode/SGR callback-error ordering, pre-cell accessibility callback timing and captured attribute identity across reset. Internal RIS reset-request notifications also need comparison against the richer source trace. Direct addon LF/dirty-row hooks were added after the observed tests and remain unrun while local builds are paused for low disk space. Raw negative internal cursor preconditions from JavaScript source tests do not map to unsigned Buffer fields and remain explicitly unproven. Overall equivalence is NOT PROVEN.
