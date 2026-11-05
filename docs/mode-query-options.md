# DECRQM raw option getters

Task 523 repairs ANSI mode 20 and DEC private mode 12 queries. This is a bounded parser repair, not completion of the parser family or the port. Task 313 and the integrated browser, platform and comparison tasks remain open. Overall equivalence is NOT PROVEN.

## Source contract and defect

`source_repo/src/common/InputHandler.ts` requestMode reads `rawOptions.convertEol` for ANSI 20 and `rawOptions.cursorBlink` for DEC 12. JavaScript getters can inspect or change the terminal, resize, reset, change the active buffer or throw. A successful getter returns an arbitrary raw value whose JavaScript truthiness chooses status 1 or 2. A throwing getter sends no reply and leaves its earlier mutations intact. The source reports only the first main parameter.

At main `1106d480`, `mode_handlers.rs` released state around the nested option gates for DEC 3 and 9001, but `mode_query.rs` read these two direct options under mutable TerminalState. Ordinary native getters that inspected state therefore failed with Borrowed. Reset and resize getters could not run either. This was not an issue in the protocol status constants.

The query handler now captures the option service with the existing core/column snapshots, reads the relevant direct or nested option without state borrowed, then calculates the status and enqueues the response. Status calculation no longer invokes getters. Existing DEC 3 numeric-column capture and DEC 9001 captured mode-object behavior remain unchanged and are covered by the retained mode tests.

The local producer starts at main1106 and owns only `mode_handlers.rs`, `mode_query.rs` and new regression artifacts. Accepted task 513 removes the other caller of `TerminalState::option_bool`. Once both producers are combined, the integration owner must delete that now-unused helper from `terminal/state.rs`. It cannot be deleted in this standalone producer because main1106 ED2 still calls it. Gordon acknowledged this integration requirement on post 669.

## Executable checks

```sh
XTERM_SOURCE_ROOT=/absolute/read-only/source_repo node scripts/mode-query-option-oracle.mjs
cargo test --locked -j2 -p xterm-core --test mode_query_option
cargo test --locked -j2 -p xterm-core --test terminal_modes --test terminal_mode_callbacks --test terminal_modes_differential
```

Use a private O: worktree and target with an explicit compiler grant. The source runner needs the reference's installed esbuild and Mocha. It builds into its own worktree-root tmp directory and removes the bundle in finally. The default command checks fixture equality; `--generate` explicitly updates the source fixture. It verifies all 776 reference byte lengths and SHA-256 records both before and after execution.

The runner selects all eight unchanged original InputHandler DECRQM tests. These retain the source's assertions and cover ordinary ANSI status, private set/reset, quirks and permanent-mode statuses. They are counted separately from the added differential corpus, not reported as eight new Rust tests.

The unchanged DECRQM cases map to the existing Rust assertions in `crates/xterm-core/tests/terminal_modes.rs`:

| Source test | Rust test |
| --- | --- |
| ANSI 2 | source_ansi_keyboard_action_permanently_reset |
| ANSI 4 | source_ansi_insert_mode |
| ANSI 12 | source_ansi_send_receive_permanently_set |
| ANSI 20 | source_ansi_newline_mode |
| ANSI unknown | source_ansi_unknown_mode |
| DEC privates with set/reset semantic | source_dec_private_set_reset_semantics |
| DEC privates quirks | source_dec_cursor_blink_quirk_gate |
| DEC privates perma modes | source_dec_permanent_modes |

The added corpus has 432 cases, replayed by one Rust test function:

- ANSI 20 and DEC 12, each followed by ignored additional main parameters and then an independent mode query and printable suffix.
- Both active buffers.
- Nine raw values: undefined, null, false, true, zero, NaN, empty string, nonempty string `false`, and an object.
- Twelve actions: inspection, reset, alternate/normal activation, resize, cursor mutation, mode mutation, disabled stdin, getter error, mutate-then-error, reply-observer reset, reply-observer error.

Comparisons include all cells and packed attributes, UTF-16 strings, both buffers, cursor/scrollbase/margins, projected modes, callback-time snapshots, getter counts and ordered replies/events. Error cases require identity with the original thrown error, not merely any failure. Successful replies execute after the getter and before the following parsed operation. Native callbacks use weak terminal references to avoid ownership cycles.

`results/mode-query-option/source.json` records the source run and exact fixture hash. Native quality, old-production controls and independent review receipts are added after actual execution. No browser runtime, native Linux/macOS, CI, release or performance result is implied by these source-derived cases.
