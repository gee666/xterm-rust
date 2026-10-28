# Shared acceptance case runner

Task 140 owns `scripts/acceptance-oracle.mjs` and its shared runner helpers. Family owners own adapters under `scripts/acceptance-adapters/`. The command is the executable entry point requested by User criteria 280-380. Demo cases use their separate `demo-e2e.mjs` entry point.

```
node scripts/acceptance-oracle.mjs --case img-01
node scripts/acceptance-oracle.mjs --case move-01
node scripts/acceptance-oracle.mjs --list
node --test scripts/acceptance-runner.test.mjs
```

`--list` lists declared contracts, not passes. A missing family adapter, missing Rust/WASM implementation, zero assertions, absent runtime, invalid receipt or mismatched capture returns nonzero. Internal missing implementation is unrun work, not an external blocker. There are no successful placeholder adapters.

## Adapter contract

Files are selected by exact case family, not arbitrary CLI module paths:

| Prefix | Adapter | Minimum runtime matrix |
| --- | --- | --- |
| img | image.mjs | chromium, firefox, webkit |
| gpu | webgl.mjs | chromium, firefox, webkit, retaining only source-defined skips |
| vt | terminal.mjs | Per exact User verification: native cases remain native; vt-28/29/30/31/34/40/41 require chromium, firefox and webkit actual WASM |
| move | move-to-cell.mjs | native deterministic method traces; real browser clicks remain separate integration gates |
| async | async-timers.mjs | native deterministic clock/operation traces; browser/window ownership remains a separate integration gate |

Each family module exports `async runCase({caseId, root, outputDirectory, signal, contract})`. Do not edit the shared dispatcher to add one case. The terminal family has multiple owners, so its shared `terminal.mjs` discovers `terminal/*.cases.mjs` modules, each exporting nonempty `caseIds` and `runCase` with that same context. Duplicate IDs fail before execution. Kassa owns `terminal/print-control.cases.mjs`, Coleman owns `terminal/replies-window.cases.mjs`, Sophie owns `terminal/modes.cases.mjs`, and Alaric owns the remaining direct case module and rich trace helpers. Coordinate exact IDs before writing a handler. Lukeni owns only the shared terminal router. The static case registry is generated from exact, fully exported User verification text by `node scripts/acceptance-case-index.mjs`, after refreshing the acceptance metadata snapshot. Explicit Chromium/Firefox/WebKit requirements select the three-engine matrix regardless of name order. Browser VT captures additionally require `render`, `api` and `callbacks`. The CLI compares the entire catalog against `buildCases` from the exported criterion authority before listing or executing, so editing the catalog's verification cannot weaken its own checks. Unresolved attachment references fail generation. No task title or directory implies behavior coverage.

Return a JSON-serializable object:

```
{
  caseId: 'img-01',
  sourceHashes: [{ path: 'source_repo/.../ImageAddon.ts', sha256: '...' }],
  artifactHashes: [{ path: 'target/.../terminal_bg.wasm', sha256: '...' }],
  commands: [{ argv: ['node', '...'], exitCode: 0, log: 'relative/path.log' }],
  captures: [{
    engine: 'chromium', runtimeVersion: 'actual version',
    implementation: 'rust-wasm', assertions: 12,
    reference: { pixels: [], buffer: {}, cursor: {}, replies: [], events: [], properties: {} },
    rust: { pixels: [], buffer: {}, cursor: {}, replies: [], events: [], properties: {} }
  }]
}
```

Native captures use implementation `rust-native`. All required engines must appear exactly once. Image/GPU captures require a real `.wasm` artifact hash and the actual engine version, not a native substitute. `sourceHashes` must include the cited primary source module; the runner rehashes source and artifact files and rejects mismatches, unsafe paths, symlinks and nonregular entries. Command logs and captures are retained in the attempt directory or referenced by a safe repository-relative path.

A capture contains the complete compared domain, not just a success flag. Image cases include pixels, buffer, cursor, replies and events. GPU cases include pixels, state and events. Terminal cases include cells, cursor, margins, normal and alternate buffers, events and exact replies. Move/timer cases include returns, errors, postState and ordered events. Additional properties may capture options, resource identity, lifetimes and callback state. Fields may be empty only when that is the actual source result for the case; tests must still execute nonzero assertions covering the named input/edge scenarios.

Encode non-JSON JavaScript values explicitly and consistently, including undefined, NaN, infinities, negative zero and object identity tokens. JSON preserves lone-surrogate strings without replacement. The worker rejects nonfinite numbers, negative zero, unsupported object types, serialization hooks, hidden/symbol/accessor fields, custom array properties, proxy prototype chains, cycles and reused object references, including returning the same object for reference and Rust captures. Do not silently normalize away a difference. Reference and Rust captures must be independent executions. Do not execute xterm.js as the Rust implementation or return the reference object for both sides. Independent source review remains necessary; metadata alone cannot prove a test is genuine.

A GPU engine skipped by an actual upstream guard may return `{engine, skipped: {reason, source, condition}}`; cite the exact preserved source condition and record its actual evaluated reason. This is a skipped runtime, not a pass. Missing browser installation, missing Rust code or an invented guard is not an upstream skip. Image runtime omissions fail. No case passes when every runtime is skipped or assertions sum to zero.

## Processes, deadlines and evidence

The dispatcher runs each adapter in an owned child process using the existing `quality-process.mjs` process-tree cleanup and disk monitor. It does not introduce a competing process manager or duplicate Cargo target directory. Defaults use the shared 1024 MiB disk reserve. `--timeout-ms` bounds the entire case, including startup and cleanup. Adapter code must close its owned browsers, servers, workers and sockets in `finally`; forced timeout additionally stops the owned process tree.

The worker supplies an AbortSignal for cooperative cleanup and retains stderr/stdout. Successful process exit is insufficient: the parent validates the result schema, hashes, nonzero assertions, runtime set, exact case ID and deep equality of every capture. It writes a unique immutable attempt directory under `results/acceptance-runs/` with raw output, `worker-result.json` and a receipt recording argv, exit/signal/error/timeout, environment, source identity and comparison status. `worker-result.json`, `configuration.json`, `process.log` and `receipt.json` are reserved for the runner; adapters may use `result.json` or other case-specific filenames. Failed attempts are not overwritten by later passes.

Runner self-tests cover unknown/duplicate IDs, invalid flags/deadlines, missing adapters/results/artifacts, zero cases, wrong/duplicate/missing engines, mismatched state, source hash drift, unsafe paths, nonregular files, callback errors and bounded termination. Actual family case execution remains required in addition to these self-tests.
