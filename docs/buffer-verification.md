# Buffer verification

Task 67 verifies `source_repo/src/common/buffer/Buffer.test.ts` against the Rust `buffer::Buffer`. It does not implement or delegate any runtime terminal behavior to JavaScript. Overall terminal equivalence remains NOT PROVEN.

## Original assertions and provenance

`node scripts/buffer-baseline.mjs` bundles the unchanged original test entry points and their actual dependencies using the reference's installed esbuild. Actual Mocha and Chai execute every original test body. The original files and assertions are not rewritten. Temporary bundles live under `tmp/buffer-baseline`.

The observed Windows baseline has 61 Buffer cases, 6 BufferSet cases and 7 BufferReflow cases, all passing without skips. `results/buffer/original.json` retains individual names/outcomes and SHA-256 hashes of every bundled source input. The original idle-memory test's upstream timeout remains intact in this baseline. Rust cleanup tests drive the scheduler directly rather than copying that sleep.

BufferSet assertion migration belongs to task 79. BufferReflow assertion migration belongs to task 62. Their successful JavaScript baseline is not Rust evidence.

## Source-case migration

The source Buffer suite's 61 cases map as follows. The machine-readable per-case map is generated in each `scripts/check-buffer.mjs` report and fails if any case is unmapped.

| Source cases | Rust evidence |
| --- | --- |
| Constructor 2, viewport fill 1 | `tests/buffer_screen_upstream.rs`, 3 direct tests |
| Wrapped-range scans 10 | Same file, 10 separately named generated tests |
| Resize matrix and alternate scrollback 39 | All associated resize input/output pairs in `tests/buffer_resize_differential.rs` |
| Marker trim/disposal 3 | `tests/buffer_screen_upstream.rs`, 3 direct tests |
| Translation slices, wide placeholders and emoji 5 | Same file, 5 direct tests |
| Idle memory cleanup 1 | Exact backing-byte assertions in `src/buffer/line_buffer_cleanup_tests.rs`; public cleanup observations in `tests/buffer_screen_upstream.rs`; resize transition also appears in the fixture |

There are 22 direct integration tests, including a second public observation of the cleanup case. Source cases and Rust test functions are different counts. The resize fixture adds stronger state checks to the original resize expectations rather than replacing them with a text-only smoke test.

## Resize observations

`node scripts/buffer-resize-oracle.mjs` instruments only the Buffer resize entry point in a temporary bundle. All original test assertions still run. The observer records the input immediately before each call and the result immediately afterward. The uninstrumented baseline runs separately to detect accidental changes in assertion outcomes.

The fixture contains 86 original resize calls plus 128 traces of 8 randomized resize calls and one directed regression, 1,111 transitions total. The directed case verifies that an empty first wrapped row can produce source `savedY=-1`; Rust preserves that signed value instead of returning an arithmetic-overflow error. The xorshift32 seed is `0xb0ff67`. Inputs include normal and alternate buffers, empty/content cells, Unicode and lone surrogates, combined strings, attributes, extended attributes and hyperlinks, changing host dimensions, cursor and saved positions, scrollback, display tracking, scroll regions, marker insertion, ConPTY/WinPTY and missing/zero/build-21,375/build-21,376 options.

Each input and output records:

- Internal dimensions separately from live host dimensions, capacity, line count, cursor, base/display, saved cursor and scroll region.
- Every cell's packed content, foreground/background, extended attributes, URL and combined UTF-16 storage.
- Full and right-trimmed UTF-16 translation, each line's wrapped flag, and surviving pre-resize line identities.
- Every original marker reference, including references disposed and removed from the active collection.
- Exact ordered insert/delete/trim events and marker disposal callbacks.

`tests/support/buffer_snapshot.rs` hydrates the recorded input into actual Rust Buffer/BufferLine/Marker objects. The test then calls Rust `resize` and compares the result to the independently executed JavaScript result. It also checks input reconstruction before the operation. Immutable has-scrollback configuration is supplied to the constructor; the other state fields are compared through public getters/fields. New rows receive identity zero; old rows must retain their individual recorded identities.

The checked-in TSV intentionally exceeds the 350-line source limit. It is generated input/result data, retained so Rust tests do not need JavaScript or network access during ordinary execution.

## Additional boundaries

`tests/buffer_screen_edges.rs` checks zero-size rejection without mutation, scratch-cell extended-attribute sharing/default reset, live host width, partial viewport fill, saved state and marker lifetime across clear, cursor viewport bounds, tab exclusion/clamping/incremental setup, invalid tab widths, a one-column buffer, missing-row translation, half-open deletion ranges and callback-driven marker disposal. A first draft expected callback-driven disposal to empty the marker list. Direct execution of the original source disproved that guess: disposing the second marker changes its line to -1 before the clear loop reaches it, so the original retains that disposed entry. The oracle now explicitly asserts this behavior, and Rust preserves it.

The exact memory test runs as a child of BufferLine's test module to observe private retained storage without adding a production diagnostic API. Vec storage length models the original ArrayBuffer's retained cell count; logical line length models its typed-array view. It checks exactly 80 * 3 * 4 bytes before cleanup and 39 * 3 * 4 afterward, preserving every original assertion. Allocator spare capacity is not claimed to match JavaScript engine allocation.

## Commands and retained evidence

With the tools in PATH:

```text
node scripts/check-buffer.mjs
cargo test -p xterm-core --test buffer_resize_differential --test buffer_screen_upstream --test buffer_screen_edges
cargo test -p xterm-core --lib original_buffer_idle_cleanup_preserves_then_realigns_exact_backing_bytes
```

The check script uses bounded process lifetimes, native-quality disk monitoring, the shared target directory and the repository's configured profiles. Each attempt records exact commands, output, pass/fail/ignored counts, test and implementation hashes, fixture hash and any buffer-source changes during execution under `results/buffer/runs`. A changed hash prevents that attempt from being reported as a stable pass. `results/buffer/latest.json` identifies the current attempt. Read its status and logs; the presence of a report is not evidence of success.

## Limits

Each transition is hydrated independently. This does not prove preceding parser writes, public callbacks, live options or asynchronous event-loop behavior. The random generator currently uses width-one cells, including Unicode and combined strings. Original assertions cover wide-cell boundaries, and task 62 adds a separate mixed-width helper corpus. New rows are checked as new identities but aliases between separately created new rows are not measured by the resize fixture.

Real scheduler batching/cancellation and failure propagation are task 27's contract tests. BufferSet/BufferService scrolling and option subscription checks are task 79. Actual parser/InputHandler, search/serialize, browser and native-platform integration remain task 99 and the project-wide verification tasks. This Windows component check supplies no native Linux/macOS or browser runtime evidence. Shared-tree compilation failures and resource interruptions remain failures, not external implementation blockers.
