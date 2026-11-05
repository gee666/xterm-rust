# Acceptance plan for the 16-agent continuation

Overall equivalence remains NOT PROVEN. This document replaces historical task ownership in `docs/acceptance-plan.md`, not its detailed source-derived behavior checklists.

## Authority and complete inventory

Read `PORT_REQUIREMENTS.md`, `AGENTS.md`, `docs/handoffs/new-swarm-16.md` and its repository snapshot. The current request uses exactly 16 agents. Actual integration is `main` in `C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Integration`. Legacy launch master and retained branches are references, not the merge target.

The supplied xterm.js 6.0.0 common, headless, browser, typings, CSS, demo, benchmarks, fixtures and all fourteen addons remain in scope. Use the existing `results/compatibility/{modules,apis,suites,fixtures,addons,runtime-contracts}.json` and `tools/compatibility/behavior-criteria.json` to enumerate exact contracts. Historical counts are inventory observations, never proof of executed assertions. Preserve all existing checklists under `docs/acceptance/`, addon maps and source hashes. Closure work must reconcile every discovered row, not sample a working subset.

## Testable behavior families

Each criterion requires source behavior through the actual integrated Rust implementation. The linked closure task covers the entire family, reusing current scoped producer fixes rather than duplicating their edits. A missing implementation, assertion, browser binding or error path remains unfinished work, not an external blocker.

| Family | Criterion | Closure task |
| --- | --- | --- |
| Common and lifecycle | 463 | 312 |
| Parser and terminal | 465 | 313 |
| Buffers and cells | 466 | 314 |
| Public APIs and options | 467 | 315 |
| Unicode | 468 | 316 |
| Browser root and layout | 469 | 317 |
| DOM rendering and themes | 470 | 318 |
| Input and accessibility | 471 | 319 |
| Scrolling and pointer | 472 | 320 |
| Selection and links | 473 | 321 |
| Search and serialization | 474 | 322 |
| Utility and status addons | 475 | 323 |
| Image addon | 476 | 324 |
| GPU and font addons | 477 | 325 |

For every family, include source defaults and valid, empty, malformed, boundary and repeated inputs. Preserve UTF-16 units and lone surrogates, streaming byte boundaries, numeric coercion/nonfinite values, raw object/return/error identity, getter order, thrown errors, callback mutation/reentry, cancellation, async completion and disposal where source supports them. Run platform-independent core tests natively and browser-only contracts through real WASM/DOM services. Document deliberate safe divergences from source hangs rather than claiming equivalence.

All fourteen addons are explicitly assigned. Search and serialize have criterion474; attach, clipboard, Fit, web-links, program-status and progress have475; image has476; WebGL, ligatures and web-fonts have477; Unicode11 and Unicode-graphemes have468. Shared renderer/input/queue dependencies require integrated tests, not disconnected helper acceptance.

## Proof and delivery gates

| Requirement | Criterion | Work |
| --- | --- | --- |
| User intent and exhaustive acceptance coverage | 445 | 287 accepted intent; 288 this plan |
| Exactly16 complementary lanes and retained-work recovery | 446 | Subsystem claims and309 integration |
| Every source/API/addon behavior and language representation | 447 | All family closure tasks and328 final reconciliation |
| fmt, warning-denied Clippy, tests, doctests, release, features, WASM, unsafe checks | 448 | 302 prerequisite repairs and304 final stable-revision matrix |
| Unchanged original assertions and full-state differentials | 449 | Scoped producer tests,327 full suites,328 accounting |
| Real native and Rust browser demonstrations/E2E | 450 | 327 and330 |
| Observed native Windows/Linux/macOS MSRV and stable runtime evidence | 451 | 302 historical diagnosis;304 actual final matrix |
| Matched speed, distribution size and classified LoC | 452 | 303 size/LoC;329 validated release timing |
| O: resource limits and complete owned temporary cleanup | 453 | 309 and every owner's cleanup record |
| Copyable README and final compatibility/verification/comparison truth | 454 | 328 and330 |

- Task327: Run complete unchanged browser suites and demo E2E.
- Task328: Complete final source API and assertion coverage audit.
- Task329: Measure matched release JS Rust performance.
- Task330: Verify clean setup and interactive demonstrations.

Verification must retain discovered, mapped, executed, passed, failed, skipped and unrun counts separately. Preserve deterministic/random seeds and failing cases. Compare full cell/codepoint/width/attribute state, both buffers, cursor/modes/scrollback, events, replies, APIs and lifecycle. Run original applicable baselines and Rust assertions unchanged, including Chromium, Firefox and WebKit and complete E01-E05/demo addon interactions. A passing helper, build, cross-compile or workflow file is not integrated runtime evidence.

Performance uses identical validated bytes, dimensions, scrollback, Unicode configuration and operations. Use production JS and release Rust, warmup and at least ten measured repetitions, median/p95, raw measurements, units, environment and timing/startup policy. Keep native parsing distinct from browser rendering. Size includes matched core/addon native artifacts and every mandatory WASM/JS/CSS asset, raw/gzip/Brotli with fixed settings. LoC uses one method with production/addon/glue/generated/test/demo/tool categories. Unmeasured results are missing, not zero.

Review must inspect actual code and independently execute relevant checks. Safe Rust and explicit unsafe invariants, ownership/RAII, compatible dependency licenses, attribution, warning-free builds, focused functions and the 350-line rule remain mandatory. No dead code, successful stubs, weakened assertions, JS terminal-engine delegation or blanket lint suppression. Record unsupported Miri/sanitizer checks honestly.

## Current complementary lanes

| Owner | Lane and current task |
| --- | --- |
| Malcolm | Integration/resources and acceptance,309/288 |
| Champa | Native parser/queue/full traces,294 |
| Ricardo | Buffers/markers/decorations,296 |
| Njoya | Browser root/write scheduling,297 |
| Liliana | DOM renderer/selection/live links,300 |
| Sei | Overview ruler component and Terminal lifecycle,308 |
| Margaret | Scroll/viewport recovery,289 then301 raw model |
| Diponegoro | Input/focus/IME/accessibility,292 |
| Wyatt | Mouse raw policies/registry,291 then305 |
| Nolan | Image protocol/browser decode,295 then326 coordinator |
| Lane | WebGL,299 then306 |
| Clifford | Ligatures/web-fonts/joiners,293 then307 |
| Clarence | Search/serialize investigation,290 |
| Yoshitsune | Utility/status addons and Fit,298 |
| Zhao | Workspace quality and native matrix,302 then304 |
| Genevieve | Intent/coverage/comparison,287/303 |

This is ownership coordination, not an assertion of completion. Full family closure tasks can be claimed after focused repairs. Follow the live task records when scope or ownership changes. Agree shared-file hunks before editing; only final tested ref integration is serialized.

## Resource and cleanup contract

New worktrees, builds and scratch use `O:/xterm-rust-swarm-16/tmp/`. Each owner records their paths/processes, sets TEMP/TMP under owned scratch, uses Cargo -j2 and requests/releases one of four heavy jobs. At most two browser matrices run within those four. Thread377 records live allocations and measurements without reviving the retired global queue. Pause new heavy jobs below8GiB RAM or10GiB free temporary disk; measure before changing limits.

Use the existing canonical merge helper from a clean committed producer branch. It rebases, tests changed crates/affected dependents/workspace/WASM and fast-forwards only the tested unchanged main ref. Do not waive failures. Recover retained work before rewriting.

After preserving reviewed source, regression fixtures and concise durable `results/` evidence, stop owned processes and remove owned temporary files, targets and worktrees. Inspect dirty/untracked contents and reparse points first; unlink only owned dependency junctions before any recursive removal. Preserve old worktrees/caches, references, credentials, unrelated work and private runtime data. No AGENTS changes, blanket clean, forced main or unknown-PID termination.

Finish only when every active criterion is satisfied or genuinely blocked, tasks accepted or correctly deferred and no reviews remain. Missing code or internal dependencies are work, not blockers. Reports retain NOT PROVEN until the full contract has actual evidence.
