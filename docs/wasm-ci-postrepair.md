# Postrepair native and all-target WASM CI

Task 388 records observed hosted verification after task 389. Overall xterm.js equivalence remains NOT PROVEN. Final task 304 and full source/browser/addon parity remain open; these results apply only to the frozen revisions below.

## Failed attempt retained

Run [42039966380](https://github.com/gee666/xterm-rust/actions/runs/42039966380) at integrated `aa3f79211e48175f317261592ff57bd20e4dc79c` completed with FAILURE. Stable Rust 1.99.1 WASM all-target compilation passed, but warning-denied Clippy failed on the 32-bit buffer-cap minimum. Windows Rust 1.85.1 separately reached the 300-second cold release-build deadline. Its tests and other gates passed. Neither failure was reclassified as unavailable OS access or a passing gate.

All eight original ZIPs and complete job logs remain under `results/quality/ci-42039966380/`, with CRC-checked derived counts and SHA-256 bindings. [The source repair](stable-wasm-quality.md) preserves these failures, the subsequent matching-compiler diagnostics, regression results, bounded-deadline change and shared Rustup manager caveat.

## Canonical source integration

The isolated reviewer accepted task 389, then reaccepted its integrated-union correction at `e7cea61`. It independently reran full WASM all-target/all-feature warning-denied Clippy on Rust 1.85.1 and actual 1.99.1, formatting and focused native tests. Every pre-existing assertion remained intact.

The unchanged canonical helper merged source `e7cea61d6e11fab4b614b3905cff0489cb64be80` into actual main from `da0acdf0`. Changed core/web native builds and all-target tests passed, with 1,895 passes across 370 summaries, zero failed or ignored tests. Affected native all-target, workspace and applicable WASM library checks passed. The helper removed the private target. The canonical log and hash-bound receipt are retained with this checkpoint; its scoped default-feature total is not the full hosted matrix below.

## Observed successful postrepair run

A non-force push created only `ci/bohemond388-e7cea61d` at that exact integrated source. Main was not pushed or rewritten, and no duplicate dispatch was submitted. Run [42046682640](https://github.com/gee666/xterm-rust/actions/runs/42046682640) completed successfully. All eight jobs and all required gates passed.

| Native runtime | Rust 1.85.1 | Actual stable 1.99.1 |
| --- | --- | --- |
| Windows x64, MSVC | PASS | PASS |
| Linux x64, GNU | PASS | PASS |
| macOS arm64 hosted VM | PASS | PASS |

Each job records actual native runtime, compiler host and matching resolved Cargo target OS. This is neither cross-compilation nor a claim about physical macOS hardware.

Every native job reported:

| Configuration | Passed functions | Harness summaries | Failed / ignored |
| --- | ---: | ---: | ---: |
| All targets and all features | 2,062 | 412 | 0 / 0 |
| Default features | 2,062 | 412 | 0 / 0 |
| No default features | 2,062 | 412 | 0 / 0 |
| All-feature doctests | 2 | 5 | 0 / 0 |
| Individual acceptance-hooks feature | 343 | 128 | 0 / 0 |

The configurations overlap and must not be summed as distinct assertions. Empty native WASM harnesses are included in summary counts, not counted as browser execution.

Full formatting, all-feature and no-default warning-denied Clippy, optimized all-target/all-feature release builds and all native test gates passed. Every native summary has unchanged before/after source hashes, no failed required gates and no fatal error. Windows hash is `2ee4ff67901c01dcecc74539cc88a447ec01bb7b4cbde2490e1ff0ff00247424`; Linux/macOS hash is `a66eedaccee1ffa80b4a842de47ee5c1e37c866d35a8fbbb05227f00f4011cb3`. Checkout line endings explain the platform byte-hash difference; each execution identifies the same Git revision and its own unchanged byte snapshot.

Windows MSRV's successful release command took 288.96 seconds in this run with `timedOut: false`. Its new 600-second bound provides measured cold-build headroom. This is not a speedup measurement or a claim that increasing a deadline fixes compiler errors. The original timed-out attempt remains failed.

## Strengthened WASM and focused safety checks

Unlike the previous library-only check, the stable Linux WASM job executed both commands successfully:

```sh
cargo check --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown
cargo clippy --workspace --locked --all-targets --all-features --target wasm32-unknown-unknown -- -D warnings
```

This proves clean-checkout test/example compilation and linting, not browser runtime. Task 389 separately retains actual seven-boundary wasm32 browser assertions and image/public-buffer regressions.

The focused Linux Miri job executed and passed 12 cell plus nine lifecycle tests on Rust 1.101.0-nightly. That is 21 interpreted tests, not complete workspace or dependency safety verification. Optional MSRV/stable Miri availability failures remain in native summaries. Sanitizers were not run.

## Original evidence and cleanup

`results/quality/ci-42046682640/` retains all eight original artifact ZIPs, eight full job logs, live API metadata, a SHA-256 manifest binding those 16 files and derived observations. All 116 ZIP-entry CRCs verified. Every native per-suite JSON count was checked against its original raw log; source/compiler/platform provenance was checked separately. The WASM raw job log records both complete command lines and successful completion.

The temporary Rust 1.99.1 compiler was removed from owned `O:/xterm-rust-swarm-16/tmp/Q/tmp/rustup199` after source review and integration. The preflight found 215 files, no links and no exact-root compiler processes. Ten owned browser-run directories containing 578 files were also removed after preserving concise evidence. This did not undo the unintended shared Rustup manager update documented in task 389. No unknown shared files, original dependencies or other agent's worktrees were deleted.

The Q worktree remains only for final evidence review and batch integration, then must be removed after checking dirty files, links and owned processes. Remote jobs are complete. Broader application gaps, complete original-suite accounting, full demos/benchmarks and final task 304 remain unfinished.
