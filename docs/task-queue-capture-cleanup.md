# Task queue capture cleanup

Task355 fixes a native ownership failure in `TaskQueue::clear`. The method cleared its vector while holding the task-list `RefCell` borrow. Destroying a task can destroy captured Rust resources, whose `Drop` implementations may read, clear or enqueue on the same queue. Those operations panicked with `BorrowError` or `BorrowMutError` even though task callbacks themselves ran outside borrows.

Clear still cancels the outstanding host callback and resets the task index first. It now takes the task vector, releases its borrow, and then drops the removed tasks. Capture cleanup sees an empty queue. Work enqueued during cleanup remains queued and keeps its newly scheduled callback. A cleanup panic propagates normally rather than being replaced by a borrow panic. Completed flush and scheduled dispatch use the same clear path.

This discards the removed vector's capacity instead of retaining it across clear. It avoids a borrow across arbitrary capture destruction without cloning tasks or allocating a second task list. No dependency, unsafe code, retry, panic suppression or second queue implementation is introduced. The change does not specify destruction ordering between different tasks.

## Native ownership versus JavaScript behavior

The source contract is `source_repo/src/common/TaskQueue.ts`. Its `clear` cancels the callback, resets `_i` and clears `_tasks`. JavaScript array clearing does not synchronously run Rust capture destructors. The new tests exercise the port's explicit resource-cleanup boundary, not an invented JavaScript destructor or an original upstream suite. Existing source-derived queue scheduling, callback-reentry, failure and deadline tests remain unchanged.

The existing `scripts/task-platform-oracle.mjs` also passes. It compares4519 results with seed `0x715cad`, including1500 randomized queue iterations,2502 platform cases and500 Windows wrapping cases. It runs the three unchanged `WindowsMode.test.ts` tests through actual esbuild and Mocha, and separately records three supplemental fixture assertions. The repository has no original TaskQueue or Platform suite discovered by that runner. These are not4519 original tests and do not establish browser scheduling parity.

## Verification

Local checks used Windows, Rust/Cargo1.85.1 and Node24.15.0, initially based on main40f1798 with this task's changes. After rebasing onto main4a38b52, all28 focused tests, native core/web Clippy, WASM library Clippy and full formatting passed again. Raw logs and source/implementation hashes are in `results/task-queue-drop/`.

- Before the fix, the five new native tests produced one pass and four failures from task-list borrow panics. The retained baseline log contains the exact failures.
- After the fix,28 focused tests passed. This comprises five new capture-cleanup tests,15 existing queue tests and eight platform/wrapping tests, with no ignored tests.
- Warning-denied core/web native all-target/all-feature Clippy passed. Warning-denied xterm-web WASM library Clippy passed, including its core dependency.
- Exact full-workspace formatting passed before and after the fix.
- The existing source/Rust task-platform oracle passed with zero mismatches. The runner's exact commands, source hashes and original-suite counts are retained under `results/task-queue-drop/differential/`.

Run from an isolated worktree, with source_repo's existing locked Node dependencies available for the oracle:

```sh
cargo test --locked -j2 -p xterm-core --test task_queue --test task_queue_edges --test task_queue_drop --test platform_windows
cargo clippy --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
CARGO_BUILD_JOBS=2 node scripts/task-platform-oracle.mjs
cargo fmt --all -- --check
```

The five new tests cover direct clear with reentrant enqueue, recursive clear, completed flush and dispatch with a processed prefix, cleanup panic and queue reuse, and enqueue followed by cleanup panic. They assert task counts, callback cancellation/dispatch and exact panic payloads. Captured test resources use weak queue handles rather than strong reference cycles.

## Independent review

The isolated task355 reviewer accepted HEAD5d2be77 against main4a38b52. The reviewer independently reran all28 focused tests, full-workspace formatting, warning-denied core/web all-target/all-feature Clippy and WASM library Clippy. It inspected the reentry, panic and weak-handle paths and verified that retained oracle hashes match the reviewed files. It did not rerun the JavaScript oracle or browser suites. No concrete problems were found and no source files changed during review.

This task does not complete the common/lifecycle family, the raw mouse facade, the original browser suites, full workspace/release/feature gates, or native Linux/macOS execution. Overall equivalence remains NOT PROVEN.
