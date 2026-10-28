# Original browser baseline

This is execution of the unchanged supplied JavaScript reference, not the Rust port. It establishes observed reference behavior and failures on Windows x64. It does not establish Rust equivalence, native Linux/macOS evidence, or a green original browser suite.

## Reproduce

After the original dependency installation and esbuild steps in [upstream-baseline.md](upstream-baseline.md):

```sh
node source_repo/node_modules/@playwright/test/cli.js install chromium firefox webkit
cd source_repo
node bin/esbuild.mjs --demo-client
node bin/esbuild.mjs --demo-server
cd ..
node scripts/upstream-browser-baseline.mjs
node scripts/upstream-browser-baseline.mjs --run --timeout-ms=300000
node scripts/upstream-browser-analyze.mjs
```

The first baseline command lists all original cases without starting a browser or a server. The `--run` command executes all suites and returns nonzero if an original assertion, setup, teardown or process fails. The analysis command verifies source accounting. Its success means accounting succeeded, not that the original tests passed. `originalProcessStatus` and `input.status` retain the failed baseline state.

Use `--suite=addon-web-fonts` or `--project=Chromium` for an explicitly scoped run. Omitting filters executes all original projects. Project names are `Chromium`, `FirefoxStable` and `WebKit`. A filtered run is never reported as a full matrix. There are no added assertion exclusions or retries.

## Server ownership and original-runner issue

The supplied Playwright `webServer` command launches npm. On this Windows run, its tests finished but plugin teardown hung and left a listening server behind. A direct-node command under the Playwright plugin also hung. Both failed attempts and explicit cleanup logs are retained. Neither is reported as a successful baseline.

The working harness starts `node demo/start.js` through the independently reviewed `startManagedProcess` helper. It refuses an already occupied port, waits for a bounded HTTP readiness check, monitors free space, and owns an explicit process-tree shutdown in `finally`. A temporary config under repository-root `tmp/` imports the original generated config and retains its projects, tests, timeouts and assertions. It resolves the original test directory and supplies the same `http://localhost:3000` base URL normally inferred by Playwright from `webServer.port`. Only server ownership moves out of the broken plugin. `--original-server` can reproduce the original plugin behavior.

Every attempt starts with an incomplete receipt and unique directory under `results/upstream-browser/runs/<id>/`. Failed and interrupted attempts cannot reuse previous success, and finished attempts are not overwritten. `results/upstream-browser/latest.json` points to the latest attempt. Runtime test output, JSON records and failure artifacts remain in that attempt directory. Source hashes are checked before and after.

## Observed complete matrix

The complete retained runtime attempt is:

`results/upstream-browser/runs/2026-10-27T23-59-39-134Z-65e3d213-db85-489a-811d-1d08350d2e8b/summary.json`

It discovered and accounted for all 4,116 case-project combinations across 14 configs and three projects. There were 3,824 executed cases, 3,752 passed cases, 72 failed cases, 292 upstream skips and zero unrun cases. All 776 reference hashes remained unchanged. The owned server stopped, no port 3000 listener remained, and no deadline/resource termination occurred. The original baseline result is FAILED.

Each table entry is passed/failed/skipped. No cases were unrun in this attempt.

| Original suite | Chromium | Firefox | WebKit |
| --- | ---: | ---: | ---: |
| core | 419/0/59 | 418/0/60 | 357/0/121 |
| attach | 2/0/0 | 2/0/0 | 2/0/0 |
| clipboard | 10/0/0 | 10/0/0 | 0/0/10 |
| fit | 4/5/0 | 4/5/0 | 9/0/0 |
| image | 210/2/1 | 209/2/2 | 208/4/1 |
| program-status | 4/0/0 | 4/0/0 | 4/0/0 |
| progress | 10/0/0 | 10/0/0 | 10/0/0 |
| search | 27/0/0 | 27/0/0 | 27/0/0 |
| serialize | 40/0/0 | 40/0/0 | 40/0/0 |
| unicode-graphemes | 1/0/0 | 1/0/0 | 1/0/0 |
| unicode11 | 1/0/0 | 1/0/0 | 1/0/0 |
| web-fonts | 5/0/0 | 5/0/0 | 5/0/0 |
| web-links | 501/0/0 | 501/0/0 | 501/0/0 |
| webgl | 62/0/9 | 57/0/14 | 2/54/15 |

The supplied integration wrapper has no ligatures config. Its original unit suites are included in the unit baseline. Absence of a browser config does not prove browser ligatures integration.

The actual installed binaries were launched after the baseline to record versions: Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. These measurements and executable paths are in the derived accounting receipt. Browser installation uses the normal user cache on C:, which had sufficient space, rather than duplicating caches on the constrained repository drive.

## Failed assertions and remaining investigation

- Fit has ten failures in Chromium/Firefox involving the original column-width upper bounds. WebKit passed all nine Fit cases. Independent font diagnostics in `results/utility-addons/font-diagnostics.json` observed default monospace widths of 8.25px in Chromium/Firefox and 9px in WebKit, producing 95/120 versus87/110 columns. Do not widen or remove the original assertions; controlled-font diagnostics remain distinct from the unchanged default baseline.
- Image has two failures each in Chromium/Firefox involving 165 versus 160 pixel image width, and four WebKit failures involving SIXEL cursor, image lifecycle/storage and resizing. All original error records remain retained.
- WebKit WebGL has 54 failures involving timeouts or screenshot pixels such as constant `[13,13,13,255]` instead of expected colors. Static inspection found an API/setup mismatch: `WebglRenderer.test.ts` constructs `new WebglAddon(true)`, but the current constructor and typings accept an options object and read `options?.preserveDrawingBuffer`, which is undefined for a boolean. The shared tests sample a cached screenshot, not direct gl.readPixels. Whether this mismatch or screenshot timing causes the failures has not been demonstrated. Do not invent an unsupported-GPU classification, transform the reference inputs to get green results, or erase the unchanged failed attempt.

The individual failing titles, source files, project names, expected statuses and assertion messages are in the derived report's `failures` array and original raw Playwright reports. Subsystem owners were notified. These are observed original-reference failures, not proof that the Rust implementation fails or passes the corresponding contracts.

## Exact source accounting

Read `results/upstream-browser-analysis/latest.json` and follow its summary path. The retained complete matrix analysis is:

`results/upstream-browser-analysis/runs/2026-10-28T00-13-23-567Z-e0d41f6b-083e-4a9d-abfe-1426b9e96c9e/summary.json`

Each `suites` row has an exact original `source` path, source `sha256`, original config `suite`, `project`, and discovered/executed/passed/failed/skipped/unrun counts. SharedRendererTests is counted separately for each importing config because those are distinct real executions. Analysis validates its counts against every runtime project summary and retains a SHA-256 link to the immutable input receipt. It never rewrites completed runtime attempts or upgrades original failures to passes.

Original Playwright reports identify source files through source maps. Relative source paths are resolved against the original compiled test directory and must exist within `source_repo`. Unresolved provenance or counter disagreement fails analysis. All test results and screenshots/error contexts remain available for independent review.

Full terminal equivalence remains NOT PROVEN. Rust-backed browser assertions, full-state Rust/JS traces, native platform execution and all other acceptance gates remain separate requirements.
