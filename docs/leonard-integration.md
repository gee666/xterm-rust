# Leonard integration checkpoint

Overall equivalence is NOT PROVEN. This branch carries the Berenice core fixes, including accepted Windows task267, and adds accepted test-lint repair276. Task264 still requires a gated main merge after task60 browser bootstrap. No browser engine or source reference was edited here.

## Provenance

`swarm/Leonard` started at `ef332c6`, then cherry-picked the twelve Berenice commits from `c30f095` through `4ac348b`. Their source and retained CI failures remain available. Willie diagnostic phase commits are `6669e24` and `be6a952` here. Windows listener ordering is `7412147`; the original accepted source commit is `0bb8aba`. The virtual-reset change `2883753` duplicates Xavier `08892e0` and should be skipped on rebase when already present.

`9bb4118` fixes three demonstrated test lints without altering production code or expected results. Review276 independently reran all three test modules plus public-cell tests, 26 passing tests, exact-file formatting, and full-core Clippy. `0cfbf95` carries Salahuddin's existing `67d5cc8` initializer-only prerequisite instead of duplicating it.

## Local commands and results

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

## Gates still failing on this branch

- `cargo fmt --all -- --check`: Windows error206 before the formatter can run, `fmt.log`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: stopped at the now-repaired core test hex lint, `clippy-before.log`. No full-workspace success is inferred from scoped Clippy.
- `cargo test --workspace --all-targets --all-features`: existing missing web module exports and u32/u64 URL fixture mismatch, `tests-before.log`. Ernest task60 owns the coherent browser bootstrap and existing fixture fix, not this branch.

Linux/macOS runtime, full browser/addon/upstream suites, full release workspace and benchmark equivalence are not established by these results. Params226, diagnostics194 and genuine async resize replay257 remain unfinished implementation work.
