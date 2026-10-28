# InputHandler print and control verification

Task137 migrates24 unchanged original cases and81 assertions from `source_repo/src/common/InputHandler.test.ts`. It does not claim that the rest of InputHandler or the complete terminal has been migrated. Overall equivalence remains NOT PROVEN.

## Original assertion mapping

All Rust files below are under `crates/xterm-core/tests/`. The common setup is `support/input_handler_core.rs`. It creates the original80x30 blank state, applies the source test dimensions and scrollback, and preserves literal spaces and UTF-16 expectations. Public stream writes exercise the actual Rust parser and engine, rather than calling JavaScript or mocking Rust buffer mutations.

| Original family/case | Rust file/test |
| --- | --- |
| print, zero-width infinite-loop regression | input_handler_core_print.rs, print_zero_width_regression_does_not_loop_or_add_rows |
| print, combining in one call | print_joins_combining_characters_in_one_call |
| print, combining across calls | print_joins_combining_characters_across_calls |
| print, REP grapheme | rep_repeats_the_preceding_grapheme_cluster |
| print, REP without state | rep_without_preceding_join_state_does_not_print |
| print, intervening escape resets REP | escape_sequence_resets_preceding_rep_state |
| print, early wide wrap clears right cells | early_wrap_clears_cells_to_the_right |
| print, strip U+00AD | soft_hyphens_are_stripped |
| ISO-2022 G0 | iso2022_g0_line_drawing |
| ISO-2022 G1/SO | iso2022_g1_line_drawing_after_shift_out |
| ISO-2022 saved charset/glevel identity | iso2022_saved_cursor_restores_charset_and_glevel_identity |
| wide cell reset by print | wide_cell_reset_by_print |
| wide cell reset by EL | wide_cell_reset_by_el |
| wide cell reset by ICH | wide_cell_reset_by_ich |
| wide cell reset by DCH | wide_cell_reset_by_dch |
| wide cell reset by ECH | wide_cell_reset_by_ech |
| earlier reverse-wrap margin regression | input_handler_core_backspace.rs, reverse_wrap_never_moves_outside_source_scroll_margins |
| default BS cannot delete last cell | default_backspace_cannot_delete_last_cell |
| default BS cannot reach previous line | default_backspace_cannot_access_previous_line |
| DEC45 deletes last cell | reverse_wrap_can_delete_last_cell |
| DEC45 reaches wrapped previous line | reverse_wrap_can_access_a_wrapped_previous_line |
| DEC45 lifts isWrapped | reverse_wrap_lifts_the_wrapped_flag |
| DEC45 stops at hard newline | reverse_wrap_stops_at_hard_newlines |
| DEC45 wide characters/early wrap padding | reverse_wrap_handles_wide_characters_and_early_wrap_padding |

The original selection contains8print,3charset,5wide-edit and8backspace cases. `scripts/input-handler-core-oracle.mjs` bundles the unchanged original test module through its installed esbuild dependency, runs the exact24 cases through Mocha and counts81 original Chai assertions. It does not replace source bodies or weaken them.

## Shared deterministic traces

The same oracle runs the real source headless terminal on54 directed traces and221 snapshots. It records both buffers, every packed cell/UTF-16 string/attribute, cursor/base/display/margins, modes, nominal InputHandler event order and accessibility callback-time state. Inputs include standalone zero-width marks, combining and surrogate chunk boundaries, soft hyphens, wide early wrap, insert/nowrap, C0 controls, charset shifts and40 reverse-wrap traces under DEC45/1045/enabled/disabled/margins/wrapped/unwrapped conditions.

`input_handler_core_differential.rs` compares all219 nominal snapshots and2accessibility snapshots in separate focused tests. Every row is compared and mismatch details are retained; there are no ignored tests. WriteBuffer completion/lifecycle, other handler families, browser behavior and full randomized terminal coverage remain separate requirements.

The first native run preserved an observer-harness bug: installing an event callback consumed nominal events before the test tried to drain them. The corrected observer collects those callbacks independently alongside a11y state. No production assertion was weakened and no terminal implementation change was needed. That original failure and an unrelated mid-edit SortedList compilation failure remain in `native-attempt-1.log` and `native-attempt-2.log`.

## Exact User case

`node scripts/acceptance-oracle.mjs --case vt-22` dispatches `scripts/acceptance-adapters/terminal/print-control.cases.mjs`. It reruns the original baseline and eight Rust backspace cases, builds the actual Rust recorder, pins a regular executable inside the immutable case attempt, and compares200 independent snapshots from40 reverse-wrap traces. The projection retains packed cells, malformed UTF-16, both buffers, cursor, margins, exact replies and events. Counterexamples and paired states are saved before a mismatch fails.

Observed native Windows receipt `results/acceptance-runs/vt-22/1793172520838-0679522c-1de1-4da8-8b94-a5c48c319ea3/` passed236 assertions,1runtime,0skips. This consists of36 original BS assertions plus200 strict paired snapshot comparisons. A prior missing-PATH attempt is retained separately and was not counted as a pass.

## Commands and current status

```sh
node scripts/input-handler-core-oracle.mjs
node --test scripts/input-handler-core-projection.test.mjs
cargo test -p xterm-core --test input_handler_core_print --test input_handler_core_backspace --test input_handler_core_differential --no-fail-fast
node scripts/acceptance-oracle.mjs --case vt-22
```

Use the reviewed build-slot wrapper for scoped Cargo commands; the acceptance parent already owns its in-process lease. Only `--generate` explicitly rewrites reference fixtures. Normal oracle invocation rejects drift and rehashes original source inputs.

`results/input-handler-core/oracle.json` records24/24 original cases,81assertions,54traces and221snapshots. `native-attempt-3.log` records26/26 Rust tests passed. Projection self-tests and owned-file rustfmt checks pass. The first scoped Clippy attempt failed on two unrelated unused Buffer cleanup members, not a passing global lint gate. Its immutable receipt remains linked from `clippy-attempt-1.log`; a later successful current lint run is still required under final quality task47. Native Linux/macOS and browser parity are not inferred from these Windows component checks.
