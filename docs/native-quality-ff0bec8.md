# Integrated native quality at ff0bec8

Overall xterm.js equivalence remains NOT PROVEN. Task 384 records this immutable integrated checkpoint. Final task 304 remains open while application producers change main. This report does not upgrade later revisions or passing zero-test native WASM harnesses into browser coverage.

## Source and execution

Local Windows and GitHub Actions both used integrated commit `ff0bec8b5f9d82c0a3df64a541a9dbc2ae407524`. The only push created unique branch `ci/bohemond304-ff0bec8b`. Main was not pushed or rewritten, and no duplicate workflow dispatch was submitted.

Local execution used Windows x64, Rust 1.85.1, Node 24.15.0 and the owned physical worktree `O:/xterm-rust-swarm-16/tmp/Q`. Cargo used two jobs, the regular private `target/` and unchanged repository profiles. TEMP, TMP and TMPDIR all pointed to the worktree's `tmp/`. The whole matrix took 664.7 seconds; individual command deadlines were 600 seconds.

```sh
CARGO_BUILD_JOBS=2 node scripts/quality-gates.mjs \
  --out results/quality/bohemond304-msrv-ff0bec8 \
  --wasm-check --timeout-ms 600000
```

The local matrix passed, with source hash `30adb86ba18df7e267c7e3adb1cd32b6e491c890aee542f1e337b6ccb77980fd` unchanged before and after. Exact argv, tool versions, metadata, target OS, raw output and per-harness counts are retained in that result directory. No browser or managed server belongs to this native run.

## Observed native matrix

[GitHub Actions run 42031502588](https://github.com/gee666/xterm-rust/actions/runs/42031502588) completed successfully on the same source revision. All eight jobs and every required gate passed.

| Native runtime | Rust 1.85.1 | Observed stable 1.99.1 |
| --- | --- | --- |
| Windows x64, MSVC | PASS | PASS |
| Linux x64, GNU | PASS | PASS |
| macOS arm64 | PASS | PASS |

These are actual hosted runtime tests, not cross-compilation or a workflow-file claim. Reports identify the GitHub-hosted environment, compiler host and resolved Cargo target OS. macOS is a hosted arm64 VM, not a physical-machine claim. Local installed Rust 1.97.1 is not presented as current stable.

Every native job and the local matrix reported the same counts:

| Configuration | Passed functions | Harness summaries | Failed / ignored |
| --- | ---: | ---: | ---: |
| All targets and all features | 2,041 | 399 | 0 / 0 |
| Default features | 2,041 | 399 | 0 / 0 |
| No default features | 2,041 | 399 | 0 / 0 |
| All-feature doctests | 2 | 5 | 0 / 0 |
| Individual `xterm-web/acceptance-hooks` feature | 333 | 119 | 0 / 0 |

These configurations overlap. They are not 6,123 distinct assertions or proof that native execution ran WASM-only tests. The one declared non-default feature is inventoried from Cargo metadata; its combination is separately exercised.

Exact full formatting, all-feature and no-default warning-denied Clippy, optimized release all-target/all-feature builds and all native test gates passed. Each native summary reports `sourceStable: true`, no fatal error and no failed required gate. Windows source hashes match the local hash above. Linux/macOS report unchanged hash `a7c4dfb8c3ba401f882425d2dcc15a4fbb6920d6dc96aecb7c762fca28196e4e`; checkout line endings differ, but all executions identify the same Git revision and each retains its own unchanged byte snapshot.

## Miri, WASM and artifact verification

The separate Linux focused Miri job actually executed 12 cell and nine lifecycle tests, all passing, on nightly Rust `1.101.0-nightly`, commit `2d20b42b8`. This is 21 scoped interpreted tests, not complete memory-safety verification of the workspace or dependencies. Native stable/MSRV Miri availability probes remain optional failures in the raw summaries. Sanitizers were not run.

The separate stable WASM job passed all-feature workspace library compilation. It did not execute browsers or check every WASM test target. A subsequent clean all-target WASM check exposed the missing CoreBrowserService test oracle and test-only warnings. Task 383 repairs those separately; the green native checkpoint does not excuse them.

All eight original artifact ZIPs and all eight complete job logs are retained under `results/quality/ci-42031502588/`. `SHA256SUMS` binds those 16 original files. `run.json` and `artifacts.json` preserve live API observations. `observed-artifacts.json` derives platform/compiler/source/count summaries from the original ZIPs without replacing them. Verification checked all 116 ZIP-entry CRCs and compared every native per-suite summary against its raw command log, including zero-test harnesses and doctests.

Historical failed runs and earlier accepted checkpoints remain intact. Full unchanged upstream/native/browser/demo accounting, current application gaps, benchmarks, complete API mapping and final post-producer task 304 are separate obligations.
