# Native gate recovery

This checkpoint fixes pre-gate CI failures and records a working Windows formatting command. It is not the final native matrix. Overall equivalence remains NOT PROVEN.

## Retired queue

The six native jobs in GitHub Actions run [40733999151](https://github.com/gee666/xterm-rust/actions/runs/40733999151), source `e720ce8f0826043a7fb673079f7bb8dacacee805`, failed before running Rust quality gates. Their verifier invoked queue tests after commit `89b4150` had explicitly retired the global queue. `runWithSlot` no longer acquired leases or forwarded cancellation; stale tests still required lease directories and queue deadlines. The same command on integration checkpoint `4b02ba1` discovered 61 tests, passed 56 and failed 5.

The queue has no application caller. `acquireSlot` is referenced only by the obsolete queue tests, and `runWithSlot` has no caller outside those tests. This change deletes that unused module and both obsolete queue suites instead of restoring the forbidden queue. The Windows relocation receipt no longer hashes the deleted module. Relocation's separate maintenance pause/ownership safety remains unchanged.

The active `quality-process` and `quality-managed` suites still assert failure output, exit status, pre-spawn and running cancellation, resource failure, finite deadlines, owned descendant termination, readiness errors and cleanup. No Rust, upstream or active process assertion was removed. A replacement regression invokes the retired CLI with a would-be child command and verifies rejection, useful worktree guidance and no child marker, receipt or lease data.

The updated native workflow verifier discovered and passed all 45 tests, with zero failures, skips or cancellations. Exact failing/passing local logs and the observed native CI job logs are retained under `results/quality/bohemond16/` and `results/quality/ci-40733999151/`.

## Windows formatting

Both installed toolchains, Rust 1.85.1 and 1.97.1, failed the exact command below at the original owned worktree path:

```sh
cargo fmt --all -- --check
```

At `O:/xterm-rust-swarm-16/tmp/worktrees/Bohemond16`, Cargo metadata listed 354 targets. Their absolute paths alone required 33,834 characters including separators/quoting, before cargo-fmt's canonical-path prefix overhead. cargo-fmt groups targets by Rust edition, exceeding the Windows process command-line limit. The earlier temporary drive mapping did not help because Cargo resolves it back to the long physical path.

Moving the actual new owned worktree to `O:/xterm-rust-swarm-16/tmp/B` solved the launch failure. The same complete command passed on both toolchains without a wrapper, package exclusions, test restructuring or changed assertions. The worktree move uses Git's metadata-aware operation:

```sh
git worktree move O:/xterm-rust-swarm-16/tmp/worktrees/Bohemond16 O:/xterm-rust-swarm-16/tmp/B
```

Do not move another agent's worktree or any legacy reference. Short-path evidence here is scoped to source `4b02ba1`; rerun full formatting after subsequent merges. The normal Windows Actions checkout is shorter still, but only an observed run can establish its result.

## Remaining matrix

Task 304 still requires source-bound Windows/Linux/macOS MSRV/stable native execution, release/default/no-default/single-feature checks, doctests and WASM compilation on a stable integrated revision. No native Rust matrix result is inferred from these Node tests or formatting receipts. Existing authenticated CI access works; historical failures are code/test failures, not an external OS blocker.

## Owned resources

Only the new owned worktree `O:/xterm-rust-swarm-16/tmp/B` and its `tmp/`/`target/` are disposable resources. TEMP and TMP are both explicitly set to that worktree's `tmp/` before subprocess runs. No owned background service or browser is running. Preserve this branch and durable results, inspect links and dirty files, then remove only this owned worktree after integration.
