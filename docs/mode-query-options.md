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

## Observed producer verification

The producer was tested on native Windows x86_64 with Rust/Cargo 1.85.1 and Node 24.15.0. Its base is main1106, with only this task's production changes. This is not an integrated-main or browser result.

| Check | Observed result |
| --- | --- |
| Unchanged source DECRQM baseline | 8 passed, 0 failed/pending/skipped/unrun within the selected suite |
| Added source corpus, generation and replay | 432 passed each run, all 776 reference hashes verified before and after |
| Native new regression | One test function replayed all 432 cases successfully |
| Real old-production control | Both original main1106 mode files produced 432 mismatching cases and exit 101 |
| Exact restoration and rerun | Both fixed byte buffers/SHA-256 restored in finally, then 432 passed |
| Native workspace all-target/all-feature tests | 2,189 passed across 498 summaries, 0 failed/ignored/measured/filtered |
| Workspace all-feature doctests | 2 passed across 5 summaries, 0 failed/ignored/measured/filtered |
| Native workspace all-target/all-feature Clippy | Passed with warnings denied |
| WASM workspace all-feature library check and Clippy | Both passed, Clippy warnings denied |
| Exact cargo fmt --all -- --check | Failed with Windows OS206 argument length |
| Five complete member formatting checks | Each passed |

`results/mode-query-option/quality.json` retains commands, toolchain, times, headroom, code hashes and the two-file control/restoration metadata. Corresponding logs retain every test result and the aggregate fmt failure. Redundant EOF newlines in five logs are preserved reversibly in `raw-log-format.json`, with the exact removed byte suffix and original/retained SHA-256. No result text was changed. `source.json` records the source run and fixture hash. `corpus-summary.json` separately counts 828 getter snapshots, 612 reply snapshots and 108 original-error cases. `source-pins.json` and `native-binary-pins.json` retain 1,401 source inputs and five live mode-suite EXEs for independent inspection.

Workspace tests refreshed three pre-existing generated reports. Their actual deltas and old/new hashes are preserved under `generated-reports*`; all three historical files were restored byte-for-byte rather than replacing prior failures. Retention bookkeeping initially hit a 16 MiB child-output limit on the 18 MB HTML report and an incorrect expectation of six binary variants. Both partial effects were inspected before proceeding. Metadata in `generated-reports.json` and `counts.json` records those corrections. Neither changed application code, test assertions, or results.

Independent review and cleanup follow these receipts. EXE pins become historical metadata after owned target/worktree removal, not a promise that the files remain available. No browser runtime, native Linux/macOS, CI, release, Miri or performance result is implied. No unsafe code was added.
