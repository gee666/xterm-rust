---
name: upstream-baseline
description: Use when executing the unchanged original xterm.js unit suites, checking reference hashes, or mapping Mocha results to actual source modules.
---

# Original unit baseline

From the repository root:

```sh
npm --prefix source_repo ci --ignore-scripts --no-audit --no-fund
node scripts/upstream-baseline.mjs
```

Node 24.15.0 was used for the retained Windows x64 evidence. Installation skips package lifecycle scripts because these unit/bundle checks do not need node-pty. Browser demos may still require native PTY installation.

The runner verifies all 776 original files against `docs/baseline/reference-manifest.json` before and after the run, builds core and all 14 addons using the original esbuild script, and calls the original unit runner. Each child has a four-minute timeout. Logs and raw JSON are retained under `results/upstream-baseline/`.

Each attempt is retained under `results/upstream-baseline/runs/<id>/`. Read `latest.json`, require `status: passed`, and follow its `summary` path. The pointer is marked incomplete before setup so interrupted or failed runs cannot reuse an older success. Each run's `summary.json` has exact per-source counters. `unit-registration.json` supplements Mocha file metadata because imported WidthCache tests otherwise appear under DomRendererRowFactory. The hook observes test registration stack frames without changing source, assertion bodies, test order or Mocha's own file fields. Its output path comes from the runner's per-attempt environment, never a shared fixed filename.

For the supplied source, expect 2,460 discovered, 2,389 executed/passed, 71 upstream pending and no failures. The wrapper's skipped count is zero; original pending cases remain separate and are never counted as passed. Two `.test.ts` files provide helpers but register no cases. Never equate original JavaScript execution with Rust assertion migration.

For original browser discovery/runtime, use `node scripts/upstream-browser-baseline.mjs` and `node scripts/upstream-browser-baseline.mjs --run --timeout-ms=300000`, then `node scripts/upstream-browser-analyze.mjs`. Read `docs/upstream-browser-baseline.md` first for demo build prerequisites, exact failure counts, owned server handling and immutable runtime/derived receipt paths. Analysis success does not turn an original failed baseline into a pass.

See `docs/acceptance/upstream-baseline.md`. Preserve pending records, source hashes and failures. Do not use stale JSON after a failed build or runner exit, edit source assertions, or overwrite expected data to hide mismatches.
