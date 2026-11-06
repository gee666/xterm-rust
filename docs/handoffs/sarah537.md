# Sarah537 handoff

Task 537 is independently accepted at `b31bc29f` on `swarm/SarahDecoration`, based on main `0442e357`. Full task 331 and complete Rust xterm.js/all-14-addon equivalence remain NOT PROVEN.

## Pickup

The reviewed own chain is `dff0f88e` -> `5c01866a` -> `69a40a38` -> `b31bc29f`. There are no private prerequisites or copies of other producers. Review, handoff and cleanup-only tails are reported in post 409 and preserved in `archive/Sarah537-complete`. Use that final report for the exact complete chain. Preserve the original commits rather than replaying the pre-rebase `d2564b1b`/`2d1babc8` attempts.

This implements the missing exported `Terminal.registerDecoration` for real exported markers, including markers from another Terminal in the same WASM instance. It uses the real Rust decoration service, cache, DOM overlay, row colors and overview canvas. The JavaScript helper performs host operators and maintains GC ownership records, not the terminal engine. Source validation requires no proposed-API gate.

The native host-properties path reads individual live `OptionValue` fields without retaining an options borrow. Height conversion repeats at each source loop comparison. Cell iteration now yields `Result<Decoration, DecorationError>`, callback traversal returns `Result`, and row errors preserve the host exception object. Native typed `register_decoration` still returns an optional decoration; the host path returns a decoration after its own earlier disposed-marker guard. Host capabilities and cleanup are installed before cache getters, including registration failure paths.

Read `docs/public-decoration.md`, especially its remaining raw-object contracts. Do not describe this as full marker/decorations or GC parity. Task 331 now depends on accepted 537 and retains writable arrays/marker fields, structural/cross-WASM marker objects, raw writable decoration fields, overview getter ordering/unknown positions, arbitrary raw colors, BigInt relational values and adversarial GC/reentry. Reproduce those cases against unchanged source before changing behavior. No nominal successful substitute closes them.

## Evidence

`results/public-decoration/index.json` pins 1,872 source inputs, 21 harness inputs, 150 durable receipts, 773 temporary runtime files and seven cached native executables at review. The actual input SHA is `09ebf2ad5cefc5706d248553cfdc31beb82642863be913c50fa159babbba627d`. Original P: and physical owned reference copies matched all 776 baseline hashes after execution.

The shared 14-phase trace passes against the unchanged original and actual exported Rust Terminal in Chromium, Firefox and WebKit. Rust's single function compares the complete source fixture. Initial and restored-final runs pass each engine. All 16 changed production files were replaced with exact old `0442e357` source, with new modules absent. That old engine compiles and fails Chromium with zero passed and one failed at the absent API. `finally` restores every source hash. Initial/final compiler WASM, served WASM and bindings are identical; negative hashes differ. All three served test modules embed the full trace and fixture.

The first control observer searched stdout, but wasm-bindgen-test's intended error was in its console panic. Raw failure is retained. Corrected verification resumed from checked old receipt, source and artifact pins without replaying the negative run. The first source trace also exposed the test's unnormalized CSS decimal expectation. Its exact CSSOM expected-value correction and raw diagnostic are retained, without a tolerance or source/production assertion change.

Six retained browser suites pass 18 functions per engine. A matching production debug package runs unchanged original decoration/color-override suites: 33 case-projects discovered, 27 executed/passed, six original WebKit renderer skips, zero failures and zero unrun cases. First-attempt zero discovery from a missing PNG dependency is retained separately. Seven existing dependency packages were physically copied and all 313 files SHA-verified, with no install or junction.

Native producer workspace all-target/all-feature tests pass 2,221 tests across 516 summaries, plus two doctests, with no failures, ignores, measurements, filtering or warnings. Strict native workspace and WASM workspace-library/public-test Clippy pass. All five whole-member format checks pass. Exact workspace formatting still fails with Windows OS error 206. This is Windows evidence, not native Linux/macOS or complete global feature/release evidence.

`review.json` retains the clean review. The reviewer verified every indexed pin, reran all seven native executables containing 36 tests, checked formatting for all 25 changed Rust files, inspected the negative control and found no scoped issues. It did not rerun builds or browsers and created no scratch files.

## Integration and resources

Urraca owns task 309/543 integration. No main-ref, canonical gate, remote CI or new build/browser allocation is held here. The matching package is for this producer only. A merged source digest differs, so rebuild or verify exact identities before using a package for final full-core runs. Reese owns broad original/demo verification in task 327; its current 27 decoration failures on the old main are the same scoped assertions fixed here. This scoped pass is not a full-core rerun.

All compiler/browser/server work was released through post 409 comment 4573. No owned jobs remain. Early schema TMP assignment expanded an old shell variable; it is disclosed rather than treated as enforced O: scratch. Final phases explicitly set TEMP, TMP and TMPDIR to owned O: storage, and no unknown temporary paths were deleted.

The only owned temporary root is `O:/xterm-rust-swarm-16/tmp/worktrees/SarahDecoration`, including regular target/tmp, physical reference and seven physical dependency packages. This document is written before removal. The owner must preserve the branch, verify every path for reparse points/redirects and every owned process, remove only hash-verified generated outputs, then use normal non-force Git worktree removal. The subsequent post 409 report and private `.pi/swarm/work/2/sarah537-cleanup.json` establish actual absence, unregistration and original-reference hashes after removal. Never clean another owner's work or traverse a reference junction.
