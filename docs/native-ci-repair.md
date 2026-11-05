# Integrated native CI after the current-stable repair

Overall xterm.js equivalence remains NOT PROVEN. This is task368's native quality checkpoint, not final integrated task304, complete original-suite task327, browser runtime acceptance or a completion certificate.

## Provenance

The workflow runs exact integrated revision `af047454ac338ac11c1b84784c2a84b92791894e` on the unique branch `ci/vesna16-af047454`. Only this new CI branch was pushed, without force. Main was not pushed or rewritten. The existing push trigger started [run42017720988](https://github.com/gee666/xterm-rust/actions/runs/42017720988); no duplicate workflow dispatch was submitted.

The revision contains independently accepted and canonically integrated native warning repair362. The previous failing run42012763111 remains unchanged in [quality-vesna16.md](quality-vesna16.md) and its original artifact directory. Local repair checks, independent217-test rerun and the canonical merge receipt are in [current-stable-quality.md](current-stable-quality.md).

Application producers have continued after this frozen CI revision. Even successful native quality on this commit does not establish complete functionality or final-source acceptance for later commits.

## Observation commands

The existing scripts use Git's configured credential helper without writing or printing tokens. From the repository root:

```sh
node scripts/github-quality.mjs runs
node scripts/github-quality.mjs status 42017720988
node scripts/github-quality.mjs artifacts 42017720988
node scripts/github-quality.mjs logs 42017720988
```

The immutable original artifact ZIPs, job logs, final `run.json` and `artifacts.json` belong in `results/quality/ci-42017720988/`. The derived `observed-artifacts.json` summarizes raw ZIP entries; it does not replace them. Original failures from earlier attempts remain in separate directories.

## Outcome

Run42017720988 completed with conclusion SUCCESS. All eight jobs finished successfully, and all eight original artifact ZIPs and eight complete job logs were retrieved. `SHA256SUMS` binds the retained ZIP and job-log bytes. Each native summary records the exact CI revision, a compiler host matching the executing OS, matching resolved native target OS, unchanged before/after source hashes, no failed required gates and no fatal provenance error.

| Native host | MSRV1.85.1 | Current stable1.99.1 |
| --- | --- | --- |
| Windows x86_64, x86_64-pc-windows-msvc | PASS | PASS |
| Linux x86_64, x86_64-unknown-linux-gnu | PASS | PASS |
| macOS arm64, aarch64-apple-darwin, hosted virtual machine | PASS | PASS |

Each of the six native jobs passes the exact full-workspace formatting check, warning-denied all-feature and no-default-feature all-target Clippy, optimized release all-target/all-feature build, and the required runtime configurations:

| Runtime command configuration | Passed functions per native job | Summaries per native job |
| --- | ---: | ---: |
| Workspace all-target/all-feature tests | 2,032 | 390 |
| Workspace default all-target tests | 2,032 | 390 |
| Workspace no-default-feature all-target tests | 2,032 | 390 |
| Workspace all-feature doctests | 1 | 5 |
| Individual xterm-web/acceptance-hooks feature | 330 | 114 |

Every test summary reports zero failed and zero ignored. Counts overlap across feature configurations and OS/toolchain jobs; they are not distinct source assertions and must not be summed as such. Host binaries with zero tests do not establish browser execution. Cargo metadata inventories the workspace's named feature. The original per-command logs remain the authority for argv, elapsed time and summaries.

The focused Miri job actually executes and passes12 cell tests plus9 lifecycle tests on native Linux with Rust1.101.0-nightly and Miri0.1.0 at revision2d20b42b88. Stable/MSRV native jobs retain failed Miri availability probes as optional tooling observations, not Miri passes. No sanitizer execution is claimed.

The separate Rust1.99.1 WASM job passes locked workspace all-feature library compilation for wasm32-unknown-unknown. It does not execute a browser.

## Remaining limits

This proves the reported native quality matrix only for frozen integratedaf047454. It does not upgrade the original failing run42012763111, later main50514541 or any later producer revision. Full original assertion mapping, browser/demo/addon execution, broader safety verification and final integrated task304 remain open. The passing CI matrix is useful native proof, not xterm.js functional equivalence.
