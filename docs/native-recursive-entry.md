# Native recursive queued parser entry

Task #332 repairs the retained Antigone `headless_async_resize.rs` counterexample. On asynchronous delivery of `A BEL B`, a Bell listener resizes the terminal from 8 columns to 6. Source `CoreTerminal.resize` calls `WriteBuffer.flushSync`. The asynchronous queue still contains its active chunk, so flush parses that same chunk recursively before resizing. Source produces `AABB`, with events `bell, bell, resize:6, callback, resize:6`.

The pre-fix native terminal skipped its queue when `Terminal.resize` ran from a listener and produced `AB`. `results/headless-async-reentry/before-fix.txt` retains a fresh failure against main `c0dd42be7420b828e9c2b0572c19374f6dcf7c5a`. The core tree hash `9f32e0aaea04bcdbf3d8905f7423d6f1c5b960c7` stayed unchanged throughout that run. The retained assertion was not weakened.

## Ownership changes

`QueuedTerminal` attaches its queue weakly to its Terminal. Public resize now flushes that queue, after the unchanged-dimensions early return and before resizing buffers. One live native queue per terminal is enforced with `HeadlessError::QueueAlreadyAttached`. Dropping the host removes the hook and its existing cleanup factory. Queue failures retain their typed error chain through `HeadlessError::Queue`.

The native queue uses `CoreTerminal::write_source_shared`. It borrows the engine for one parser action at a time. Bell delivery occurs after releasing the engine and decoded-buffer borrows, before EXECUTE resets preceding-join state. Nested parsing therefore executes through the actual parser and decoder. There is no parser clone, saved-prefix replay or special-case text duplication.

A decoded allocation has shared identity. A short nested decode overwrites the same allocation, but growth creates a new allocation while the outer parse retains the original one, matching the source Uint32Array reference. Each active parse keeps its own source, input offset, decoded length and starting cursor. Only genuinely paused async handlers save continuation fields on the engine. The dirty range survives nested completion, as the source dirty-row tracker does.

A scoped depth guard keeps parser disposal deferred until the outer active chunk returns. Shared services and emitters still dispose immediately. Both normal return and errors release the depth and Bell-delivery guards. There is no unsafe code or new dependency.

The direct mutable CoreTerminal driver remains available. Its bounded source traversal and the shared driver intentionally differ in ownership: an existing `&mut CoreTerminal` cannot release its caller's borrow for recursive callbacks. Both drivers reuse the same decoders, parser action interpreter, range arithmetic and end-of-input notification builder. The existing browser/public parser bridge still uses its original direct driver. The cooperative driver is wired only to Native QueuedTerminal.

## Reproduce

With the supplied source dependencies installed:

```text
node scripts/headless-async-reentry-oracle.mjs
cargo test -p xterm-core -j2 --test headless_async_resize --test headless_recursive_fixture --test headless_reentry --test headless_queue --test headless_queue_edges --test headless_disposal_contract --test native_queue_differential
cargo test -p xterm-core --all-targets --all-features -j2
cargo test -p xterm-core --doc -j2
cargo clippy -p xterm-core --all-targets --all-features -j2 -- -D warnings
```

`XTERM_SOURCE_ROOT` may point to an existing copy of the exact supplied reference with Node dependencies. The source runner builds into an owned tmp directory, invokes the actual original timer body synchronously and disposes each original terminal. It does not modify source files or assertions, use sleeps, or substitute a fake parser. Normal mode rejects fixture drift. `--generate` explicitly updates the reviewed source fixture.

The 16 directed cases cover async string and byte delivery, synchronous flush, idle and unchanged resize, queued tails, allocation growth to 5000 and 140000 source units, input, reset, disposal, nested reset/disposal, queued callback appends, typed listener failure, and combining/wide Unicode. Each compares dimensions, cursor, base/display offsets, disposal state, pending-unit accounting, queue offset/length, every retained row's UTF-16 text, ordered events/callbacks and errors. The nested error test checks that the original typed listener cause survives, rather than accepting an arbitrary error string. One extra native test checks duplicate attachment rejection and drop/replacement cleanup.

## Observed verification

Native Windows MSVC with Rust 1.85.1 and Node 24.15.0 passed 16 source cases and their 16 Rust comparisons. Core all-target/all-feature execution passed 1504 test functions in 227 summaries, plus 1 doctest. Warning-denied all-target/all-feature core Clippy passed. Raw commands and outputs, fixture hash and imported source hashes are retained under `results/headless-async-reentry/`. Changed-file formatting passed separately. None of these counts is a workspace or cross-platform pass.

The isolated review accepted `dbb46b2` after independently running 32 tests across nine focused native targets, the 16-case source oracle, scoped warning-denied Clippy and changed-file rustfmt. It found no code problems. It did not run workspace/browser or Linux/macOS gates. Its counts and scope are retained in `review.json`.

## Remaining scope

This repair establishes the concrete native Bell/resize recursive-entry contract. Callbacks invoked inside arbitrary custom parser handlers or other synchronous producer callbacks still need separate recursive-entry assertions and ownership work under parser closure task #313. The 16-case fixture is not the complete original InputHandler suite or full-state corpus. The existing 9306-state corpus remains unchanged and passed in the full core run. Browser queue behavior, all-feature workspace quality, native Linux/macOS and full equivalence remain separately tracked and NOT PROVEN.
