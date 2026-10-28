# Native queue ordering review

Task169 is an independent PA03 review. Implementation remains with Epictetus for the portable queue, Antigone for native integration and Xavier for browser integration. The review does not change their implementation files.

## Source inspection

Inspected `source_repo/src/common/input/WriteBuffer.ts`, `common/CoreTerminal.ts`, `headless/Terminal.ts`, the Rust `input/write_buffer{,_drain,_types}.rs`, and `headless/{native_queue,queue_bridge,queued,terminal,events}.rs`. The separate callback/data queues, empty-text truthiness, strict 50M and 12ms thresholds, synchronous partial-error state, headless unchanged-size resize suppression and queued parsed-event suppression are deliberately preserved. Source inspection alone is not a runtime pass.

## Shared-host microtask concern

The reviewed NativeWritePump version snapshots all timers, invokes all of them, then polls ready futures. JavaScript drains Promise/queueMicrotask jobs between timer callbacks. Two WriteBuffers sharing one native pump can therefore observe a different ordering when timer A returns an already-ready pending handler and timer B is also ready. A rejected A handler additionally needs the uncaught-error report before false continuation, all before B's later timer.

This is a source-read concern pending the exact paired trace, not a claimed demonstrated counterexample yet. Existing five source/native scenarios used one QueuedTerminal with its own pump and did not establish cross-queue ordering. Antigone reopened task128 and owns the native fix/regressions. Prior acceptance and fifteen scoped tests remain historical scoped evidence, not proof of this extra path.

`scripts/native-queue-order-oracle.mjs` supplies the independent source probe. It transforms only type/module syntax and import URLs in the unchanged reference modules, following the existing write-buffer-check skill. Its small loader is intentionally separate because importing the existing executable oracle would rerun nineteen tests and overwrite that oracle's fixtures. Both resolved and rejected A cases record complete action/callback/parsed/error logs, pending units and outstanding timers. A real `setImmediate` macrotask checkpoint drains all current microtasks; it does not guess a fixed number of Promise awaits or sleep.

The source runner creates a unique attempt, records running status before reading reference inputs and retains source SHA-256 values. It never executes Rust or substitutes a Rust result for JavaScript. Failed source assertions must remain failed. Native comparison is owned by task128 and independently checked under169.

## Planned additional PA03 checks

- Direct reset while a handler is actually pending must retain the decoded chunk and parser continuation, then finish the original callback before subsequent input.
- Disposing the terminal while a handler is pending must clear queued callbacks/timers and prevent resumed parsing, without pretending an unpolled Rust future completed.
- Callback or parsed-event failure must retain source partial queue/terminal state. Reentry must not acquire an already-held RefCell borrow.
- Browser microtask/error delivery and native Windows/Linux/macOS runtime checks remain separate. No native helper result proves browser scheduling.

## Execution status

**UNRUN.** Source and native runner execution is paused by the shared P: disk-headroom policy. The script is written but has not been syntax-checked or executed yet. No successful source trace, Rust regression result or fix verification is claimed here. Resume only after the resource owner clears the pause, retain each attempt and inspect actual output before accepting the finding or fix.

Commands after clearance:

```text
node scripts/native-queue-order-oracle.mjs
cargo test -p xterm-core --test headless_queue --test headless_queue_edges
```

Do not regenerate unrelated oracle fixtures or weaken assertions. Overall xterm.js equivalence remains NOT PROVEN.
