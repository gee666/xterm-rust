---
name: task-platform-check
description: Use when checking common task scheduling, platform detection and Windows linefeed wrap behavior against the xterm.js source.
---

Run from repository root:

```sh
cargo test -p xterm-core --test task_queue --test task_queue_edges --test platform_windows
node scripts/task-platform-oracle.mjs
```

Install source_repo's locked npm dependencies first. The script runs the original esbuild and actual WindowsMode Mocha suite, then uses Node 24 TypeScript stripping for supplemental unchanged assertions and deterministic plus seed 0x715cad queue/platform/wrap traces compared with the Rust example. It writes source hashes, environment and exact counts to results/task-platform/oracle.json and retains mismatches in counterexample.json. Cargo is configurable with CARGO; all subprocesses have 120-second timeouts and share the repository target directory. Use the checked-in build profile to limit disk growth.

The actual Mocha suite exercises real BufferService/OptionsService construction. Its three assertions do not prove live Windows terminal linefeed integration; supplemental structural fixture runs are reported separately. Real browser timer/idle dispatch is task84, and engine linefeed/resize/copy integration is task28. Do not claim those from algorithm-only tests. See docs/task-scheduling-acceptance.md for exact branch requirements.
