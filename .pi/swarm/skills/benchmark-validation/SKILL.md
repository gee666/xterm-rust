---
name: benchmark-validation
description: Use when preparing matched production JavaScript and release Rust measurements, testing benchmark failures, or validating workload and timing receipts before publishing comparisons.
---

# Benchmark validation

Use current canonical main, not launch master. The protocol and caller/adapter responsibilities are in `docs/benchmark-protocol.md`.

Run `node --test scripts/matched-benchmark.test.mjs scripts/benchmark-contract.test.mjs scripts/full-trace-capture.test.mjs scripts/quality-process.test.mjs`. Pure protocol fixtures are not application performance results.

The actual headless recipe is in `docs/matched-headless-performance.md`. Import independently verified production bundles with `matched-benchmark-import.mjs`, build the private release worker with `matched-benchmark-native-build.mjs`, capture source expectations using `matched-benchmark-workloads.mjs`, and run `matched-benchmark-preflight.mjs`. Both real engines must match full RICH2 state/events/replies and exact addon outputs before any timed repetitions. Adapters reject failed or stale paired receipts.

Public JavaScript callback latency includes its host queue, unlike native synchronous writes. Do not claim those ratios are parser speedups. The separate `--synchronous-core` workload mode uses the unchanged production internal `CoreTerminal.writeSync`, which upstream deprecates and does not support reliably with async handlers. It applies only to the supplied builtin synchronous workloads, never browser rendering or public API throughput generally. Keep the modes and raw receipts separate. Do not retry or drop unfavorable timings.

Run `node scripts/benchmark-runner.mjs --config CONFIG.json` with owned scratch under repository tmp or authorized O: temporary root, durable output outside disposable scratch, exact executable commands and a bounded process deadline. Keep Cargo -j2 and current four-heavy/two-browser coordination. Rehash actual source/artifact/runtime pins before and after execution. Setup, input decoding, snapshot/hash validation and disposal stay outside the interval.

Inspect the durable receipt, not only process exit. Every warmup and at least ten measured samples must validate, with median/p95 and all raw values retained. Process/output/completion mismatch or cleanup failure means failed with null summary. Review adapters independently because receipt assertions alone cannot prove completion or provenance. Keep overall equivalence NOT PROVEN while requirements remain incomplete. Refresh timings when measured production sources change; sizes/LoC remain task333.
