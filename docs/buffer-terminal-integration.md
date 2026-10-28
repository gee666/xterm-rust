# Live buffer, parser and addon integration

Tasks 99 and 160 exercise actual terminal operations, not hydrated Buffer inputs. Their implementation is in progress. No Rust integration pass is claimed here, and overall equivalence remains NOT PROVEN.

## Shared observation contract

`scripts/buffer-terminal-oracle.mjs` bundles the original headless public Terminal, SerializeAddon, SearchEngine and SearchLineCache using the reference's existing esbuild. It feeds actual writes through the parser, awaits source write callbacks and observes every operation boundary. Rust uses the actual public native Terminal with the same dimensions, writes and operations. SerializeAddon loads through the real addon manager. Search borrows the real ring through `search::with_buffer`, including physical stale slots and negative-index aliases; it does not create a text-only substitute.

Each fixture record includes:

- Both normal and alternate buffers, every packed cell and UTF-16 string, widths, attributes, wrapped flags, cursor, display/base, scroll regions and modes. This reuses the shared terminal snapshot format.
- Saved X/Y for both buffers, including signed saved Y.
- Marker creation results, live/disposed lines, public marker identity/order and exact disposal callbacks.
- Ordered bell/cursor/linefeed/scroll/reply/title/render events.
- ANSI output from an activated SerializeAddon after every operation.
- Five SearchEngine outcomes per observation, including actual exceptional outcomes rather than converting exceptions to no-match.
- At each trace's end, serialization fed through a fresh actual parser at the same dimensions. The source restored state and event sequence are the oracle. Source serialization is not assumed lossless.
- Clear retains the actual previous cursor-row object as the new first row. Reference identity is checked separately from cell equality.

## PA06 cases

The drafted corpus includes the original five-row clear and marker-disposal assertions after ten separately awaited writes of LF CR test0 through test9. Four out-of-viewport markers are disposed, the current test9 row becomes line zero, and the four remaining rows are blank. Additional source traces cover styled prompts, wide and combining cells, wrapped prompts, a cursor above the final row, a scrolled-up viewport, normal/alternate marker isolation and reset.

Scroll operations include signed line/page counts, top/bottom, negative absolute targets, middle positions, values beyond both ends, no-change calls, empty scrollback, alternate screen and after clear/reset. Marker offsets include negative and beyond-buffer positions. The public native `scroll_to_line_signed(i64)` preserves these accepted signed source targets without breaking the existing unsigned convenience method. Both routes use the actual terminal scroll service; negative values are not silently rejected or cast to unsigned. JavaScript numeric ABI validation and values outside i64 remain separate binding criteria.

## Current generated evidence versus drafts

Before the coordinated disk pause, the first source-only generator run captured 69 traces, 801 snapshots, 95 marker creations, 4,005 search queries and 69 source restore comparisons. That initial 16-field schema was then extended as described below. The current generated fixture is `crates/xterm-core/tests/fixtures/buffer-terminal.tsv`, with source hashes and individual search failures in `results/buffer-terminal/oracle.json`.

That run exposed 80 actual source RangeError exceptions. Details and an exact reproducible operation trace are in `results/buffer-terminal/search-ring-regression.md`. Task 36's owner is preserving them as typed ring-recursion errors with compatible browser RangeError behavior. They must not disappear as skipped queries or successful None results.

During the disk pause, Kassa permitted a tiny source-only regeneration under the 1 GiB resource guard. The extended fixture now has 73 traces, 862 snapshots, 107 marker creations, 4,310 search queries and 73 restore comparisons, with 19 fields per record. It includes the PA06 families above, default marker registration, the addMarker alias, absolute scroll targets at plus/minus 2^53, public marker identities, clear row identity and restore events. All source assertions passed; the same 80 exceptional search outcomes remain visible.

Rust tests are now exported at `crates/xterm-core/tests/buffer_terminal_differential.rs` and `tests/support/buffer_terminal.rs`. They use the shared snapshot_state helper and live-ring search helper. They are formatted but UNRUN pending a coordinated build slot. No source-only capture is counted as Rust execution.

After resource coordination permits builds, the next steps are:

1. Run the normal source oracle without --generate to reject fixture drift.
2. Execute the exported native test target with bounded, resource-monitored commands through the shared build queue.
3. Retain every counterexample, exact counts, source/test hashes, tool versions, command output and code changes. Fix real engine/addon mismatches rather than trimming the corpus.
4. Request independent review only after the current tests execute. A source-only capture is not Rust evidence.

## Observed native execution

The first coordinated native run executed both test functions. The five public marker-disposal callback cases passed. Across the 73 main traces, all full cell/buffer/mode snapshots, saved cursors, marker identities/disposal, ordered events, ANSI output and 73 parser-restored states matched. Eight search observations failed because the live-ring view rejected old-width alternate-buffer rows after resize, while the source permits them. All 80 exceptional source RangeError outcomes already matched the new native typed errors. The overall test target correctly FAILED, with one Rust test passing and one failing.

All eight counterexamples and the process receipt are retained under `results/buffer-terminal/regressions/ring-width-1793149702432/`. This exploratory run did not capture an immutable Rust workspace hash, which is recorded as a limitation. The search owner removed the extra live-ring width restriction and ran focused search tests, but this full integration rerun has not happened yet. The next build request was refused before starting by the 1 GiB disk guard; its receipt is `results/build-slots/1793150286515-47268-44a68f30-a1f7-486a-a4cf-051fa3e6e1c3/receipt.json`.

The new reusable checker captures Rust/Cargo versions, current source and callback oracles, native tests, scoped warning-denying Clippy, workspace formatting, input hashes and whole-workspace drift. Run it through the build queue:

```text
node scripts/build-slot.mjs --owner Buffer-integration --lane native -- node scripts/check-buffer-terminal.mjs
```

It initializes all current artifact statuses before execution, retains per-attempt logs and does not reuse an older passing result after compilation or setup failure. It labels semantic observations separately from overall quality/source stability. That checker is currently UNRUN because the latest resource gate refused its child command.

## Host cleanup and platform gaps

The integration audit found no production call to BufferCleanup::new outside tests. A generic adapter with only component polls is not actual native/browser host integration. Criterion 413 and tasks 179/180 explicitly track real normal/alternate cleanup owners, reset reattachment, idle/priority execution, cancellation/drop and original host error identity. Task 99 verifies their cooperation with actual terminal operations. Existing component cleanup tests and base Buffer acceptance do not close this gap.

No native Linux/macOS, actual browser runtime or full addon DOM lifecycle result is supplied by these Windows/source checks. SearchAddon selection/decorations remain task 59. Native/browser host scheduling and platform jobs remain separately required. Disk pressure is a coordinated execution pause, not permission to mark unrun tests passed or delete source, fixtures or prior evidence.
