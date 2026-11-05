# Alternate buffer cleanup preconditions

Task371 resolves the existing `wasm_buffer_cleanup::inactive_normal_and_active_alternate_are_both_scheduled` failure without changing production behavior or removing an assertion.

## Diagnosis

The source `src/common/buffer/Buffer.ts::resize` shrinks line columns only inside `_isReflowEnabled`. That getter returns `_hasScrollback` when no Windows PTY override applies. Alternate buffers do not have scrollback. A resize from12 to3 therefore leaves alternate rows with12 logical cells and schedules no cleanup. Rust's `screen_resize.rs` already implements this behavior correctly.

The existing browser test wrote `normal`, entered the alternate screen, wrote `alt`, shrank12 to3 and expected pending cleanup in both buffers. The alternate expectation had no valid precondition. Prior task367 retained the identical failure on unchanged main6b8f80c and its queue repair in all three engines. See `results/task-queue-clock/baseline-buffer-browser.json`. This is not a newly discovered production regression or a reason to make alternate rows shrink contrary to source.

The test now also grows3 to4 before its original pending-cleanup assertions. Source considers this column growth relative to the buffer's last terminal size. Resizing alternate rows from logical12 to4 leaves an oversized backing allocation and requests cleanup. Normal rows still have retained12-cell backing storage at this point, because both resizes happen synchronously before the host callback. Growing them from3 to4 also requests cleanup. The original test can now exercise two valid cleanup requests.

Every original line and all11 existing assertion macros in the browser file are unchanged. The patch adds16 lines, including assertions that the alternate buffer has no pending cleanup and still has12-cell rows after the initial shrink. The original assertions for pending cleanup in both buffers, eventual completion and at least two host requests remain intact. No production files, dependencies, unsafe code, upstream files or original upstream assertions changed.

## Source and native regression

`scripts/buffer-cleanup-oracle.mjs` bundles the unchanged source headless Terminal and its actual dependencies in memory with esbuild. It runs the real parser, BufferSet, Buffer and cleanup queues in a Node VM with controlled timer hosts. It does not replace the terminal engine or rewrite its behavior. Both idle and priority hosts, starting widths12 and80, and scrollback0 and10 produce8 traces. Each records both buffers' row lengths and cleanup-pending state before resizing, after shrinking, after growing and after draining the queue.

The oracle checks `crates/xterm-core/tests/fixtures/buffer-cleanup.tsv`. Only explicit `--write` updates expectations. The native `buffer_cleanup_resize` test function replays all8 source traces through the Rust Terminal and BufferService, including the source's final cleanup rescan. Native buffer state is host-independent; source idle/priority variants deliberately repeat that state comparison. Actual timer dispatch and cancellation are tested by the existing browser suite.

`results/buffer-cleanup/source.json` retains all bundled source hashes and the fixture hash. `verification.json` identifies source, test and script hashes, unchanged assertions, baseline evidence and exact scoped counts.

## Observed verification

Windows x64, Node24.15.0, Rust/Cargo1.85.1, base main2f1be3b:

- Unchanged source oracle passes8 traces.
- Focused native cleanup command passes18 Rust test functions, including the new function replaying8 traces.
- Existing real browser cleanup suite passes4/4 tests in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. No skipped or filtered cases.
- Full workspace formatting passes.
- Warning-denied native Clippy for the new regression target and WASM Clippy for the existing browser cleanup target pass.

Raw logs and the browser runner report are in `results/buffer-cleanup/`. Generated WASM/bindings and private compiler outputs are disposable, not committed. The report retains their hashes and original run paths, not a promise that those temporary paths still exist.

## Independent review and integration

The isolated task371 reviewer accepted commit `a683fc3` with no findings or tracked edits. It independently reran all8 source traces, all18 focused native tests, all4 browser tests in each engine, workspace formatting and both scoped Clippy checks. `review-browser-results.json` retains its final-source browser report.

The canonical helper integrated accepted source and review evidence into main `00dbec3`. It passed1873 native tests across355 result summaries for the changed core/web crates, affected-dependent all-target checks, workspace native checks and applicable WASM library checks. Those are default-feature canonical integration gates, not the final full-feature/platform matrix. `canonical.log` retains the gate output. The helper removed the private compiler target after successful integration.

## Reproduction

Run in an isolated checkout with Rust's WASM target, wasm-bindgen0.2.100 CLI and the existing source Node dependencies. Set `XT_SOURCE_ROOT` or `XT_NODE_MODULES` for an external reference/tool installation. Set `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` when they are not installed at the default runner paths. Keep temporary builds on the run-owned O: volume and set `CARGO_BUILD_JOBS=2`.

```sh
node scripts/buffer-cleanup-oracle.mjs
cargo test --locked -j2 -p xterm-core --test buffer_cleanup_resize --test buffer_cleanup_injection --test buffer_contracts
WASM_TEST=wasm_buffer_cleanup node scripts/browser-host-check.mjs
cargo fmt --all -- --check
cargo clippy --locked -j2 -p xterm-core --test buffer_cleanup_resize -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_buffer_cleanup -- -D warnings
```

This task establishes the cause of a specific invalid test precondition and verifies the corrected cleanup path. It does not establish full Buffer suite parity, the full native all-feature/platform matrix or browser/demo equivalence. Overall equivalence remains NOT PROVEN.
