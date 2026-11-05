# Circular list value cleanup

Task358 fixes native resource destruction under `CircularList`'s storage `RefCell`. Replacing or removing a slot, clearing physical slots during length expansion, or replacing storage during a capacity change could run a value's `Drop` implementation while storage remained borrowed. Cleanup that read or mutated the same list panicked with `BorrowError` or `BorrowMutError`.

Removed values now leave storage before they are destroyed. Slot assignment returns the displaced value. Length expansion detaches all affected physical slots and commits the new length before releasing them. Capacity changes install the copied live slots, capacity and reset start index before releasing the old storage. Reentrant cleanup therefore sees the committed mutation, and its subsequent writes survive. A cleanup panic propagates with storage reusable, rather than being replaced by a borrow panic.

Assignment keeps a new value with the caller until slot reservation succeeds. A recoverable reservation failure therefore also releases the rejected value outside the storage borrow. This failure path follows ownership inspection; the tests do not simulate allocator exhaustion.

Length expansion uses scratch vectors proportional to removed materialized slots, not the numeric length range. This replaces allocation-free `retain` because removed values must remain alive until the complete mutation is published and the borrow ends. Capacity changes reuse the old storage itself as the returned cleanup owner, without another copy. No dependency, unsafe code, suppression, timer, retry or parallel list implementation was added.

## Source behavior and scope

The reference is `source_repo/src/common/CircularList.ts`. JavaScript slot assignment and array replacement do not synchronously run Rust destructors. These tests define the port's native resource-cleanup boundary, not invented original JavaScript destructor assertions. Existing source event ordering, wrapped indexing, physical-slot clearing, sparse storage, retained values and capacity copying remain unchanged. `T::clone` still has its existing caller-defined contract; use `Rc<T>` for JavaScript-style object identity.

The fix covers removed values from successful mutations and ownership of a rejected assignment on reserve failure. It does not introduce a destruction order between different removed values. Arbitrary reentrant `Clone` implementations are not supported by this change. Existing documented JavaScript/Rust validation differences remain in the oracle report.

## Verification

Checks ran on Windows with Rust/Cargo1.85.1 and Node24.15.0 from an isolated O: worktree based on main5ca9f593. After rebasing onto mainbb6eada, all53 focused tests, both Clippy commands and full-workspace formatting passed again. The implementation/test hashes match the original oracle run. Logs, exact source hashes and raw oracle results are in `results/circular-list-drop/`.

- Negative control against copies of the unchanged base utility modules: all five new tests failed from storage borrow panics. `baseline.log` retains the failures. The baseline was built as a standalone utility rlib, not as a workspace test.
- Fixed normal Cargo run:53 tests passed, zero failed or ignored. This includes five new regression functions and48 existing CircularList tests. A separate full xterm-core all-target/all-feature run against mainbb6eada plus this fix passed1528 tests across232 summaries with zero failed or ignored.
- The new tests cover all three assignment overloads with replacement and removal, length expansion with multiple detached slots and nested mutation, growing and shrinking capacity with preserved live identity, panic propagation and reuse, and overwritten push cleanup before the trim event. Test captures use weak list handles, not reference cycles.
- Existing Event/CircularList source oracle:26 unchanged upstream tests containing95 assertions,25 directed checks,96 native tests and30,055 compared states/traces passed. Seed `0xc1ac01a7`;20,047 list states and10,008 event traces were compared. These counts are different measures, not30,055 original tests.
- Warning-denied native core/web all-target/all-feature Clippy, warning-denied xterm-web WASM library Clippy, and exact full-workspace formatting passed. These are scoped lint checks, not the full workspace runtime/platform matrix.

```sh
cargo test --locked -j2 -p xterm-core --test circular_list_drop --test circular_list --test circular_list_edges --test circular_list_splice --test circular_list_numbers
cargo clippy --locked -j2 -p xterm-core -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --lib --target wasm32-unknown-unknown -- -D warnings
CARGO_BUILD_JOBS=2 node scripts/check-events-and-circular-list.mjs
cargo fmt --all -- --check
```

The existing oracle was reused without modification. Its generated `verification.json`, `latest.json` and run directory are retained under this task's `differential/` directory; paths inside those unmodified raw reports refer to their original `results/events-circular-list/` location. Only this run's generated outputs were relocated, and the prior tracked snapshot was restored.

Browser runtime, native Linux/macOS, complete common-family closure312, full workspace/release/features and final equivalence are not established by this task. Overall equivalence remains NOT PROVEN.

## Independent review

The isolated task358 reviewer accepted HEADb141da9 against mainbb6eada. It independently reran all53 focused tests, exact full-workspace formatting and both warning-denied Clippy commands. It inspected success, recoverable failure and unwind ownership, checked27 retained source/implementation hash entries, and found no concrete problem. The reviewer did not rerun the report-writing oracle, browser suites or other native platforms. No source files changed during review.

## Temporary ownership

Owned root: `O:/xterm-rust-swarm-16/tmp/Y16`, with regular private `target/` and `tmp/` directories. `tmp/baseline/` holds copied utility modules and two standalone negative-control binaries. No server or browser was started. These disposable paths must be removed after review/integration while preserving the source branch and tracked evidence. No other worktree or original reference is a cleanup target.
