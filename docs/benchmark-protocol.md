# Matched benchmark protocol

Task 326 provides validation and orchestration, not measured performance. Task 329 still needs actual production JavaScript and release Rust adapters, source-derived workload manifests and matched measurements. Synthetic fixtures under `scripts/test-fixtures/benchmark-*` test the protocol only. Do not publish their durations or hashes as application evidence.

The runner reuses `quality-process.mjs` for bounded processes and owned process-tree termination. It reuses the strict RICH2 decoder in `full-trace-capture.mjs` for terminal outputs. No dependency or alternate terminal engine was added.

## Commands

```text
node --test scripts/benchmark-contract.test.mjs
node scripts/benchmark-runner.mjs --config path/to/config.json
```

The config is JSON with `manifest`, `adapters`, `scratchRoot`, `output` and positive integer `timeoutMs`. `manifest` contains the workload object below. `adapters` has exactly `javascript` and `rust` entries, each with executable `program`, string-array `args` and working directory `cwd`. The executable is invoked directly without a shell. Existing arguments precede these runner arguments:

```text
--benchmark-manifest ABSOLUTE_SHARED_JSON_PATH --benchmark-output ABSOLUTE_FRESH_RESULT_PATH
```

Create `scratchRoot` as an owned real directory under repository `tmp/` or the authorized O: run temporary root. The runner creates a unique child, never reuses prior adapter results, and removes that child in `finally`. It preserves unrelated siblings. `output` is a durable receipt outside disposable scratch. Existing success is invalidated before validation. Both successful and failed executions write a receipt. The CLI exits nonzero on failure.

Adapter stdout/stderr and JSON results are limited to 8 MiB per file. A larger result fails rather than dropping snapshots. Choose bounded workloads or explicitly revise this limit with tests before larger scrollback captures. A process timeout, signal, nonzero exit, missing output, malformed output or mismatch fails the whole run, retaining command status, logs and available raw results. Failed runs have no summary or ratios. Cleanup failure is also failure.

## Workload manifest

Version 1 requires exactly these top-level fields:

- `schemaVersion` is 1.
- `execution` is `terminal` or `browser-rendering`.
- `environment` is an object recording the actual shared CPU, OS, architecture, timing granularity, concurrency and repetition order. Browser cases additionally require `viewport`, `dpr`, `font`, `palette` and `frameCompletion`. Supply actual matching values and include all other relevant page settings.
- `warmup` is an integer of at least 1. `repetitions` is an integer of at least 10.
- `cases` is a nonempty array with distinct case IDs.

Each case has exactly `id`, `options`, `operations`, `inputBytes`, `inputSha256`, `operationCount`, `expectedOutput` and `expectedOutputSha256`.

`options` has `cols` of at least 2, `rows` of at least 1, nonnegative `scrollback`, `unicode` and `settings`. `unicode` has nonempty `provider` and `version` strings plus a `settings` object. Terminal `settings` records all other applicable options. It cannot duplicate the four named option fields. Nothing may silently select a different Unicode provider or source default on one engine.

Operations are ordered and chunk boundaries are significant:

```json
{"kind":"write","encoding":"utf8-bytes","data":"YQ=="}
{"kind":"write","encoding":"utf16le","data":"YQA="}
{"kind":"resize","cols":80,"rows":24}
{"kind":"addon","addon":"search","method":"findNext","arguments":["text",{}]}
```

`data` is canonical base64. UTF16LE includes lone surrogates losslessly and must contain complete two-byte units. UTF8-bytes is a byte stream, including malformed UTF8 when relevant. Both engines must use the named write entry point and exact same chunks. Recompute `inputBytes` from all writes and `inputSha256` from their concatenated raw bytes. `operationCount` counts every operation, including resize and addon calls. Empty operation lists and zero-byte writes are supported. An empty manifest is not a measurement.

`expectedOutput` has exactly `terminalTrace` and `addons`. `terminalTrace` is a nonempty RICH2 capture with its event column, as emitted by existing full-trace adapters. It records full cells, UTF16 units, packed attributes, both buffers and their saved/scrollback metadata, cursor, modes, Unicode version, events and replies. At minimum include final state. Its final dimensions must agree with the last resize or initial options, and its final Unicode version must agree with the options. `addons` is an object with complete source-derived addon outputs, such as search spans/events, exact serialization output or rendering pixels/frame assertions. A helper-only state capture is insufficient for actual browser measurements.

Capture the expected output from the unchanged reference before measuring. `outputHash` in `scripts/benchmark-contract.mjs` decodes RICH2 and hashes canonical `{terminal, addons}` JSON. `caseIdentity` validates/recounts a case and hashes all its fields, including options and chunk boundaries. `validateManifest` hashes the complete validated manifest. `canonical` sorts object keys but preserves array order and UTF16 strings. It rejects nonfinite numbers, negative zero, undefined and non-JSON objects rather than silently normalizing them.

## Adapter result

The result has exactly `schemaVersion`, `engine`, `profile`, `manifestSha256`, `timing`, `build` and `cases`.

- Engine is `javascript` or `rust`. Profile must respectively be `production` or `release`.
- `manifestSha256` is the hash returned by `validateManifest`.
- `timing` is exactly `{"clock":"monotonic","unit":"ms","startupIncluded":false,"completion":"awaited"}`. This protocol measures steady-state work only. A startup-inclusive experiment needs a separately identified protocol, not a relabelled result.
- `build` has nonempty `commands`, `toolchain` and `flags`, plus lowercase SHA256 `sourceSha256` and `artifactSha256`. Commands are a nonempty string array. Record the actual optimized build commands, lockfiles/features, optimization/LTO/strip/panic flags and toolchain versions. Verify hashes against executed artifacts.
- `cases` follows manifest order with no omissions or extras. Each row has `id`, `caseSha256`, `inputBytes`, `operationCount`, `warmup` and `measurements`.

Each sample has exactly `durationMs`, `completion`, `pendingOperations`, `output` and `outputSha256`. Duration must be finite and nonnegative, completion must be `completed`, pending operations must be zero, and the reported output hash must equal both the actual complete output and the expected output. Every warmup and timed sample is validated. Warmup count and repetition count must match exactly. Validation/hashing happens outside the measured engine interval.

Adapters must actually await write callbacks, addon promises and equivalent browser frame/GPU completion before stopping the monotonic clock. These receipt fields detect missing completion, but cannot prove that an adapter telling a lie awaited work. Review adapter implementation and independently compare actual output before using its measurements. Similarly, supplied build/environment metadata is a contract, not automatic proof of provenance. No adapter or actual workload was delivered by task 326.

## Aggregation

Raw samples and outliers remain in the durable receipt. Median uses the middle sorted value for odd counts or the arithmetic mean of the two central values for even counts. p95 uses the nearest-rank value at `ceil(0.95 * n) - 1`, with zero-based indexing. Warmup is not part of these statistics.

Throughput is independently recomputed bytes or operations times 1000 divided by median milliseconds. A measured zero duration stays zero, with null throughput instead of infinity. The paired ratio is JavaScript median divided by Rust median, or null if the Rust denominator is zero. Both sides must validate before any ratio is emitted. Never compare browser rendering against native parsing or claim full equivalence from a valid timing receipt.
