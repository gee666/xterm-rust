# Workspace gate repair checkpoint

This checkpoint repairs formatting and test compilation on integration base `3ee576a9b01c774d2305936ec9e44993df7f0b5d`. It is not final port verification. Task 304 owns the stable post-integration quality and native OS/toolchain matrix.

## Repairs

- `cargo fmt --all` corrected 18 files without changing assertions. On the shorter Zhao16 O: worktree path, the complete `cargo fmt --all -- --check` command runs successfully. Adding further targets can still exceed Windows process argument limits, as reported by the font owner. That separate failure remains unresolved here.
- The locked js-sys 0.3.77 has no `Symbol::new`. The clipboard coercion assertion now supplies `Symbol::iterator()`, still a JavaScript symbol, and still requires a TypeError. No production clipboard behavior or assertion was removed.
- Reused Clarence's `dc8a6ee` search test fix. A four-argument JavaScript call uses `Function.apply` and an argument array because the locked js-sys lacks `call4`. The test loads tracked CSS instead of an absent generated copy.

## Observed native Windows results

Host is `x86_64-pc-windows-msvc`, Rust/Cargo 1.85.1. Builds use private O: target artifacts and two Cargo jobs. Raw logs are under `results/quality/zhao16/`.

| Command | Source checkpoint | Result |
| --- | --- | --- |
| `cargo fmt --all -- --check` | `e916e91`, before search cherry-pick | Passed |
| `cargo clippy --workspace --locked --all-targets --all-features -j2 -- -D warnings` | `9738b4a` | Passed |
| `cargo test --workspace --locked --all-targets --all-features -j2` | `9738b4a` | 1,962 passed, zero failed or ignored, 349 harness summaries including empty examples/WASM-only native harnesses |
| `cargo test --workspace --locked --doc --all-features -j2` | `e916e91` | One compile-fail doctest passed, other three crate doctest harnesses had zero cases |
| `cargo check --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown -j2` | Formatting checkpoint plus uncommitted Symbol fixture fix | Failed on stale `wasm_input` host implementations and one unnecessary `mut`; Diponegoro owns those repairs |

These native totals are test functions, not upstream assertion or browser case counts. The native commands do not execute tests guarded by `cfg(target_arch = "wasm32")`. Release builds, default/no-default feature combinations, stable Rust, and post-merge full gates remain task 304 work, not passes inferred from these results.

The initial WASM attempt also retains the `Symbol::new` compilation failure in `wasm-first.log`. `wasm-second.log` shows subsequent input test-host failures after that repair. No failure is reclassified as a platform limitation.

## CI access audit

The existing `scripts/github-quality.mjs` safely read GitHub Actions using the configured Git credential helper. No credentials were logged or copied. The latest observed Rust workflow run was [40733999151](https://github.com/gee666/xterm-rust/actions/runs/40733999151), source `e720ce8f0826043a7fb673079f7bb8dacacee805` on historical branch `swarm/Joyce-integration`.

The workflow reported six failed native Windows/Linux/macOS MSRV/stable jobs, a successful focused Miri job, and successful WASM library compilation. This is historical workflow status, not final native runtime proof, nor proof of the reasons those native jobs failed. The raw API status is retained in `results/quality/ci-40733999151/run.json`. Access works; there is no demonstrated external OS/CI blocker. Task 304 must obtain source-bound native execution evidence after the producer changes integrate.

## Canonical integration

The isolated review accepted task 302 at `b51ef4a`, independently checking repair diffs, log hashes and test totals and rerunning the complete fmt command. The reviewed pre-rebase history is retained as `archive/Zhao16-quality-pre-rebase` so the command-source references above remain reachable.

The canonical helper rebased the repairs onto `c17bff2` and merged `1ad04e1b043e539856f52fc9b8f9668a67a4d8d9` into main. It built and all-target tested all four changed crates with default features, checked affected dependents/all workspace code and the applicable WASM libraries, and removed the private target. Its log has 1,962 passing tests and 350 harness summaries, including empty harnesses. It does not add an all-feature/release/platform/browser claim. Exact output and SHA-256 are retained in `results/quality/zhao16/canonical-merge.{log,json}`.

The first helper invocation correctly refused three tracked reports that the earlier tests had refreshed. I inspected and restored only those generated report paths to preserve the earlier failure evidence. The second invocation completed without waived gates.

Overall equivalence remains NOT PROVEN.
