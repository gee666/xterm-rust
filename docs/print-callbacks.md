# Print callback boundaries

Task 153 ports `source_repo/src/common/InputHandler.ts` print and REP behavior through the real Rust parser. It owns `terminal/print.rs`, `print_render.rs`, the print registration closure, and REP preparation/registration. Full terminal equivalence remains NOT PROVEN.

## Required cases

- PR01: Snapshot the selected charset object once per print call. A callback changing G0 or the invoked level must not alter the remainder of that call. A later chunk takes a new snapshot. Shared source map identity is retained, not rebuilt per character.
- PR02: Snapshot columns, wraparound, insert mode and screen-reader enablement at the same source statements. Rows and active-buffer selection remain live where source accesses them later.
- PR03: Invoke Unicode property providers without a terminal, buffer, row or attribute borrow held. Provider changes of active Unicode version affect later property calls through the same live service.
- PR04: Deliver each a11y character before writing its cell or incrementing its cursor. Release terminal/event queue borrows before callbacks. Record callback-time cursor, cells and charset state, not just final output.
- PR05: Retain the source attribute-object target across callbacks and reset. Mutations of that same object remain visible. Replacing the current object does not retarget the old print call. Extended attributes share payload and URL identity; ordinary ExtendedAttrs cloning must not strip payload during printing.
- PR06: Read the live current hyperlink ID after the a11y callback and before wrap, separately from the captured cell-attribute target. Marker registration, scroll and user callbacks run without a retained terminal-state borrow.
- PR07: Keep row references where source does. On autowrap, release borrows before scrolling, then reacquire the source-selected current row. Preserve joined-cell copying and wide-cell cleanup. Missing rows after resize take the source's defensive return, not a made-up cell or panic.
- PR08: Commit preceding join state only at the source completion point. An exception preserves partial cells, cursor and pending join state. Stop later callbacks and propagate the exact concrete callback cause.
- PR09: REP selects and expands the prior grapheme under a short borrow, then uses the same callback-safe print path. A11y, charset snapshots, attribute targets and exceptions cannot revert to a whole-command mutable state borrow.
- PR10: Soft hyphens produce neither cells nor a11y events. Empty calls retain source dirty/wide-cleanup behavior. Payloads stay UTF-16, including lone-surrogate charset replacements, through the existing `AccessibilityChar(Vec<u16>)` path.

## Verification

`tests/charset_terminal.rs::callbacks_observe_pre_write_cells_and_live_captured_map_identity` consumes the paired source callback fixture from task 82. The existing 402 charset engine snapshots and CSI/REP suite must continue passing. Add directed provider-reentry, callback-error, attribute-target/reset and mode-snapshot cases. Run rustfmt, scoped warning-free Clippy and real terminal regressions after the disk build pause lifts. Keep each command and its current source hashes in results/print-callbacks/.

The five private provider/attribute/error/REP tests passed. Timur independently checked the source and passed 18 integration tests, including 402 ordered-state profiles and six callback profiles. Task 153 is accepted. This is scoped native proof, not complete terminal or browser equivalence.
