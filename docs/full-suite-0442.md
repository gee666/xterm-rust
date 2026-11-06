# Full browser checkpoint on0442

Overall equivalence is NOT PROVEN. Task327 remains open. This checkpoint ran every original browser config and both custom demo suites against real Rust/WASM, then ran fresh original JavaScript unit and browser baselines. It retains failed, skipped and unrun cases rather than reporting a green application.

## Source and build

The isolated checkout started at main `0442e3578a7f57d35c2ae6ff20d7186820983065`. Rust inputs remained1866 files with digest `b9d19e7a3d09fdc5dffa4cc1232b56cf6bf96a905fb111f935452d80decc857f`. The debug package was copied from accepted531, not rebuilt. All75 package-file hashes, all build artifacts and the input digest matched before and after copying. Its actual WASM hash is `0493b554f5782083fac68f383235eb40df84d6f69252666ce699afdb3790a6f6`.

`results/reese327/package-copy.json` records the transfer and exact build report. The existing no-build verifier checked source/WASM/bindings for every Rust command. Fixture hashes identify actual on-disk scripts, including the custom Unicode11 assertion correction later accepted as541. The initializer and all Rust APIs still came from the real compiled Rust package, never the original JS engine.

Observed environment was Windows x64, Node24.15.0, Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Browser versions were measured again after the original baseline using the existing analyzer. These are not native Linux/macOS observations or a performance comparison.

## Commands

The first bounded supervisor ran sequential commands using one allocated browser job, two workers per matrix and zero retries. Every original config kept its assertions and10s hooks. Core had a900s global budget, addon configs600s and each custom demo180s.

```sh
node --test scripts/e2e/build-identity.test.mjs scripts/e2e/build-bootstrap.test.mjs scripts/e2e/fixture-server.test.mjs scripts/e2e/upstream-config.test.mjs scripts/acceptance-runner.test.mjs
node scripts/e2e-upstream.mjs --suite SUITE --no-build --timeout-ms 600000
node scripts/e2e-upstream.mjs --suite core --no-build --timeout-ms 900000
node scripts/demo-e2e.mjs --case demo-05 --no-build --timeout-ms 180000
node scripts/demo-e2e.mjs --case demo-06 --no-build --timeout-ms 180000
node scripts/upstream-baseline.mjs
```

`SUITE` enumerated all13 original addon configs. Ligatures has no original Playwright config, which is an explicit coverage gap rather than a fourteenth addon pass. The original wrapper inventories14 configs total with core. All14 were actually executed against Rust.

The first original browser attempt failed before any case because the new physical reference copy did not yet contain generated `demo/dist/server-bundle.js`. Its failed attempt remains intact. Only the existing original build commands were then run in the owned physical copy, followed by one explicit corrected full baseline:

```sh
node bin/esbuild.mjs --demo-server
node bin/esbuild.mjs --demo-client
node scripts/upstream-browser-baseline.mjs --run --timeout-ms=900000
node scripts/upstream-browser-analyze.mjs results/upstream-browser/runs/2026-11-06T00-57-19-880Z-085e4abf-05e5-4305-8f60-de0b5460177c/summary.json
```

The two build commands run from `source_repo`; the other commands run from the repository root. No source_repo source or assertion changed. The original owned server stopped, its port was released and all776 reference hashes matched afterward. Full commands, finite deadlines, exit codes and failures are retained under `results/reese327/` and the immutable attempts.

## Original browser assertions

These are case/project combinations, not assertion counts, native Rust test functions or mathematical equivalence proof. An attempted setup failure is not a successful test body.

| Config | Rust pass/fail/skip/unrun | Original JS pass/fail/skip/unrun |
| --- | ---: | ---: |
| core |760/22/239/413|1194/0/240/0|
| attach |6/0/0/0|6/0/0/0|
| clipboard |20/0/10/0|20/0/10/0|
| fit |13/6/0/8|17/10/0/0|
| image |596/5/4/34|628/7/4/0|
| program-status |12/0/0/0|12/0/0/0|
| progress |30/0/0/0|30/0/0/0|
| search |81/0/0/0|81/0/0/0|
| serialize |120/0/0/0|120/0/0/0|
| unicode-graphemes |3/0/0/0|3/0/0/0|
| unicode11 |3/0/0/0|3/0/0/0|
| web-fonts |15/0/0/0|15/0/0/0|
| web-links |1503/0/0/0|1503/0/0/0|
| webgl |96/80/37/0|121/55/37/0|
| Total |3258/113/290/455|3753/72/291/0|

Both matrices discovered4116 cases. Rust attempted3371; original JS attempted3825. The72 source failures remain failures, not skips or expected successes.

`results/reese327/aggregate.mjs` recomputes the first Rust attempt from raw reports with the existing `testCounts`. Its start/finish bounds prevent later reruns from silently replacing or double-counting those cases. `browser-summary.json` pins the raw/summary reports and preserves every failed record.

`compare-baseline.mjs` maps all4116 named case/project combinations one-to-one to the fresh source baseline. Only the generated top-level `.test.js` filename is mapped back to `.test.ts`. No expected value or result is normalized. `source-comparison.json` records3254 passed in both,52 source-passed/Rust-failed,447 source-passed/Rust-unrun,290 skipped in both, one source-skipped/Rust-unrun,61 failed in both, seven source-failed/Rust-unrun and four source-failed/Rust-passed. This is outcome mapping, not complete assertion or output equivalence.

## Concrete findings

- Six unchanged Firefox core beforeAll hooks hit10s. Their fallout accounts for413 core cases unrun. Original JS core completed1194 passes and240 skips with no unrun cases under the same worker count and hook deadlines. Rust image and Fit setup failures add34 and8 unrun cases. These are initialization gaps, not external OS blockers.
- The remaining16 executed core failures reach the absent public registerDecoration API. Task537 owns its actual integration. The historical missing registerLinkProvider diagnosis is stale on this snapshot.
- Image's private-accessor test reaches missing `_core._inputHandler` in Chromium and WebKit. The Chromium palette.png/qoi20x5 resizing assertions also fail in the fresh original baseline, with the same165-versus160 values. They must not be mislabeled as new Rust resizing defects or fixed by changing the original assertions.
- WebglAddon is still absent. The unchanged original WebGL suite catches constructor errors and can continue on DOM, so its96 Rust passes are not96 GPU passes. Task306 retains actual renderer/addon integration.
- The web-links regex/provider suite passes1503 cases, but live custom demo hover still fails. Regex success is not real mouse-hover acceptance.

## Demo and unit baseline

Demo05 discovered/executed21, passed17 and failed4. Three failures reach WebGL renderer switching; one WebKit HTML clipboard read is denied. Demo06 discovered48, attempted44, passed39, failed5 and left4 unrun at its180s global bound. The executed failures are links, missing WebGL and WebKit clipboard behavior. The four unrun cases remain named in the raw report.

All six Unicode11/Graphemes demo lifecycle cases completed and passed. Task541 corrects the custom Unicode11 disposal expectation against actual original lifecycle/width behavior. Its original-source negative control, independent review and limitations are in `docs/unicode-demo-disposal.md`. That acceptance does not close either demo suite.

The fresh unchanged original unit baseline discovered2460, executed/passed2389 and retained71 original pending cases, with zero failures, added skips or unrun cases. It inventoried73 modules,71 test-bearing and two support-only TestUtils modules. This is JS reference execution, not2389 mapped Rust implementations.

The E2E/acceptance runner tests passed34/34 before the transport repair. No native interactive, native OS, Miri, release, full assertion-audit or speed/size comparison matrix was rerun in this browser allocation. Accepted531's native quality evidence remains separate and source-bound.

## Next work

Task548 replaces the old21.7MB base64 bootstrap and its per-byte JS traversal with an asynchronous request for the same verified WASM. Its36 unit order/failure controls pass. A separately allocated full core/image/Fit rerun then completed with zero startup failures, server errors or unrun cases, without changing the original2-worker/10s-hook/zero-retry settings. Core remains1167 pass/27 fail/240 skip, image628 pass/7 fail/4 skip and Fit17 pass/10 fail.

The derived latest complete-config ledger combines these three unfiltered reruns with the eleven other first-run configs on identical Rust inputs:4116 discovered,3825 attempted,3701 passed,124 failed,291 original skips and zero unrun. It is not one fresh14-config run under the new fixture. `results/reese327/transport-proof.json`, `transport-updated-summary.json` and `source-comparison-updated.json` retain exact scope and raw report pins. The initial failed matrix above remains intact. See `docs/e2e-wasm-fetch.md`.

Full327/330, application gaps, native platform checks, final mapping and comparisons remain open. This checkpoint cannot be accepted as full equivalence.
