# Windows wrapping callback order

Task267 replaces two inline effects with CoreTerminal's real source-style registrations.

Source `CoreTerminal._enableWindowsWrappingHeuristics` subscribes to the common linefeed emitter and registers a CSI H handler that returns false. Enabling an already-enabled policy leaves both registrations in place. Disabling disposes them. Reenabling appends a new linefeed listener and gives the CSI handler newest-first priority.

The old Rust inline code ran the workaround before all linefeed callbacks and only inside the built-in CSI H handler. It failed both listener observation order and consuming user CSI callbacks. The common linefeed emitter is now shared with the native public event. Raw `TerminalEvent::LineFeed` observation remains available, but does not redispatch the public event. `WindowsWrapping` owns only the two disposable registrations and service handles, with no terminal-owner cycle.

## Reproduce

With the original source and its esbuild dependency available:

```sh
node scripts/windows-wrapping-oracle.mjs [source-root]
cargo test --locked -p xterm-core --test windows_wrapping_order --test windows_terminal --test platform_windows --test headless_reentry --test headless_reset_host
cargo clippy --locked -p xterm-core --lib --test windows_wrapping_order --test windows_terminal --test headless_reentry --test headless_reset_host --all-features -- -D warnings
```

The optional source root is useful in an isolated worktree. The original source is only read, never patched. The generated `results/windows-wrapping/source.json` records hashes of every bundled source module and the environment. The TSV contains 15 original-terminal traces consumed by six Rust tests.

The cases cover initial constructor policy, before/after user listener registration, repeated enable, disable/reenable, policy mutation during emitter snapshot traversal, callback failure before the hook, both CSI registration orders with true/false consumption, in-place nested option mutation followed by reset, and disposal before/after the Windows LF listener. Existing `windows_terminal` tests retain conpty build, null/undefined, string coercion, LF and cursor-position assertions.

## Review correction

Independent review found that native disposal cleared the newly shared LF emitter before removing the Windows listener. Source listener removal first copies the active listener array, so later callbacks still run when the replacement array is cleared. Native disposal now removes the Windows registrations before clearing that emitter, even while the parser owns the engine. Two additional original-source traces retain both listener-order cases. The corrected scoped command passes 20 tests and affected-test Clippy.

## Limits

These are native/common event-order checks, not browser runtime evidence. Browser DOM redraw/copy, the broader full terminal suites, and native Linux/macOS gates remain separate requirements. Async queued resize replay is still task257 and is not fixed by this change.

The initial full-core run stopped at the pre-existing parse-diagnostic x0/x2 assertion in `core_services_integration.rs:188`. After adding Willie commit4f34bd0, which strengthens that assertion to the actual four source records, the gated merge ran all 1478 core tests successfully across 221 test/example suites. Its dependent all-target check then failed at existing web `character_joiner` exports and a URL-id u32/u64 fixture mismatch, so main was not advanced. Full core Clippy also reports pre-existing `format_collect` in `move_to_cell.rs` and `single_range_in_vec_init` in `input_handler_bounded.rs`. Scoped library and affected-test Clippy passes. No failing test was skipped to claim a full pass. The gated-merge log is `results/integration/berenice/gated-merge.log`.
