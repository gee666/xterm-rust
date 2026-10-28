# E2E runtime checkpoint

Overall equivalence is **NOT PROVEN**. Task 271 accepts the execution tools, not the full E01–E05 requirements. Tasks 45, 46 and 50 remain unfinished.

## Integrated tools

The accepted runner changes reached clean `main` at `ef332c6`. The canonical merge gate passed the native workspace check; the separate Node suite passed 14 tests. This does not establish that main has a working browser distribution. Actual browser runs used the larger, isolated `swarm/Sandra` candidate containing the compiled Xavier producer stack and runner fixes, ending at `e2658b8`.

Commands from an isolated checkout, with Git, Rust, the WASM target, matching wasm-bindgen 0.2.100 and installed reference Node dependencies on PATH:

```sh
node --test scripts/e2e/*.test.mjs
node scripts/build-demo.mjs --debug
node scripts/e2e-upstream.mjs --suite addon-fit --no-build
node scripts/demo-e2e.mjs --case demo-05 --no-build
node scripts/demo-e2e.mjs --case demo-06 --no-build
node scripts/check-native-demo.mjs
```

`--no-build` now rejects a stale Cargo input digest, changed WASM/binding bytes or an older build report without provenance. Runtime reports retain actual input/artifact identities and structured command outcomes. A matching prebuilt package is required; the generated package is not checked in.

The fixture compiles real WASM asynchronously and holds a per-navigation readiness image until bindings are published. The earlier synchronous transport failed Chromium's 8 MB compilation limit on the debug distribution. No browser flag, source assertion change or source-engine fallback was used to fix that failure. Missing addon constructors stay absent and are reported; they are not fake implementations.

## Observed Windows results

Node v24.15.0, Rust/Cargo 1.85.1, native Windows x64. Browser versions and page errors are in the retained browser-runtime attachments. These are browser runs on Windows, not native Linux/macOS evidence.

| Command | Discovered | Executed cases | Passed | Failed | Original skips | Unrun |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| original addon-fit against Rust, all three engines | 27 | 3 | 0 | 3 | 0 | 24 |
| demo-05, all three engines | 21 | 21 | 12 | 9 | 0 | 0 |
| corrected demo-06, all three engines | 48 | 48 | 20 | 28 | 0 | 0 |
| bounded addon-fit timeout control | 9 | 0 | 0 | 0 | 0 | 9 |

A failed setup is an attempted case, not evidence that its test body or original assertions ran. Cases suppressed by a failed beforeAll are unrun, not original skips. All four commands returned nonzero as expected for their failures. No assertion was removed to obtain a passing total.

- Fit fails during unchanged source setup at `TestUtils.ts:511`: the public `window.term.unicode` getter is absent. One setup attempt per engine failed, leaving eight other cases unrun in each engine.
- Demo-05 passes live scalar/nested options, actual theme pixel changes, size/font/padding geometry and wrapped Unicode search in each engine. ANSI/HTML serialization and renderer switching fail at absent Serialize/WebGL bindings.
- Demo-06 passes actual attach socket traffic, fit sizing, progress/program-status events, search and web-font behavior in each engine. Clipboard passes Chromium and Firefox. Image, ligatures, serialization and WebGL constructors are absent; the public Unicode getter is absent. Both combinations need serialization and fail at that boundary.
- Corrected link probes now require visible-screen hover readiness before URI activation and fail in all three engines. Earlier probes reached URI activation in Firefox/WebKit without that hover assertion. The cause of the hover failure, and WebKit's absent OSC52 reply, still need source-versus-Rust isolation. These observations are not yet proven implementation defects.
- The font probe initially expected a spaced FontFace family without browser-added quotes. Using an unspaced family removes that browser-normalization mistake without relaxing actual font loading or relayout assertions.
- The native checker passed its current Windows command/parser/state/error/cleanup scope, including native example tests and scoped warning-denied Clippy. It does not complete E01's three-OS, every-byte-partition or complete interaction matrix.

## Retained runs

All paths below are relative to the repository. Summaries, raw Playwright reports/logs, runtime/state JSON and screenshots are retained. Large trace ZIP files remain in the Sandra worktree rather than the Git distribution.

- Matching debug build: `results/demos/build/2026-10-28T16-36-37-221Z/report.json`.
- Fit three-engine failure: `results/e2e/upstream/2026-10-28T16-49-23-299Z-ab385825-5643-48a5-bcac-462e14e39945/`.
- Final demo-05 matrix: `results/demos/browser/2026-10-28T16-48-32-069Z-c0d9bafd-2475-481c-a9bb-2229eaae79d8/`.
- Corrected demo-06 matrix: `results/demos/browser/2026-10-28T16-44-02-220Z-51d17c5a-ffdd-4c15-a0d6-e7dee27b90c3/`.
- Timeout control: `results/e2e/upstream/2026-10-28T16-36-39-054Z-43f18e0d-b4d2-4e81-96b9-732e27bc745d/`.
- Native Windows checker: `results/demos/native/runs/2026-10-28T16-26-55-080Z-df697ff6-3996-4d9e-b9d0-2bbe643129cd/`.

Independent `swarm_review` accepted task 271 after rerunning the 14 Node tests, discovery, real Chromium demos/Fit and a timeout control. It also checked that server listeners were closed. The first review exposed and led to fixes for a rejected-promise test race, stale-build acceptance and discarded child outcomes. The final audit and full browser/source/addon/native platform matrices still remain with their original tasks. Do not report this checkpoint as complete equivalence or complete E2E acceptance.
