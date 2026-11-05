# Task queue clock reentry

Task367 repairs the confirmed clock callback ordering mismatch in `TaskQueue::process`. The original counterexample from `swarm/A16` is retained unchanged in `results/task-queue-clock/source-probe.mjs`, `source.json`, `native-repro.rs`, `native.txt` and `verification.json`. Those files describe the earlier failure, not the repair's test results. Current evidence is `repair-verification.json` and the adjacent logs.

## Behavior and error representation

The unchanged source `src/common/TaskQueue.ts` checks `_i < _tasks.length`, calls `performance.now()`, then reads and invokes `_tasks[_i]`. Rust previously cloned the task before reading the clock. A reentrant clock that cleared the queue and enqueued a replacement caused source to execute `new` but Rust to execute `old` and discard `new`.

Rust now checks the loop bound, reads the clock, and only then clones the current task. It holds no task-list borrow across the clock. The current index is also read after the callback, so clearing at a nonzero index dispatches the replacement's current task. Clearing without replacement, or synchronously flushing the queue during that clock, returns an explicit missing-task error. It does not run a stale task, panic, or silently succeed.

Scheduled `TaskQueue::process`, `process_priority` and the corresponding `DebouncedIdleTask` methods now return `Result<(), TaskQueueError<S::Error>>`. `TaskQueueError::Callback(E)` retains the existing task, request or warning failure unchanged. `MissingTask` represents invoking an absent source function, even when `S::Error` is `Infallible`. Synchronous enqueue and flush results remain `S::Error`; their lookup phases did not change. `BufferCleanup`'s two process methods propagate the queue error around their existing cleanup error domain. Injected scheduler failures retain the same inner value. Tests now unwrap that additional typed layer without changing their original error, retry, deadline or identity assertions.

The production browser dispatcher maps `MissingTask` to a JavaScript `TypeError`. Callback failures return their original `JsValue` unchanged. Error class and object identity are tested; engine-specific source exception messages are not promised to match. No dependency, unsafe code, fallback queue or JavaScript engine delegation was added.

## Shared source traces

`scripts/task-queue-clock-oracle.mjs` erases TypeScript syntax and type-only imports from the unchanged source in memory. It executes the actual queue with controlled timer and clock hosts in Node and in Chromium, Firefox and WebKit. It does not alter the source or its original assertions. `--write` explicitly generates the checked-in TSV corpus; ordinary execution checks it instead of refreshing expectations.

The 32 traces cover idle and priority queues at task-start clocks, task-end clocks and the second task's start clock. Actions clear, flush, replace, or replace with a throwing task. Priority deadline construction and the initial deadline read are also covered, where clearing precedes the loop guard and must succeed normally. Each trace asserts exact clock/task/request/cancellation order, error kind, remaining work and pending callback state. Native Rust and the real browser/WASM adapter consume the same TSV through one shared test helper. TSV avoids a new test dependency in the portable core.

The new native and WASM suites each contain one Rust test function replaying all32 cases. They are not32 original upstream tests. The browser test also checks thrown callback object identity and retries queue use after every outcome. Its controlled host calls the retained callback through the real browser/WASM function boundary. The existing `wasm_host` suite separately exercises ordinary host timing.

## Observed verification

Windows x64, Rust/Cargo1.85.1 and Node24.15.0, initially based on main6b8f80c:

- Restoring only the old early lookup in an otherwise repaired build makes the new native test fail with `task:old` versus expected `task:new`. `baseline-native.log` retains the failure.
- The source oracle matches all32 cases in Node and each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4.
- The repaired native replay passes32/32 cases. Existing queue and buffer-consumer suites also pass,38 Rust test functions in the focused command.
- Full xterm-core all-target/all-feature tests pass1539 tests across236 result summaries; its two compile-fail doctests pass.
- Warning-denied core/web native all-target/all-feature Clippy and scoped web WASM library/new-test/existing-error-test Clippy pass. Full workspace formatting passes.
- Existing `wasm_task_queue_errors` passes5 tests per browser and `wasm_host` passes10 per browser. The new WASM replay passes its32 cases per browser.
- Existing source/Rust task-platform differential checks match4519 results with seed `0x715cad`, including1500 random queue iterations. The unchanged original WindowsMode Mocha suite passes3/3. This is not4519 original tests.

At this task367 checkpoint, an additional unchanged browser regression suite was red. `wasm_buffer_cleanup::inactive_normal_and_active_alternate_are_both_scheduled` fails line139 because the alternate buffer has no pending cleanup. Both unmodified main6b8f80c production and the repair pass3/4 tests per browser and fail this same assertion in all three engines. The baseline was rebuilt after temporarily restoring only the four owned production files, then the repair was reapplied byte-for-byte. Raw reports and logs are retained under `browser/` and `baseline-buffer-browser.json`. No test assertion was changed. Follow-up task371 diagnosed an invalid fixture precondition and preserved every original assertion while adding the source-correct shrink-then-grow sequence. Its suite subsequently passed4/4 in each engine. See `docs/buffer-cleanup-resize.md` and `results/buffer-cleanup/`. The earlier failure was not a scheduler parity pass.

## Independent review

The isolated task367 reviewer accepted8dabe05 against main e165104 with no findings or tracked edits. It independently reran all38 focused native tests, the source32-case oracle and the Rust/WASM32-case replay in all three browsers, full workspace formatting, native core/web all-target/all-feature Clippy and scoped WASM Clippy. All passed. It verified every source and fixture hash in `repair-verification.json` and inspected the identical retained baseline/repair alternate-buffer failure. That separate failure remains task371.

## Reproduction

Run in an isolated worktree with an installed WASM target, matching wasm-bindgen0.2.100 CLI and existing Playwright installation. Set `XT_SOURCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` when using an external read-only reference or tool installation. Set `CARGO_BUILD_JOBS=2` and keep temporary output on the run-owned volume.

```sh
node scripts/task-queue-clock-oracle.mjs --browsers
cargo test --locked -j2 -p xterm-core --test task_queue --test task_queue_edges --test task_queue_drop --test task_queue_clock --test buffer_contracts --test buffer_cleanup_injection
cargo test --locked -j2 -p xterm-core --all-targets --all-features
cargo test --locked -j2 -p xterm-core --doc
cargo clippy --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_task_queue_clock --test wasm_task_queue_errors -- -D warnings
WASM_TEST=wasm_task_queue_clock node scripts/browser-host-check.mjs
WASM_TEST=wasm_task_queue_errors node scripts/browser-host-check.mjs
WASM_TEST=wasm_host node scripts/browser-host-check.mjs
CARGO_BUILD_JOBS=2 node scripts/task-platform-oracle.mjs
cargo fmt --all -- --check
```

Generated WASM/binding binaries are disposable and are not committed. Their exact hashes, source hashes and command reports are retained. This task does not establish full common/lifecycle parity, native Linux/macOS runtime equivalence, or completion of the original browser/demo suites. Overall equivalence remains NOT PROVEN.
