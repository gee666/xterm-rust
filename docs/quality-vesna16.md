# Native quality checkpoint

Overall equivalence is NOT PROVEN. Task361 records this checkpoint. Final integrated quality task304 remains open while application producers continue changing main. Task362 repairs current stable Clippy failures separately.

## Revisions and environment

Local commands use unchanged integrated commit `5ca9f5932284e78bb06c89bdd209801e7304d8db` in private physical worktree `O:/xterm-rust-swarm-16/tmp/V`. The input snapshot hash is `8155e80153e72af92202ac0e21998a3c0c4744dee6aee1bf13a9f5af1fa5ef12`. Results include source hashes before and after each matrix, compiler host, resolved native target, CPU, OS, Cargo metadata, installed components, exact commands, deadlines and raw output.

Local Windows is `10.0.26300`, x86_64, Ryzen 9 7900X, Node `24.15.0`. The existing default Rust compiler is `1.85.1`. The other previously installed compiler is `1.97.1`, which is NOT current stable. CI reports current stable `1.99.1`. The directory named `vesna-stable-5ca9f593` is the local installed `1.97.1` run, not evidence of current stable compatibility.

All local compilers run sequentially with Cargo `-j2`, the worktree's regular `target/`, unchanged repository profiles, and TEMP/TMP/TMPDIR under its owned `tmp/`. No browser matrix belongs to this checkpoint.

## Local MSRV

Initial command:

```sh
node scripts/quality-gates.mjs --out results/quality/vesna-msrv-5ca9f593 --wasm-check --timeout-ms 180000
```

The cold all-feature test and release-build commands reached their explicit 180-second deadlines. These are failed attempts, not passing or skipped gates. The test log contains 1,573 passed functions in 235 summaries before termination, with no assertion failure reported. Remaining outcomes are unreported. Both timeout logs and command termination metadata are retained unchanged.

The bounded warm rerun used a new result directory:

```sh
node scripts/quality-gates.mjs --out results/quality/vesna-msrv-warm-5ca9f593 --wasm-check --timeout-ms 600000
```

It PASSED all required native gates with identical before/after source hashes. All-feature, default and no-default test commands each report 2,012 passed functions in 385 summaries, with zero failed or ignored. These are overlapping configurations, not 6,036 distinct tests. All-feature doctests report one passed function in five summaries. The individually declared `xterm-web/acceptance-hooks` feature reports 326 passed functions in 113 summaries. Exact full formatting, warning-denied all-feature/no-default Clippy, optimized release all-target/all-feature build, and all-feature workspace WASM library compilation also pass. Zero-test host binaries do not establish browser execution.

Local Miri is unavailable on this compiler. Its failed availability probe is retained, and no local Miri or sanitizer execution is claimed. CI has a separate focused Miri job, which is not complete safety verification of the workspace or its dependencies.

The existing CI runner verification command also passes all 45 Node tests with zero skipped or cancelled. The raw output is retained with the checkpoint evidence.

## Local installed newer compiler

The same immutable source finished with `RUSTUP_TOOLCHAIN=1.97.1`, the existing quality runner and 600-second command deadlines. Its source hash remained unchanged. All-feature, default and no-default tests each pass 2,012 functions in 385 summaries. Doctest1, feature326, full formatting and optimized release all-target/all-feature build pass. Warning-denied all-feature and no-default Clippy fail on two equivalent loop-head forms in `ligatures/font_walk.rs` and `web_links.rs`. There are no command timeouts in this run. The checkpoint does not waive these failures or use lint suppression. WASM was not requested for this compiler because its installed targets do not include WASM. The MSRV WASM result above and CI's separate WASM result have distinct provenance.

## Native CI checkpoint

A non-force push created only the unique branch `ci/vesna16-bb6eada9` at integrated commit `bb6eada9e6024d9f106ccb22dcb9e7a7dee6427e`. Main was not pushed or rewritten. Existing workflow `Rust native quality` started run [42012763111](https://github.com/gee666/xterm-rust/actions/runs/42012763111). No duplicate dispatch was submitted.

That CI revision includes task357 after the local snapshot. It is not the local snapshot and must not be reported as one. Six native OS/toolchain jobs, focused Miri and WASM compilation run on the recorded CI revision. Raw artifacts and final job conclusions belong under `results/quality/ci-42012763111/`.

Observed Linux stable uses Rust `1.99.1`, not local `1.97.1`. Its quality gate fails on the two loop diagnostics plus a test-only manually implemented no-op waker in `osc_edges.rs`. Its unchanged all-feature/default/no-default tests, release build, doctests and formatting pass. Task362 owns equivalent loop/no-op-waker simplification without dropping assertions. Native CI failures are application quality work, not unavailable-platform blockers.

The run completed with overall conclusion FAILURE. All eight job logs and all eight artifact ZIPs have been retrieved, not merely inferred from workflow configuration. `observed-artifacts.json` summarizes the original ZIP contents without replacing them. Its six native summaries have stable before/after source hashes and no fatal provenance errors.

| Native job | Observed result |
| --- | --- |
| Windows 1.85.1 | PASS |
| Linux 1.85.1 | PASS |
| macOS 1.85.1 | PASS |
| Windows stable 1.99.1 | FAIL, all-feature and no-default Clippy |
| Linux stable 1.99.1 | FAIL, all-feature and no-default Clippy |
| macOS stable 1.99.1 | FAIL, all-feature and no-default Clippy |

Every native job passes 2,014 functions in 385 summaries for each all-feature/default/no-default configuration, one doctest in five summaries, and 328 functions in 113 summaries for the individually enabled feature. No failed or ignored tests are reported. CI has two more tests than the local snapshot because its recorded revision includes task357. All six jobs also pass formatting and optimized release all-target/all-feature builds. The stable jobs still FAIL as a whole. Task362's matching local `--keep-going` Clippy additionally enumerates equivalent helper diagnostics hidden behind CI's first compilation failures.

The separate Miri job actually executes and passes 21 functions across `cell_upstream` and `lifecycle`. The WASM job passes workspace all-feature library compilation. These are scoped results, not complete safety or browser runtime proof.

A successful focused Miri or WASM job is not a successful six-job native matrix, browser execution or full-port equivalence. No post-task362 CI result is included here; its original failures remain intact.
