# Open task queue clock reentry mismatch

Task367 is confirmed but not repaired. The retained minimal probes are in `results/task-queue-clock/` on `swarm/A16`. They are counterexamples, not new passing parity tests or a complete browser run.

The source `TaskQueue._process` tests `_i < _tasks.length`, calls `performance.now()`, then looks up and invokes `_tasks[_i]`. Native `TaskQueue::process` clones `current_task()` in the loop condition before calling `Scheduler::now()`. A reentrant clock can therefore replace the task between these statements, with different callbacks executed.

The probe enqueues `old`, then clears and enqueues `new` in the first clock call during idle processing. It uses finite times and an ordinary completing task. The unchanged source executes only `new`. The current native core executes only `old` and then clears the replacement. Native Windows x64/Rust1.85.1 and Node24.15.0/TypeScript5.9.3 produced these retained outputs. Source and implementation hashes are recorded. No source assertion was altered.

```sh
XT_SOURCE_ROOT=/path/to/unchanged/source_repo node results/task-queue-clock/source-probe.mjs
cargo build --locked -j2 -p xterm-core
# Use the rlib Cargo just built and the matching target/debug/deps directory.
rustc --edition 2021 results/task-queue-clock/native-repro.rs --extern xterm_core=target/debug/deps/libxterm_core-HASH.rlib -L dependency=target/debug/deps -o tmp/clock-reentry.exe
./tmp/clock-reentry.exe
```

The source probe transpiles the unchanged file in memory and provides only controlled timer/clock/log hosts. The native probe links the normal Cargo-built library. This is not a fake terminal implementation. The probes do not establish real browser clock mutation behavior, missing-task error handling or cross-platform results.

The repair must look up the task after the clock callback. It must also handle clear-without-replacement with a typed error corresponding to the source's TypeError, not a panic, an old callback or successful no-op. The current `Scheduler::Error` can be `Infallible`, so choosing a sound generic queue-error representation is part of the work. Inspect existing queue consumers before changing public Rust results. Preserve all existing scheduler/capture-cleanup assertions and original differential traces. Broader throwing clock/cancellation behavior remains subject to full source-family audit312.

Task366 does not change any queue behavior. Its two reviewed commits are independent of this later counterexample-only commit. Do not claim task367 complete or merge it as an implementation repair.
