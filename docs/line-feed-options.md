# Line-feed option getter and scroll dirty ranges

Task 484 continues the same swarm's full Rust port. This repair does not close full parser task 313, full quality task 304 or overall equivalence.

## Source behavior and repair

`InputHandler.lineFeed` marks the current row dirty before reading `rawOptions.convertEol`. A getter may reset, resize, switch buffers, change the cursor, call another line feed or throw. Subsequent mutations use the live buffer. Rust held both terminal state and the old buffer borrowed during that getter, causing a borrow error instead of executing the callback.

`terminal/controls.rs` now marks the old row, captures the options service and releases both borrows before reading `convertEol`. It resolves the active buffer again after that read. Getter errors retain their original callback identity and committed partial state. This applies to LF, VT, FF and the existing direct line-feed host entry point.

The corpus also exposed an omitted source `CoreTerminal` hook. `BufferService.onScroll` fires public observers and then marks the live scrolling margins dirty. Rust instead marked those rows in selected callers, missing direct buffer activation from a getter and marking too late relative to later service observers. `terminal/construction.rs` now performs that shared hook after public forwarding, without holding terminal state across observers. The three duplicate marks in parser scrolling, public scrolling and mode-driven buffer activation are removed. The hook uses a weak state handle and the terminal's existing disposable subscription list.

No new dependency, public API or unsafe code was added. Task 475's accepted BS/RI changes are separate and must remain intact when integrating the independent `controls.rs` hunks.

## Reproduction

```sh
# XTERM_SOURCE_ROOT is optional when source_repo exists in this checkout.
XTERM_SOURCE_ROOT=/path/to/untouched/source_repo node scripts/line-feed-option-oracle.mjs
cargo test --locked -p xterm-core --test line_feed_option -j2
```

The source script bundles the unchanged headless terminal using its installed esbuild. It verifies the entire 776-file reference manifest before and after execution and removes its owned temporary bundle in `finally`. `--generate` explicitly updates the retained fixture. Normal execution rejects fixture drift.

The 144 cases cover three control bytes, both initial buffers, both getter return values and 12 callback actions. They compare both buffers, all cells and attributes, cursor, dimensions, margins, terminal modes, callback checkpoints, getter counts, exact error identity and ordered line-feed/scroll/resize/cursor/render events. Source callbacks include reset, direct buffer activation, resize, cursor mutation, nested line feed, throwing before or after mutation, public scroll observers that mutate or throw, and later service observers that change margins. The last cases distinguish the correct scroll dirty-mark phase from a late mark after every observer.

These are deterministic source-derived cases, not randomized coverage or a complete original InputHandler suite. Rust executes its own terminal and raw-option store adapter, not the JavaScript engine. The source baseline is not counted as a Rust pass.

## Evidence

Base main is `26c7c42d`. Raw logs and receipts are in `results/line-feed-option/`.

- Source generation and non-generating replay each pass 144 cases. Both preserve all 776 reference hashes.
- The native fixture passes all 144 cases in one test function.
- Real old-production negatives replace the four production files from the base revision, then fail 144 cases. Replacing only the old getter file fails 144. Replacing only the old construction hook fails 36 cases and passes 108. Each replacement is restored in `finally` and byte-hash checked, followed by a passing fixed run. `negative-summary.json` retains exact files and restored hashes.
- All-target/all-feature core tests pass 1,565 test functions across 246 result summaries, with zero failed or ignored tests. Both core doctests pass. Strict all-target/all-feature core Clippy and core rustfmt pass.
- Native and WASM workspace all-feature library checks pass. These are checks, not full workspace tests or Clippy.
- Full `cargo fmt --all -- --check` fails with Windows OS error 206 at this path. Its raw output is retained and is not a pass.
- The first test compile had two test-only type/lifetime mistakes, now corrected. Early source event capture incorrectly destructured a numeric scroll event, now corrected before final fixture generation and negatives. Early failures remain separate from final evidence.
- Whole-core tests regenerate existing buffer/serialize result files. Their unrelated changes were inspected and restored to the branch baseline. The regenerated buffer summary records 73 passing traces and 862 snapshots. An initial Node-based restore hit `ENOBUFS` on the large existing HTML report before changing that file; the subsequent explicit restore of that owned generated file completed. No unrelated source or reference was changed.

No browser runtime, Linux/macOS runtime, CI, release/performance/size comparison or complete original-suite matrix ran for this task. Browser getter integration remains part of the open full-family/browser tasks. Overall equivalence remains NOT PROVEN.
