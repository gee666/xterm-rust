---
name: write-buffer-check
description: Use when changing WriteBuffer queue ordering, parser continuation, time slicing, synchronous drains or host scheduling adapters.
---

# WriteBuffer checks

Run from the repository root with Node 24.15 or newer and Rust 1.85 or newer:

```text
node scripts/write-buffer-oracle.mjs
cargo test -p xterm-core --test write_buffer
cargo clippy -p xterm-core --test write_buffer -- -D warnings
```

After retaining these outputs as `results/write-buffer/final-{oracle,tests,clippy}.log` and the owned rustfmt check as `final-fmt.log`, run `node scripts/write-buffer-evidence.mjs`. It validates counts and records exact hashes of the implementation, tests, fixture and logs. Do not run it against stale logs after code changes.

The oracle transforms read-only source modules into `tmp/write-buffer-oracle/`. It runs all 19 unchanged original cases with their original async assertions, then replaces the host timer and clock for 128 deterministic traces of 64 steps. Seed `0x5717eb0f` covers FIFO input, empty text/bytes, synchronous writes/flushes, user-input fast paths, time slices and disposal. It records source hashes, exact case outcomes and fixture hash in `results/write-buffer/oracle.json`. Rust replays complete action/callback/parsed-event logs, pending units, backing queue length, offset, disposal and scheduled-task count after all 8192 steps.

Controlled Rust tests supplement the synchronous trace with pending/rejected futures, exact12ms, strict50M and50-chunk boundaries, reentrancy, errors and ownership cleanup. They use an explicit test scheduler and manual polling, not sleeps. Native and browser production adapters require separate integration tests; this helper does not establish their behavior.

Keep shell commands bounded by explicit timeouts. Preserve `.cargo/config.toml` profiles and monitor free disk space before large builds. Do not edit fixtures to hide a mismatch. Retain failing logs under `results/write-buffer/` and add a regression before regenerating.
