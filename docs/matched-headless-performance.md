# Matched headless performance

Task 329's adapters use the accepted `benchmark-runner.mjs` contract. They run the actual original production headless bundle and the release Rust terminal engine. They do not implement a second parser or delegate Rust execution to JavaScript.

No performance result is established by this document. Inspect the retained runner receipts and their status. A failed receipt has no ratio, including when only one engine's outputs match.

## What is timed

Each iteration constructs a fresh terminal outside the interval, configures the same dimensions, scrollback, default options and Unicode provider, and attaches the shared event projection. Write inputs are decoded before the interval. Serialization addon activation is outside the interval.

The JavaScript interval starts immediately before executing the ordered operations and ends after every public `Terminal.write` callback and synchronous resize/addon call returns. It includes public write scheduling. Rust uses `std::time::Instant` around the same operations on `CoreTerminal`, with synchronous writes and no asynchronous host queue. Native pending parser handlers or cleanup work fail the run. The source adapter checks that queued write data is empty.

This compares completed headless workloads through those two documented entry points. It is not a parser-only microbenchmark, native public host-queue comparison, startup benchmark, browser render benchmark, or proof of application equivalence. No DOM renderer, browser font, frame, GPU work, WebGL or image rendering is timed. Browser rendering requires its own comparable browser workloads and completion checks.

Construction, provider loading, input decoding, output capture, hashing, validation and disposal are outside the interval. Builtin parser event generation remains part of the real workload. JavaScript observers collect events as they occur; native events accumulate in the real engine and are drained after execution. Serializer calls and creation of their returned strings or UTF16 vectors are inside the interval; converting source strings to lossless UTF16 arrays is outside it.

Every iteration captures the full final RICH2 terminal state and the accumulated ordered events/replies. The original full-trace projection is reused without dropping its event column. All serialization outputs are retained as UTF16 units. As in the existing RICH2 protocol, the asynchronous `WriteParsed` notification is outside that projection. Every warmup and measured iteration must match the independently captured source state and addon outputs exactly. Intermediate state snapshots are not included in this workload protocol.

## Workloads and provenance

`scripts/matched-benchmark-workloads.mjs` defines seven deterministic cases: ASCII through UTF16 and byte entry points, Unicode11 with a split surrogate pair, grapheme-provider chunks, ANSI state/replies, bounded scrollback with shrinking/growing reflow, and actual live SerializeAddon calls. Shared JSON retains exact chunks, raw byte counts, input hashes, options and expected output. Bounded buffers keep full captures below the reviewed runner's output limit.

The original adapter loads production ESM bundles built by unchanged upstream `bin/esbuild.mjs --prod` commands. Task333 artifacts may be reused only after verifying their producer receipts and exact hashes. It observes builtin charset identities using a separate untimed terminal because production bundles do not export the charset table. It does not substitute a test bundle.

The native worker is `crates/xterm-addons/examples/matched_benchmark.rs`. The existing `serde_json` development dependency reads the workload; no new dependency is needed. Snapshot helpers come from the existing core test support. `matched-benchmark-native-build.mjs` performs the actual locked default-feature release build into the private target and pins Cargo inputs, source files and the executed binary. Both adapters rehash all declared sources and artifacts before and after execution. Reported source and artifact digests hash canonical arrays of named individual pins, not just Git revisions.

## Reproduction

Obtain genuine optimized original headless, Unicode11, Unicode-graphemes and SerializeAddon bundles. Create a JavaScript provenance file using the schema demonstrated by the retained `prepare-js.executed.mjs.txt` driver. Update paths and independently verify every source and artifact pin. Do not label a debug or ad hoc test bundle as production.

From an owned worktree with a private `target/`, allocate one compiler slot before running:

```text
node scripts/matched-benchmark-native-build.mjs
node scripts/matched-benchmark-workloads.mjs results/matched-performance329/javascript-provenance.json results/matched-performance329/rust-provenance.json results/matched-performance329/manifest.json
node scripts/matched-benchmark-preflight.mjs results/matched-performance329/manifest.json results/matched-performance329/javascript-provenance.json results/matched-performance329/rust-provenance.json results/matched-performance329/paired.json
```

Set `TEMP`, `TMP` and `TMPDIR` to its owned `tmp/` directory. The build script enforces Cargo `-j2`, bounded execution and minimum available memory/disk. The workspace's default release flags remain unchanged. The workload command records actual OS/CPU details and observed minimum positive clock deltas across 10,000 reads of both clocks, outside measurements. These observations describe granularity, not clock accuracy.

Create a runner config following `docs/benchmark-protocol.md`. Set `manifest` to the generated object, `scratchRoot` to an owned real temporary directory, `output` to a durable receipt, and a bounded `timeoutMs`. Each adapter uses Node with these arguments before the runner adds manifest/output paths:

```text
scripts/matched-benchmark-adapter.mjs --engine javascript --provenance results/matched-performance329/javascript-provenance.json --paired-receipt results/matched-performance329/paired.json
scripts/matched-benchmark-adapter.mjs --engine rust --provenance results/matched-performance329/rust-provenance.json --paired-receipt results/matched-performance329/paired.json
```

Run `node scripts/benchmark-runner.mjs --config CONFIG.json`. Retain the config, environment/resource observation, build receipts, pins and complete runner receipt. Historical absolute runtime paths become unavailable after owned temporary cleanup; the named source/artifact hashes remain evidence of what actually executed. Rebuild and refresh pins rather than trusting historical paths.

The paired preflight captures both real engines once and compares all seven cases before measured repetitions. A failing preflight retains outputs and forbids timing. Adapters reject failed preflight receipts or receipts for a different manifest, source or artifact. The runner then validates every sample before reporting medians, nearest-rank p95, throughput and paired ratios. It runs two warmups and ten measured iterations per case, all JavaScript cases before all Rust cases. Each sample uses a fresh terminal; warmup warms process code, not terminal contents. Preserve all raw values and report machine contention/order bias. Results from this checkpoint must not be described as final integrated performance while main or functionality is unfinished.
