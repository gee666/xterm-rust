# Full-suite checkpoint

Overall equivalence is NOT PROVEN. Task327 remains open. Task376 retains this failed checkpoint and repairs one incorrect custom demo assertion. It does not accept the full application or its verification.

## User intent and scope

Port the complete supplied xterm.js behavior to Rust, including all14 addons, native Windows/Linux/macOS, Chromium/Firefox/WebKit, interactive examples, exact source/assertion mapping and measured comparisons. Exactly16 complementary agents share implementation, testing and review. Integration is main, not launch master. Preserve source_repo assertions and AGENTS.md. Use bounded -j2 builds on O: and remove owned temporary outputs after preserving evidence.

These fresh runs used frozen main `e1651049cf208ae47965b9c9f7e206fecf36cdd6` in isolated `swarm/B`. Later producer fixes are not part of these results. The package report pins Rust-input digest `3661cb6cf07784549c345c0ede25c9e22f81e26a1f5ba367abbb8934bf641653`, WASM and bindings. Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 ran on Windows x64. Browser runtime attachments identify the actual engines. This is not a final native-platform checkpoint or complete faithful native assertion mapping.

## Commands and fresh native results

From the owned O: checkout, with existing tools and reference dependencies available:

```sh
node scripts/build-demo.mjs --debug
node scripts/upstream-baseline.mjs
cargo test --locked --workspace --all-targets --all-features -j2
node scripts/check-native-demo.mjs
node --test scripts/e2e/*.test.mjs scripts/acceptance-runner.test.mjs
node scripts/e2e-upstream.mjs --suite SUITE --list
node scripts/e2e-upstream.mjs --suite SUITE --no-build --timeout-ms 600000
node scripts/demo-e2e.mjs --case demo-05 --no-build --timeout-ms 180000
node scripts/demo-e2e.mjs --case demo-06 --no-build --timeout-ms 180000
```

`SUITE` enumerates core and all13 original addon configs below. There is no original ligatures browser config. Its absence does not establish ligature browser equivalence. The core attempt used900000ms. Small addon attempts used240000ms. Exact argv, timeouts, process status, source/build/fixture identities, requests and original reports are retained under the paths in [summary.json](../results/full-suite-checkpoint/summary.json).

- The unchanged original unit runner passed2389 cases, with71 original pending, zero failures/skips/unrun, and2460 discovered. All73 test modules were inventoried,71 were test-bearing. This includes original common/headless/browser/addon unit tests, not Rust assertion coverage. The fresh receipt is `results/upstream-baseline/runs/2026-11-05T11-48-38-821Z-23684b84-bc19-4beb-af1d-cc4c000ca298/summary.json`.
- The Rust all-target/all-feature workspace run passed2037 test functions across394 summaries. It is not a one-to-one mapping to those2389 original unit cases. The complete log is `results/full-suite-checkpoint/workspace-all-features.log`.
- Native interactive checker passed all planned checks, state/attribute assertions, six intentional failures, recovery after an invalid command, EOF and disposal. Receipt `results/demos/native/runs/2026-11-05T11-49-57-957Z-f2dc6643-0d88-4bcd-b233-a7ff1b602605/summary.json`. An earlier attempt was correctly rejected because CARGO_TARGET_DIR was set. The retry unset that override and used the same regular local target. Both attempts remain retained.
- Existing E2E/acceptance runner tests passed34/34 before and after the custom assertion edit. These include beforeAll fallout accounting and stale-build checks, not all E05 crash/clipboard/socket failure scenarios.
- Fresh reference verification matched all776 manifest files, with zero mismatches. The full old evidence index was not current after adding new results, so its `--check` was not a passing final evidence-index gate.

## Unchanged original assertions against Rust

All14 configs were discovered, totaling4116 case/project combinations. No source test or assertion was changed. Each row covers all three original projects. Non-skipped execution counts include setup failures and interruptions. They do not imply every test body or assertion completed.

| Config | Discovered | Executed | Passed | Failed | Original skips | Unrun |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| core, including exact omitted-case replay |1434|1194|821|373|240|0|
| attach |6|6|6|0|0|0|
| clipboard |30|20|20|0|10|0|
| fit |27|27|17|10|0|0|
| image |639|438|0|438|3|198|
| program-status |12|12|12|0|0|0|
| progress |30|30|30|0|0|0|
| search |81|81|81|0|0|0|
| serialize |120|120|120|0|0|0|
| unicode-graphemes |3|3|3|0|0|0|
| unicode11 |3|3|3|0|0|0|
| web-fonts |15|15|15|0|0|0|
| web-links |1503|1503|1503|0|0|0|
| webgl |213|176|96|80|37|0|
| Total |4116|3628|2727|901|290|198|

The first900s core attempt had790 pass,371 fail,210 skip and63 unrun. A broad Firefox replay also timed out and is retained as another failed attempt, not added to these unique totals. The exact63 previously unrun cases were then selected without changing assertions. They produced31 pass,2 fail and30 original skips, zero unrun. `results/full-suite-checkpoint/core-unrun-selection.json` lists that selection; `replay-unrun.mjs` reproduces its command from the repository root. The old interrupted failures remain failures in the totals. A future clean run may supersede them, but this checkpoint does not erase them.

The198 Image cases remain unrun after setup failures. Repeating absent-constructor setup cannot establish their assertions. They need the full live ImageAddon binding, task311, followed by reruns. Actual IIP/SIXEL component work since this snapshot does not replace this missing full-addon matrix.

The96 WebGL reported passes are NOT96 GPU tests. The unchanged `addons/addon-webgl/test/WebglRenderer.test.ts` catches constructor errors in beforeAll and runs shared renderer assertions anyway. With WebglAddon missing, those can exercise DOM. Missing constructor and pixel/decorations failures remain explicit task306 work.

## Original JavaScript comparison

The retained unchanged full JS browser baseline is `results/upstream-browser/runs/2026-10-27T23-59-39-134Z-65e3d213-db85-489a-811d-1d08350d2e8b/summary.json`. Recomputing its raw reports with current `testCounts` accounts for the same4116 combinations,3824 attempted,3752 passed,72 failed,291 original skips and one timeout-unrun. The historical summary incorrectly labels that last case as an original skip. It is WebKit's `standalone tests (Shadow dom) / regression tests / #4790: cursor should not be displayed before focusing` in `addon-webgl-all-runtime.json`, expected passed with no skip annotations and a zero-duration skipped result after suite/teardown timeouts. [Corrected accounting](../results/full-suite-checkpoint/historical-baseline-accounting.json) preserves the raw historical evidence unchanged. This is historical execution, not a fresh rerun or Rust proof. Its source hashes remain compatible with this reference. It already has ten Fit failures, image geometry failures and WebKit GPU failures. Compare exact original records before calling every Rust failure a new regression. No original bound was widened to clear Fit.

## Demo results and precise HTML assertion repair

The original custom demo05 attempt passed15/21 and failed6. Three failures were caused by our assertion that serialized HTML contain the decimal substring `255`. The unchanged source `SerializeAddon.ts::_getHexColor` emits hex CSS. Correct red HTML uses `#ff0000`, so that assertion rejects the source format.

The repaired custom test parses the actual HTML, finds the span containing `HTML 界😀`, and requires its canonical CSS color to be exactly `rgb(255, 0, 0)`. Missing text/span, black or other colors still fail. No source_repo assertion changed. All three engines reached and passed the new color check. The full rerun passed17/21 and failed4. Three failures remain the missing WebGL renderer; another reaches an actual clipboard-read permission denial after the successful color check. We did not suppress that denial or report a green demo.

Demo06 passed35/48 and failed13. Missing ImageAddon/WebglAddon exports account for six failures; link hover for three; Unicode11 restoration for three; Firefox OSC52 query output for one. The Unicode11 addon source has an empty dispose method, unlike Unicode-graphemes which restores its old version. Its custom demo expectation needs an explicit source/demo-policy review rather than silently changing the core provider. Task330 retains this work.

## Failure routing and retained artifacts

[Failure routing](../results/full-suite-checkpoint/failure-routing.json) names every failing title, file, project, summary and message. Unmodified raw Playwright JSON and logs preserve full errors. Requests, missing-binding lists, failure screenshots and error contexts are retained.23 duplicate trace ZIP resource archives totaling about79MiB were removed after hashing. [omitted-traces.json](../results/full-suite-checkpoint/omitted-traces.json) explicitly records them as not retained. No trace archive is claimed as deliverable evidence.

- Task305 receives312 core attempts blocked by missing live `_core.mouseStateService.activeProtocol/activeEncoding`. Leif's task375 owns the concrete live mode binding follow-up.
- Task331 receives missing public registerDecoration and live decoration lifecycle failures.
- Core DECSCUSR also lacks core.coreService.decPrivateModes; DECSTBM lacks `_core._bufferService.buffer`. They remain named binding gaps, not fake fixture services.
- Tasks306 and311 retain actual WebGL/Image activation, pixels, exports and original matrices.
- Task330 receives demo WebGL switching, clipboard host behavior, Unicode11 policy and hover failures. Full original web-links regex cases passing does not establish live hover.
- Full native family mapping312-316, E01 native platforms, E02/E03 interactions, complete addon combinations, all E05 failure controls, final snapshot reruns and task328's complete assertion audit remain required.

Posts463,468 and469 share these failures with owners. Overall equivalence stays NOT PROVEN. Task327 must not be accepted merely because this checkpoint is independently reviewed.

## Temporary ownership

Only O:/xterm-rust-swarm-16/tmp/B, its regular target/tmp outputs and two explicitly owned dependency junctions were created. The junctions are node_modules and source_repo/node_modules pointing to the existing P: dependency installation. They must be unlinked before recursive cleanup. Build artifacts are pinned by hashes rather than committed. Cleanup and review receipts will be recorded after verification; no other agent's files, caches or processes may be removed.
