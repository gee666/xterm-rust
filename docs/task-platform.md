# Task queues, platform detection and Windows wrapping

Task #58 ports these read-only reference modules to `xterm-core`:

| Reference | Rust | Verification |
| --- | --- | --- |
| common/TaskQueue.ts | src/task_queue.rs | tests/task_queue.rs, tests/task_queue_edges.rs, queue trace in examples/task_platform_trace.rs |
| common/Platform.ts | src/platform.rs | tests/platform_windows.rs and platform trace |
| common/WindowsMode.ts | src/windows_mode.rs | three faithful Rust assertion cases, unchanged JS assertions and wrap trace |
| common/Version.ts | src/version.rs | exact version assertion |

Paths in the Rust column are relative to `crates/xterm-core/`. The detailed acceptance checklist is [task-scheduling-acceptance.md](task-scheduling-acceptance.md). All 36 common Types declarations and 11 TestUtils exports have explicit ownership mappings in [common-type-mapping.md](common-type-mapping.md). Those mixed contracts are not marked fully implemented merely by this task.

## Contracts

`TaskQueue<S>` uses a supplied `Scheduler` for requests, cancellation, a performance clock and warning reporting. The portable core does not import browser or operating-system APIs. `CallbackKind::Priority` uses a 16 ms dispatch-time budget. `TaskQueue::idle` selects true idle callbacks or priority fallback from an explicit host capability flag.

Tasks are `Fn(&TaskQueue<S>) -> bool`. True repeats the current task, false advances the index. Mutable captures use `Cell` or `RefCell`. The queue releases its internal borrows before invoking callbacks. Callback enqueue, clear and recursive flush therefore preserve the source index semantics, including its clear-then-enqueue skip behavior. No hidden repeat limit is imposed on synchronous flush.

A scheduler's `request` returns `Result<Handle, Error>`. Enqueue and dispatch propagate failures with the source's partial state retained. The queued task remains after an enqueue scheduling failure; a failed dispatch reschedule leaves no pending token. This error channel prevents browser adapters from throwing through a live Rust `RefMut`. Flush and clear do not request callbacks and remain infallible apart from user callback panics. Callback panic preserves the pending/index ordering tested in `task_queue_edges.rs`; JS/Rust exception traces are not yet differentially compared.

Handles clone shared state, while `downgrade` gives host callbacks a `WeakTaskQueue`. Last-owner drop cancels pending work. Tasks receive the queue argument to avoid capturing a strong self-reference. Debounced tasks clear the previous callback before setting the next; dispose clears work and a later set is accepted, matching upstream.

`Platform::detect` consumes navigator metadata and the presence of `process.title`. Browser process shims do not force Node detection. Missing navigator outside Node returns `MissingNavigator` instead of guessing a host OS. Browser integration must supply actual metadata. Flags retain source case rules, including Safari's surprising start-only regex match: `Safari Chrome` matches, `Chrome Safari` does not. Safari major version is a JavaScript-number-compatible `f64`. Zoom remains one regardless of device-pixel ratio because that is the source implementation.

The Windows helper accepts the real circular list of shared `BufferLine` objects. It uses signed previous-row ring indexing, not a slice approximation. Only null and U+0020 are excluded. Legacy `BufferLine.get` produces a null-like code for missing columns, and an empty combined string yields NaN, which marks wrapped. RefCell borrows end before mutating the current line, even if storage aliases the previous row.

## Observed verification

Commands on native Windows with Node v24.15.0 and cargo/rustc 1.85.1:

```sh
cargo test -p xterm-core --test task_queue --test task_queue_edges --test platform_windows
node scripts/task-platform-oracle.mjs
cargo clippy -p xterm-core --lib --test task_queue --test task_queue_edges --test platform_windows --example task_platform_trace -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/task_queue.rs crates/xterm-core/src/platform.rs crates/xterm-core/src/windows_mode.rs crates/xterm-core/src/version.rs crates/xterm-core/tests/task_queue.rs crates/xterm-core/tests/task_queue_edges.rs crates/xterm-core/tests/platform_windows.rs crates/xterm-core/examples/task_platform_trace.rs
```

- 18 focused Rust tests passed, none failed or ignored.
- The original WindowsMode Mocha suite passed all three assertions using the real BufferService, OptionsService and BufferLine after the upstream esbuild build. Its raw Mocha JSON is retained as original-windows.json. Three additional unchanged assertion executions use a structural fixture; those are supplemental, not counted as six distinct original tests.
- 4,517 JS/Rust operation results matched, including 1,500 seeded queue iterations, 2,500 environment cases and 500 Windows-wrap cases. Seed is 7,429,293, hexadecimal 0x715cad. Directed deadline/reentrancy cases supplement the random corpus.
- Focused Clippy and rustfmt results are recorded under `results/task-platform/` together with source hashes and machine-readable counts. Historical integration compilation failures are retained separately, not reported as passing runs.

Install the original's locked npm dependencies before running the oracle. It invokes the upstream core esbuild script and the actual original WindowsMode Mocha suite, without modifying source or assertions. For its supplemental differential fixtures it strips only imports, exports and TypeScript syntax from reference runtime modules. The structural BufferService fixture is explicitly reported separately in `oracle.json`. The Rust trace endpoint is a deterministic test adapter, not a user terminal demo. It shares the repository target directory and honors the low-artifact dev/test profile introduced after the disk-space interruption.

## Remaining integration checks

Full equivalence is NOT PROVEN. Task #84 owns the real browser timer/idle adapter and live dispatch/cancellation tests. Task #28 owns actual Windows-mode linefeed/resize/copy integration. Native Linux/macOS execution is not established here. Entire-workspace lint/build/tests, original full service suites, browser-backed terminal behavior and independent review are separate gates. This utility report must not be used as evidence that they passed.
