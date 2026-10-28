# Buffer reflow helpers

`crates/xterm-core/src/buffer/reflow.rs` ports every exported helper in `source_repo/src/common/buffer/BufferReflow.ts`. Rust names use snake_case. Wrapped rows are shared `Rc<RefCell<BufferLine>>` references, so compaction and layout application preserve the row identities observed by the containing buffer.

The five helper contracts are:

- `get_wrapped_line_trimmed_length` uses actual content length for the final wrapped row. Intermediate rows retain their entire old width unless a null final cell precedes a wide first cell on the next row.
- `reflow_larger_get_lines_to_remove` merges wrapped groups in place, skips the entire cursor group unless configured otherwise, repairs wide cells crossing the new edge, clears fragments, and returns removal ranges as `(start, count)` pairs.
- `reflow_larger_create_new_layout` returns `NewLayoutResult { layout, count_removed }`. It emits deletion events at indices adjusted for previous deletions before changing list order. Event callbacks can inspect or mutate storage without a held storage borrow. An emitted event is not rolled back after a callback error.
- `reflow_larger_apply_new_layout` snapshots references before rearranging them and updates logical list length afterward.
- `reflow_smaller_get_new_line_lengths` calculates exact wrapping points without splitting wide cells. It does not copy cells; the containing buffer performs the backward copy and viewport adjustment.

Widening callers resize physical rows before invoking the helper, matching source Buffer. The faithful upstream tests also cover direct calls without that preparation, preserving the source's bounded cell-copy behavior.

`ReflowError` reports invalid columns/ranges, missing rows, caller-held incompatible row borrows, storage failures and listener dispatch failures. No error is suppressed. One-column ASCII shrinking is supported. A wide character at a one-column wrap returns `InvalidColumns` rather than entering the source helper's non-progressing loop. Public terminal dimensions have a separate minimum of two columns. Invalid removal ranges are rejected before events; invalid layout indices are rejected before rearrangement. Partial cell movement before a later error is not transactional.

## Evidence

Run `node scripts/buffer-reflow-oracle.mjs` with Node 24, Rust/Cargo, and the reference's installed development dependencies. It uses upstream esbuild to bundle the actual read-only implementation, then runs the unchanged original Mocha suites via `scripts/buffer-baseline.mjs`. No runtime engine delegates to JavaScript.

The Rust tests are:

- `buffer_reflow_upstream.rs`, all seven source helper test cases and their assertions.
- `buffer_reflow_edges.rs`, event timing, error propagation, reentrancy, row and extended-attribute identity, empty/unwrapped data, width-one behavior and explicit invalid-input boundaries.
- `buffer_reflow_differential.rs`, 3,000 deterministic cases using seed `0xb0ffef10`. Half widen, half shrink. Widening compares full raw cells, UTF-16 strings including lone surrogates, widths, foreground/background, extended attributes/URL, trim/wrap metadata, removals/layouts, callback-time list lengths and surviving row identities. Shrinking compares exact destination lengths.

The generated fixture `tests/fixtures/buffer-reflow.tsv` intentionally exceeds the source-file line limit. It retains every deterministic oracle input and expected result for reproducible diagnostics. The fixture is data, not handwritten implementation. Raw results and source hashes are in `results/buffer-reflow/`; always read the recorded status rather than assuming concurrent workspace builds passed.

Full Buffer resize/viewport and memory-cleanup behavior belongs to task 27, the complete Buffer suite migration to task 67, and real parser/addon integration to task 99. Helper comparisons do not replace those checks. Overall terminal equivalence remains NOT PROVEN. This helper run alone provides no browser or native Linux/macOS evidence.
