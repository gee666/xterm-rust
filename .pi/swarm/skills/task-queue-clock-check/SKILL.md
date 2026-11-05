---
name: task-queue-clock-check
description: Use when changing task queue dispatch, scheduler clocks or queue error propagation and checking reentrant clock behavior against unchanged source.
---

# Task queue clock checks

Read `docs/task-queue-clock-gap.md`. Run `node scripts/task-queue-clock-oracle.mjs --browsers` to check the shared32-case fixture against unchanged source in Node and all three browsers. Set `XT_SOURCE_ROOT` and `PLAYWRIGHT_MODULE` for existing external installations. Use `--write` only when deliberately regenerating source expectations; inspect fixture changes before accepting them.

Run `cargo test --locked -j2 -p xterm-core --test task_queue_clock --test task_queue --test task_queue_edges --test task_queue_drop --test buffer_contracts --test buffer_cleanup_injection`. The native and browser tests share `crates/xterm-core/tests/fixtures/task-queue-clock.tsv` and its parser in `tests/support/task_queue_clock.rs`, without adding core test dependencies.

Run `WASM_TEST=wasm_task_queue_clock node scripts/browser-host-check.mjs`, followed by existing `wasm_task_queue_errors` and `wasm_host` targets. Follow the browser-host-check skill for wasm-bindgen0.2.100 and Playwright setup. Coordinate one bounded heavy/browser slot, `CARGO_BUILD_JOBS=2`, owned temporary paths and finite tool timeouts. Do not use the retired global build queue.

Retain exact logs, source and artifact hashes, and case counts in results. Do not commit generated WASM/binding binaries. A passing replay means32 controlled source traces, not32 unchanged upstream tests or full terminal parity. The unrelated existing alternate-buffer cleanup failure remains task371 until separately fixed; preserve its assertions and report it rather than skipping it.
