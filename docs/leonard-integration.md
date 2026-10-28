# Leonard integration checkpoint

Overall equivalence is NOT PROVEN. This branch carries the Berenice core fixes, including accepted Windows task267, and adds accepted test-lint repair276. The unchanged canonical gate merged this branch into main at `acee593` after the task60 bootstrap. Task264 independent review accepted the integration. No browser engine or source reference was edited here.

## Provenance

`swarm/Leonard` started at `ef332c6`, then cherry-picked the twelve Berenice commits from `c30f095` through `4ac348b`. Their source and retained CI failures remain available. Willie diagnostic phase commits are `6669e24` and `be6a952` here. Windows listener ordering is `7412147`; the original accepted source commit is `0bb8aba`. The virtual-reset change `2883753` duplicates Xavier `08892e0` and should be skipped on rebase when already present.

`9bb4118` fixes three demonstrated test lints without altering production code or expected results. Review276 independently reran all three test modules plus public-cell tests, 26 passing tests, exact-file formatting, and full-core Clippy. `0cfbf95` carries Salahuddin's existing `67d5cc8` initializer-only prerequisite instead of duplicating it.

## Main integration

After Ernest bootstrap `b6ed3ff`, the canonical helper rebased this branch. The one conflict in headless disposal retained both reset-hook cleanup and immediate parser disposal. Git dropped duplicate virtual-reset and diagnostic-phase patches already in main. The diagnostic test still checks four records, exact UTF-16 input/reply values and cursor phases `[0, 0, 2, 2]`.

`node scripts/worktree-merge.mjs` then passed1492 core tests across223 test/example suites, native all-target checks for all affected crates, workspace check and the browser/WASM library check. It fast-forwarded main to `acee593` and removed this private target. Exact successful transcript is `results/integration/leonard/gated-merge.log`; the first conflicting rebase is retained in `gated-rebase.log`.

Final rebased code commits are `aa01851` for Windows ordering, `bbe11fe` for immediate parser disposal, `ac589e9` for accepted lint repair276 and `88d7ea7` for the existing numeric test initializer fix. `ea15a85` formats the diagnostic state construction. Earlier commit IDs below describe the pre-rebase checks, not a claim that those checks ran against the larger main bootstrap.

## Pre-rebase local commands and results

Run from this worktree. Environment is native Windows MSVC, Rust/Cargo1.85.1, LLVM19.1.7. Logs are `results/integration/leonard/`.

- `cargo test --locked -p xterm-core --test core_services_integration --test input_diagnostics --test windows_wrapping_order --test headless_disposal_contract --test headless_reset_host`: 19 tests passed across five suites, `focused-tests.log`.
- `rustfmt --edition 2021 --check crates/xterm-core/tests/move_to_cell.rs crates/xterm-core/tests/event_context.rs crates/xterm-core/tests/input_handler_bounded.rs`: passed, `core-test-fmt.log`. An initial invocation without the edition failed before parsing async syntax; it was corrected, not treated as a passing check.
- `cargo clippy --locked -p xterm-core --test move_to_cell --test event_context --test input_handler_bounded -- -D warnings`: passed, `core-test-clippy.log`.
- `cargo test --locked -p xterm-core --test move_to_cell --test event_context --test input_handler_bounded --test public_cell_numbers`: 26 passed, `core-test-tests-2.log`. Both 512-case move-to-cell corpora and every assertion remain intact.
- `cargo clippy --locked -p xterm-core --all-targets --all-features -- -D warnings`: passed, `core-all-clippy-2.log`. Earlier failing logs retain each demonstrated lint.
- `cargo test --locked -p xterm-core --all-targets --all-features`: 1478 passed across221 test/example suites, `core-all-tests.log`.
- `cargo test --locked -p xterm-core --no-default-features --all-targets`: 1478 passed across221 test/example suites, `core-no-default-tests.log`.
- `cargo test --locked -p xterm-core --doc`: one passed, `core-doctests.log`.
- `cargo test --locked -p xterm-core --release --test core_services_integration --test input_diagnostics --test windows_wrapping_order`: 14 passed across three suites, `release-tests.log`.
- `cargo test --locked -p xterm-core --release --all-targets --all-features`: 1478 passed across221 test/example suites, `core-release-all-tests.log`.

The first short coordination post mistakenly said29 focused tests. Counting the actual retained transcript gives19, as reported above.

## Pre-rebase failed gates and remaining scope

- `cargo fmt --all -- --check`: Windows error206 before the formatter can run, `fmt.log`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: stopped at the now-repaired core test hex lint, `clippy-before.log`. No full-workspace success is inferred from scoped Clippy.
- `cargo test --workspace --all-targets --all-features`: failed before the bootstrap on missing web module exports and the u32/u64 URL fixture mismatch, `tests-before.log`. Task60 fixed those compile gaps. The successful merge checked affected native targets but did not rerun the whole workspace all-feature runtime command here.

## Independent task264 review

The reviewer accepted task264 on the integrated snapshot at `026a8d8`. It independently ran1492 core tests,1942 workspace all-target/all-feature tests, one workspace doctest,19 focused regressions and14 focused release tests. Core Clippy, native all-target checks, web/support WASM library checks, the all-feature workspace release build and exact-file formatting passed. Only four command transcripts were retained by the reviewer: `review264-core-tests.log`, `review264-workspace-tests.log`, `review264-workspace-doc.log` and `review264-workspace-clippy.log`, now copied unchanged into this result directory. Other observations above are the review verdict, not fabricated log files.

Workspace formatting still fails with Windows error206. Workspace Clippy still reports existing test lints at `crates/xterm-web/src/renderer/width_cache_tests.rs:131` and `crates/xterm-web/tests/scrollbar_visibility.rs:52`. No workspace Clippy pass is claimed. The reviewer found no producer deletion, assertion weakening, lint suppression or unsafe change.

Linux/macOS runtime, full browser/addon/upstream suites and benchmark equivalence are not established by these results. Params226, diagnostics194 and genuine async resize replay257 remain unfinished implementation work.
