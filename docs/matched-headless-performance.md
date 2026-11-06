# Matched headless performance

Task 329's adapters use the accepted `benchmark-runner.mjs` contract. They run the actual original production headless bundle and the release Rust terminal engine. They do not implement a second parser or delegate Rust execution to JavaScript.

The retained Windows checkpoint contains passing actual measurements in two distinct modes. Both use the same seven input/option/chunk manifests and full expected terminal/addon outputs. A failed receipt has no ratio, including when only one engine's outputs match. Overall equivalence remains NOT PROVEN.

## Observed checkpoint

Production terminal sources match integrated main `14139449bae2792afb019f42e8438b074938cfe7`, with no crate delta from task333's `6cfd03d7` freeze. Benchmark adapters were added afterward. The public-callback experiment used adapter commit `f837222df414cd9eac23ff9d01ad955d71856aa2`; each experiment's provenance also retains the exact runtime-script hashes. The executed native worker SHA256 is `039b1f9e63f559eb3a90fc49faad38007308dd4f2167d55945848a50786f8a45`.

Machine: native Windows 11 Home, build `10.0.26300`, x64, AMD Ryzen 9 7900X, 24 logical processors, Node `v24.15.0`. Both clocks had a minimum observed positive delta of 100 ns over 10,000 reads. Rust toolchain details, locked dependency identity, optimization flags, all raw samples and resource observations are retained. Other coordinated browser/compiler lanes could be active. This is not an idle-machine, CPU-isolated, cross-platform or randomized-order study.

Each mode passed seven actual paired preflight captures before timing. Every one of the two warmups and ten measured samples per case and engine passed full output validation. That is 84 samples per engine per mode, including warmups, not 84 measured samples. The two modes' input hashes and complete expected-output hashes also match case by case.

### Synchronous production-core mode

The [receipt](../results/matched-performance329/measurements.json), [paired captures](../results/matched-performance329/paired.json), [manifest](../results/matched-performance329/manifest.json) and [resource record](../results/matched-performance329/measurement-resources.json) retain the complete run. Values below are milliseconds, rounded for display only.

| Workload | JS median | JS p95 | Rust median | Rust p95 |
| --- | ---: | ---: | ---: | ---: |
| ascii-utf16 | 0.8142 | 1.5780 | 1.5964 | 2.2443 |
| ascii-utf8 | 0.6202 | 1.0565 | 1.4716 | 1.8156 |
| unicode11-stream | 0.4798 | 1.6475 | 0.4819 | 0.4936 |
| graphemes-stream | 0.4580 | 0.6591 | 0.6372 | 1.0637 |
| ansi-controls-replies | 0.4407 | 0.5943 | 0.5309 | 0.8761 |
| scrollback-resize-reflow | 0.2025 | 0.2837 | 0.2135 | 0.2308 |
| serialize-live-addon | 0.2594 | 0.3049 | 0.3362 | 0.5540 |

Rust's observed median was higher in every synchronous case, with Unicode11 nearly equal. No speedup is claimed. Small samples, fixed engine order, JIT behavior, allocator/event-recording differences and machine contention limit interpretation.

This mode calls the actual production JavaScript `CoreTerminal.writeSync`, not `InputHandler` test helpers or a substitute engine. Upstream marks this internal method deprecated and unreliable with asynchronous parser handlers. The unchanged warning is retained in capture/measurement logs. These workloads install no asynchronous/custom handlers and must match the callback mode's full output. This mode is a limited synchronous-core comparison, not a supported public API throughput guarantee.

### Public callback latency mode

The separate [public-callback receipt](../results/matched-performance329/public-callback/measurements.json) retains the first experiment and all 20 archived producer/config/capture/result files are hash-checked in its `archive.json`. This mode includes the original public write queue's scheduling delays, unlike the synchronous native entry point.

| Workload | JS median | JS p95 | Rust median | Rust p95 |
| --- | ---: | ---: | ---: | ---: |
| ascii-utf16 | 13.7912 | 14.7373 | 1.4695 | 1.5373 |
| ascii-utf8 | 13.5102 | 14.9810 | 1.5158 | 1.8919 |
| unicode11-stream | 29.8156 | 32.4083 | 0.4824 | 0.5317 |
| graphemes-stream | 29.8802 | 30.6075 | 0.6556 | 0.7986 |
| ansi-controls-replies | 14.6234 | 15.2475 | 0.4949 | 0.5469 |
| scrollback-resize-reflow | 30.3745 | 31.0245 | 0.2187 | 0.2465 |
| serialize-live-addon | 30.2462 | 31.2779 | 0.3342 | 0.3788 |

These large latency differences must not be presented as parser/engine speedups. Public latencies increased by roughly 15 ms per awaited chunk on this Windows run, consistent with scheduling overhead. Queue delay was not separately instrumented, so this is not a direct measurement of a particular Windows timer setting. The synchronous experiment shows why ignoring the different entry points would mislead. Neither run was retried to improve its numbers.

Native addon tests passed 234 tests across 58 result summaries, with no failures/ignored/filtered tests. Warning-denied addon all-target/all-feature Clippy and addon formatting passed. Nine actual release-worker rejection controls returned errors without panics or successful output files. Importer controls also verified genuine bundle reuse, destination/reference guards, stale-success invalidation, source/bundle drift detection and owned failure cleanup. The shared protocol/adapter suite passed 65 tests. Exact aggregate `cargo fmt --all -- --check` failed with Windows OS206; the member pass does not replace that required gate. Full workspace, browser and native OS coverage remain the port's separate requirements.

## What is timed

Each iteration constructs a fresh terminal outside the interval, configures the same dimensions, scrollback, default options and Unicode provider, and attaches the shared event projection. Write inputs are decoded before the interval. Serialization addon activation is outside the interval.

In public-callback mode the JavaScript interval starts immediately before executing the ordered operations and ends after every public `Terminal.write` callback and synchronous resize/addon call returns. It includes public write scheduling. Rust uses `std::time::Instant` around the same operations on `CoreTerminal`, with synchronous writes and no asynchronous host queue. Native pending parser handlers or cleanup work fail the run. The source adapter checks that queued write data is empty. The optional synchronous-core mode instead uses the production internal `CoreTerminal.writeSync` and is limited to these builtin synchronous workloads.

This compares completed headless workloads through those two documented entry points. It is not a parser-only microbenchmark, native public host-queue comparison, startup benchmark, browser render benchmark, or proof of application equivalence. No DOM renderer, browser font, frame, GPU work, WebGL or image rendering is timed. Browser rendering requires its own comparable browser workloads and completion checks.

Construction, provider loading, input decoding, output capture, hashing, validation and disposal are outside the interval. Builtin parser event generation remains part of the real workload. JavaScript observers collect events as they occur; native events accumulate in the real engine and are drained after execution. Serializer calls and creation of their returned strings or UTF16 vectors are inside the interval; converting source strings to lossless UTF16 arrays is outside it.

Every iteration captures the full final RICH2 terminal state and the accumulated ordered events/replies. The original full-trace projection is reused without dropping its event column. All serialization outputs are retained as UTF16 units. As in the existing RICH2 protocol, the asynchronous `WriteParsed` notification is outside that projection. Every warmup and measured iteration must match the independently captured source state and addon outputs exactly. Intermediate state snapshots are not included in this workload protocol.

## Workloads and provenance

`scripts/matched-benchmark-workloads.mjs` defines seven deterministic cases: ASCII through UTF16 and byte entry points, Unicode11 with a split surrogate pair, grapheme-provider chunks, ANSI state/replies, bounded scrollback with shrinking/growing reflow, and actual live SerializeAddon calls. Shared JSON retains exact chunks, raw byte counts, input hashes, options and expected output. Bounded buffers keep full captures below the reviewed runner's output limit.

The original adapter loads production ESM bundles built by unchanged upstream `bin/esbuild.mjs --prod` commands. Task333 artifacts may be reused only after verifying their producer receipts and exact hashes. It observes builtin charset identities using a separate untimed terminal because production bundles do not export the charset table. It does not substitute a test bundle.

The native worker is `crates/xterm-addons/examples/matched_benchmark.rs`. The existing `serde_json` development dependency reads the workload; no new dependency is needed. Snapshot helpers come from the existing core test support. `matched-benchmark-native-build.mjs` performs the actual locked default-feature release build into the private target and pins Cargo inputs, source files and the executed binary. Both adapters rehash all declared sources and artifacts before and after execution. Reported source and artifact digests hash canonical arrays of named individual pins, not just Git revisions.

## Reproduction

Obtain genuine optimized original headless, Unicode11, Unicode-graphemes and SerializeAddon bundles and the producer size/build receipts from task333's unchanged upstream build commands. Do not label a debug or ad hoc test bundle as production. Import them without copying dependencies:

```text
node scripts/matched-benchmark-import.mjs source_repo results/distribution333/original/original-sizes.json tmp/production-js-reproduced results/matched-performance329/javascript-provenance.json
```

The source path must contain the unchanged physical reference and its actual generated production bundles. The artifact destination must be a fresh child of owned repository `tmp/`. The importer verifies all reference pins and each actual bundle before/after copying, rejects non-production recipes and preserves the source. It invalidates previous provenance on failure and removes only its freshly created artifact directory. The original experiment's `prepare-js.executed.mjs.txt` and runtime-sealing drivers remain archived as the exact recipe used then.

From an owned worktree with a private `target/`, allocate one compiler slot before running:

```text
node scripts/matched-benchmark-native-build.mjs
node scripts/matched-benchmark-workloads.mjs results/matched-performance329/javascript-provenance.json results/matched-performance329/rust-provenance.json results/matched-performance329/manifest.json --synchronous-core
node scripts/matched-benchmark-preflight.mjs results/matched-performance329/manifest.json results/matched-performance329/javascript-provenance.json results/matched-performance329/rust-provenance.json results/matched-performance329/paired.json
```

Omit `--synchronous-core` to reproduce the separate public-callback mode. Keep each mode's manifest, provenance, preflight and measurement receipt separately. Set `TEMP`, `TMP` and `TMPDIR` to its owned `tmp/` directory. The build script enforces Cargo `-j2`, bounded execution and minimum available memory/disk. The workspace's default release flags remain unchanged. The workload command records actual OS/CPU details and observed minimum positive clock deltas across 10,000 reads of both clocks, outside measurements. These observations describe granularity, not clock accuracy.

Create a runner config following `docs/benchmark-protocol.md`. Set `manifest` to the generated object, `scratchRoot` to an owned real temporary directory, `output` to a durable receipt, and a bounded `timeoutMs`. Each adapter uses Node with these arguments before the runner adds manifest/output paths:

```text
scripts/matched-benchmark-adapter.mjs --engine javascript --provenance results/matched-performance329/javascript-provenance.json --paired-receipt results/matched-performance329/paired.json
scripts/matched-benchmark-adapter.mjs --engine rust --provenance results/matched-performance329/rust-provenance.json --paired-receipt results/matched-performance329/paired.json
```

Run `node scripts/benchmark-runner.mjs --config CONFIG.json`. Retain the config, environment/resource observation, build receipts, pins and complete runner receipt. Historical absolute runtime paths become unavailable after owned temporary cleanup; the named source/artifact hashes remain evidence of what actually executed. Rebuild and refresh pins rather than trusting historical paths.

The paired preflight captures both real engines once and compares all seven cases before measured repetitions. A failing preflight retains outputs and forbids timing. Adapters reject failed preflight receipts or receipts for a different manifest, source or artifact. The runner then validates every sample before reporting medians, nearest-rank p95, throughput and paired ratios. It runs two warmups and ten measured iterations per case, all JavaScript cases before all Rust cases. Each sample uses a fresh terminal; warmup warms process code, not terminal contents. Preserve all raw values and report machine contention/order bias. Results from this checkpoint must not be described as final integrated performance while main or functionality is unfinished.
