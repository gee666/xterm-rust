# Original xterm.js unit baseline

The supplied reference identifies itself as xterm.js 6.0.0. No public commit is asserted. The original 776-file hash inventory is retained at `docs/baseline/reference-manifest.json`, copied from the pre-launch inventory at `tmp/source-manifest.json`. Installation and generated outputs are excluded from that original inventory.

Run from the repository root with Node.js 24.15.0 and npm available:

```sh
npm --prefix source_repo ci --ignore-scripts --no-audit --no-fund
node --test scripts/upstream-baseline.test.mjs
node scripts/upstream-baseline.mjs
```

The lockfile is unchanged. Installation skips package lifecycle scripts because the unit and esbuild checks do not require the native PTY. A working original interactive demo may require separately rebuilding node-pty. This command does not claim demo readiness.

The runner first checks all 776 original file sizes and SHA-256 values, builds the core and all 14 addons using `bin/esbuild_all.mjs`, then calls the original `bin/test_unit.js` with Mocha's JSON reporter. It checks every original hash again afterward. Child processes have explicit timeouts with process-tree termination through `scripts/quality-process.mjs`. A free-space monitor aborts below 256 MiB rather than exhausting the shared drive. Missing dependencies, source differences, build/test errors, missing generated test modules, inconsistent result counts and missing source-callsite metadata fail the run.

## Observed result

The retained Windows x64 run with Node v24.15.0 discovered 2,460 unit cases across 422 Mocha suites. Of those, 2,389 passed, 71 were pending in the supplied source and none failed. No original assertions were changed or disabled. These are original JavaScript results, not Rust migration counts or browser end-to-end evidence.

There are 73 original source `.test.ts` modules under core/addon `src`. Seventy-one define test cases. The remaining `src/common/TestUtils.test.ts` and `src/browser/TestUtils.test.ts` export helpers without registering cases.

## Evidence and counters

Every attempt gets a new `results/upstream-baseline/runs/<id>/` directory. `results/upstream-baseline/latest.json` points to its summary and is marked `incomplete` before dependency checks or builds. Failed and interrupted attempts cannot reuse an older passed summary. Finished attempt files are not overwritten by future runs. Early flat files from the first observed run are retained as historical evidence, not as the current result.

Read the pointer, require its status to be `passed`, then read its referenced summary. Artifact paths inside that summary resolve from the repository root. The following table describes the files within that attempt directory.

| Artifact | Purpose |
| --- | --- |
| `summary.json` | Commands, Node/Mocha/esbuild versions, platform, reference hashes, per-source counts and every pending title |
| `original-unit.json` | Complete original Mocha case records and original reporter metadata |
| `unit-registration.json` | Actual registration-callsite provenance for all cases |
| `original-unit.log` | Escape replies and diagnostics printed by original tests, separate from JSON |
| `esbuild.log` | Original build output |
| `results/upstream-baseline/npm-ci.log` | Initial dependency installation output |
| `runner-tests.log` | Integrity/counting guard tests, including negative controls |

`summary.json.suites` contains one record per original source module with `source`, `generated`, `discovered`, `executed`, `passed`, `failed`, `pending`, `skipped` and `unrun` fields. `pending` is the original Mocha pending count, including disabled source cases. Those cases are never counted as executed or passed. `skipped` is reserved for additional exclusions by this wrapper, of which there are none. `unrun` is zero after a successful complete original run; failed setup or discovery returns failure rather than claiming zero missing cases.

Mocha's usual `file` field labels tests from an imported test module with the importer's filename. The ten WidthCache cases are imported by DomRendererRowFactory. `upstream-registration.cjs` observes the actual test registration stack without changing original test bodies, order or reporter fields. The per-source summary uses that callsite provenance, while the raw reporter output remains intact. This prevents incorrectly classifying WidthCache as a module with no tests.

The small runner unit suite mutates only temporary files under repository-root `tmp/`. It checks same-length corruption, growth, missing files, path escape rejection, count inconsistency and isolation of successful, failed and interrupted attempts. It never mutates the actual reference to test its integrity guard.

## Remaining verification

Original browser suites still need their own project/case discovery and runtime reports. Original unit success alone establishes no Rust terminal state, browser rendering, addon integration, or native Linux/macOS runtime equivalence. The required directed, malformed/chunked, randomized, addon and browser acceptance matrix is in `docs/acceptance/upstream-baseline.md`. Overall equivalence remains NOT PROVEN.
