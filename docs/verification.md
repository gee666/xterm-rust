# Verification status

Overall equivalence is **NOT PROVEN**. This is an in-progress evidence report, not a completion certificate. Common utilities have focused passing results, but the initial full-workspace gates failed. Terminal integration, complete API/addon coverage, native Linux/macOS execution and Rust-backed browser end-to-end results are not established here.

## Resume after disk exhaustion

Main reported that the P: drive reached about 7 MB free, interrupting the swarm with SQLite `cannot rollback - no transaction is active`. Main's database quick check passed. The original 45-worker run resumed; no second swarm was launched. Main removed only generated Cargo artifacts, recovering about 3 GiB, and committed `.cargo/config.toml` as `705104c`. See `docs/resume-run2.md` for the preserved scope and resource policy.

Aaliyah independently read the resumed brief and configuration and observed about 2.9 GiB free with `df -h .`. Development/test debug information and incremental caches remain disabled; assertions and release profiles are unchanged. Large builds must check available space, use the shared target directory and report low space before exhaustion. The Node-only evidence/runner tests do not create a second Cargo target directory.

The interruption exposed a stale-success hazard. Sargon reported that a prior `results/dcs/verification.json` still said success while the latest Rust log showed disk exhaustion. The verifier owner is correcting report initialization and retaining the failed attempt. Treat every receipt as a dated/source-bound observation; a successful JSON summary without its matching current logs is not enough. This resource interruption does not excuse missing engine/browser/addon implementation or establish an external blocker for that work.

`.gitattributes` disables line-ending conversion for retained manifests, raw results and oracle fixtures so checkout cannot silently rewrite hashed evidence. This does not modify `source_repo` or user-owned startup files.

Detailed acceptance checks are in [verification-acceptance.md](verification-acceptance.md). The [compatibility inventory](compatibility.md) is maintained separately and must map every source contract and suite. It is not permissible to infer implemented or passed behavior from a task's title or a file's existence.

## Reference provenance and integrity

The reference is the user-supplied `source_repo` xterm.js 6.0.0 tree. No public revision is asserted. The pre-launch `tmp/source-manifest.json` is retained byte-for-byte as `docs/baseline/source-manifest.json`. Its SHA-256 is `fad438d8ae8c48870f454646a1fbbc0bba8e6c87526bd3ae0b56be28b02a71f2`.

The first independent check matched all 776 listed files, with zero missing or changed files. This checks exact bytes, including original assertions. It does not check added files against the initial manifest. Generated dependency/build outputs are not a source revision. The fresh compatibility inventory documents its separate traversal and exclusions.

Run from the repository root with Node 24 or later:

```sh
node --test scripts/verification-index.test.mjs
node scripts/verification-index.mjs
node scripts/verification-index.mjs --check
```

The index records every retained regular file under `results/`, its byte length and SHA-256, excluding itself. `--check` leaves that snapshot intact and reports changed, missing and newly added evidence, altered manifest bytes and current reference mismatches. It exits nonzero when the snapshot is stale. Seven Node tests include symlink/junction refusal, changed/missing/added files and a manifest-only formatting change, and assert that checking never rewrites the prior index. It reports listed reference mismatches separately and exits nonzero for changed or missing files. Invalid paths, duplicates and unhandled I/O errors fail rather than being ignored. Its tests cover byte hashing, ordering, changed/missing files, invalid paths, CLI failure status and index self-exclusion.

`results/verification/index.json` is a file snapshot, not a test runner. Regenerate it after all report/result edits. Do not redirect its stdout to a file under `results/` during generation, because that would hash a log before its final contents are written. Concurrent agent writes can make any file inventory stale; final release checks must use an immutable committed snapshot.

## Independently rerun module checks

The reviewer was Aaliyah. Observed environment was native Windows x64, Node v24.15.0, rustc 1.85.1 and cargo 1.85.1. Commands, results, seeds and review findings are retained in `results/verification/reviews/summary.json`, with raw logs beside it.

| Scope | Observed result | Limits |
| --- | --- | --- |
| Lifecycle | 9 Rust tests and the shared 16-callback JS fixture passed | No upstream Lifecycle.test.ts exists. Direct contract tests, not a fictitious migrated suite. |
| Params | 20 unchanged original Mocha/Chai cases, 22 Rust tests and 49,409 JS snapshots passed | Seed `0x2389abcd`; typed numeric/API differences documented in `params.md`. |
| UnicodeV6 | 65,536 original BMP assertions, 1,114,112 width fixtures, 110 property cases and 5 Rust tests passed | Provider only, not UnicodeService, grapheme addons or terminal composition. |
| XParseColor | 7 original cases with 49 assertions, 153,872 differential operations and 9 Rust tests passed | Seed 1480806220; no terminal OSC/browser integration coverage. |

The combined focused Rust command passed 45 test functions, with zero failures or ignored tests. Do not add the assertion, operation, snapshot and test-function totals together. They count different things.

```sh
cargo test -p xterm-core --test lifecycle --test params --test params_differential --test unicode_v6 --test xparse_color
cargo clippy -p xterm-core --lib --test lifecycle --test params --test params_differential --test unicode_v6 --test xparse_color --example xparse_color_oracle -- -D warnings
node scripts/check-lifecycle.mjs
node scripts/check-params.mjs
node scripts/params-baseline.mjs
node scripts/unicode-v6-oracle.mjs
node scripts/check-xparse-color.mjs
```

Focused warning-denied Clippy and explicit-file rustfmt also passed. The exact rustfmt file list is in the receipt. These historical focused runs happened in a changing shared working tree, not a frozen final revision. Their original-source hashes and fixtures are recorded by the oracle scripts, but they cannot substitute for final immutable-snapshot workspace results.

Other reviewers' receipts live under `results/reviews/` and subsystem result directories. For example, Wallada's keyboard review records 61 original cases, 156 assertions, 251,452 differential cases with seed 1262836051, and 8 Rust test functions. Those are scoped observations, not a full-terminal result. Consult the receipt and exact source mapping before claiming a contract passed.

## Full-workspace quality results

Kassa owns the portable quality runner and CI execution. The initial native Windows run is retained as `results/quality/windows-initial/summary.json`, including exact argv, exit codes, logs, source hashes and environment.

| Initial gate | Result |
| --- | --- |
| cargo fmt --all -- --check | Failed |
| Workspace all-target/all-feature Clippy with warnings denied | Failed |
| Workspace all-target/all-feature tests | Failed |
| Workspace all-feature doctests | Passed command, not proof of documented example coverage |
| Workspace release all-target/all-feature build | Failed |
| Workspace default-feature tests | Failed |
| Workspace no-default-feature tests and Clippy | Failed |
| Miri availability probe | Failed; Miri tests were not run |

The initial summary names seven failed build/lint/test/format gates. Source hashes changed during execution, so even passing commands do not establish one-snapshot gate success. Live DCS API mismatches and BufferLine test lint errors were reported to their owners. Keep this failed attempt when later reruns pass; do not overwrite it or infer that its errors remain current without rerunning.

The source scan found no unsafe candidates in that snapshot. This was a lexical scan, not a semantic safety proof. Miri and sanitizers were not executed. The WASM target/build was not requested in this attempt. See the quality receipt for flags, installed targets, tool versions and exact failure logs.

## Independent findings and retained regressions

- Lifecycle originally skipped resources added after a guarded nested clear. Cornelia found the discrepancy with JavaScript Set iteration. The owner retained a shared JS/Rust regression, implemented iterator tombstones and passed independent reruns. Task #5 was accepted for this module scope.
- Bradley found five malformed CSS parseInt discrepancies after the original Color suite and its first random corpus were green. `results/reviews/color-bradley.json` retains the inputs and expected/actual packed results. That receipt requests changes, so it is not a passing Color review. A later fix requires its own regression evidence and acceptance.
- APC uses shared OSC dispatch. Viesturs found that callbacks deleting active registrations can make upstream access a missing array element and throw, while the first Rust version silently skipped it. Task #17 received changes requested. The owners are adding explicit Result errors and retained mutation cases. Do not declare that discrepancy fixed from an API signature change alone.
- Tecuichpo reported CircularList capacity behavior needing a sparse representation instead of eager Vec allocation for large source-supported capacities. This remains a reported finding until its retained boundary tests and review demonstrate the correction.
- The LoC decoder originally stripped a BOM-only source file, counting zero physical lines instead of one. Aaliyah retained the counterexample, Octavia fixed `ignoreBOM`, and independent reruns passed all eleven counting tests plus the exact legacy baseline. Task #49 accepts the method and partial snapshot only. Task #98 owns the final post-integration refresh. See `results/verification/reviews/loc-review.json`.
- Options object coercion initially used one infallible conversion callback for both numeric comparison and string error formatting. Aaliyah identified the missing JS hint/error paths. Athena is adding fallible Number/String hints and tests; live migration compile failures remain failed attempts, not passing Options evidence.
- The first original-unit baseline wrapper reused fixed log/report paths. Aaliyah requested changes to task #92 so isolated attempts and upfront incomplete status prevent overwritten failures or stale successful summaries after interruption. Existing original results are not proof that this wrapper defect is fixed.

The review list changes as fixes land. Module acceptance never establishes browser wiring, platform behavior or complete terminal equivalence. Reviewers must inspect actual code and execute checks; a swarm lifecycle state is not a test result.

## Coverage accounting and remaining checks

The first compatibility snapshot identified 209 production modules, 770 declared API contracts, 14 addons and 120 test-related files. It classified 73 unit, 25 browser-integration, 6 benchmark and 16 test-support files. Its 2,449 static test definitions are not runtime assertions or executed cases. The inventory initially marked 196 modules missing and 13 partial, and 766 APIs missing and 4 partial. These are mapping snapshot counts, not the current implementation percentage; regenerate the inventory as owners add mappings.

Per-suite execution counts remain unknown unless a result actually measures them. Missing original/Rust/browser execution cannot be entered as zero failures. The suite inventory contains explicit `not assessed` and null counts where no run was observed. Reconcile future original baseline and Rust receipts with that inventory before publishing aggregate totals.

Required work not established by this report:

- Complete parser/buffer/engine/public API integration and faithful remaining upstream suites.
- Full-state terminal differential tests for cells, attributes, cursor, modes, scrollback, events, replies and APIs across streaming boundaries, malformed input, resize/reflow and alternate screen.
- All fourteen addon implementations and actual terminal/browser integration, including Unicode/grapheme, image, search/serialize, WebGL, font and interaction paths.
- Observed native Linux and macOS runtime verification. A CI workflow or cross-build is not evidence that either ran.
- WASM bindings and actual Rust-backed Chromium, Firefox and WebKit end-to-end assertions for rendering, input, mouse, selection, clipboard, IME, sizing, links, accessibility, disposal and addons.
- Working user-run native and Rust-backed browser demos from a clean checkout.
- Matched validated release speed/artifact-size comparisons with warmup, at least ten repetitions, median/p95 and JS/WASM/native distribution distinctions. LoC must use the same classification/counting method for both trees.
- Final warning-free stable-snapshot workspace gates, safety review, committed source and independent final gap audit.

These are unfinished internal work and unrun verification, not automatically external blockers. If an external resource prevents a required run, retain the exact attempted command, missing resource, resolution attempts and concrete unblock step. No such limitation permits changing `NOT PROVEN` to a completion claim.
