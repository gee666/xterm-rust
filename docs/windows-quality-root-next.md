# Windows quality root after the NEXT source union

Task 584 repairs a real formatting-path budget failure in task 576's combined checkout. It does not establish hosted Windows quality success or full port equivalence.

The unchanged 57 quality-runner controls initially reported 56 passes and one failure. The complete 550-target inventory needed 32,876 quoted file-argument characters at the modeled hosted `D:/a/q` root. Windows allows 32,767 characters, and the existing test reserves another 512 for the executable and non-file arguments. The exact failure remains in `results/integration576/initial/quality-controls.log`.

The workflow now selects `../../q` relative to hosted `RUNNER_TEMP`, rather than `../q`. With the existing hosted layout `D:/a/_temp`, this is a real physical checkout at `D:/q`. It is not a drive alias, junction or argument-list substitution. Setup and cleanup use the identical expression. Existing-path refusal, exact ownership-marker checking, reparse-point refusal, source preservation and log-before-cleanup ordering are unchanged. The native gate commands, target discovery, command-line limit and 512-character reserve are unchanged.

Local lifecycle fixtures put their simulated runner temporary directory at `fixture/runner/runner-temp`. The same two-parent expression then selects `fixture/q`, so tests never create or remove a directory outside their owned fixture. All local scratch stays beneath the owned O: worktree's `tmp/` directory.

## Observed checks

On Windows with Node 24.15.0 and Cargo 1.85.1:

- All 57 original quality-runner controls passed after the path change, including all nine setup/cleanup lifecycle subtests. The raw output is `results/windows-root584/controls.log`.
- Current Cargo metadata after accepted 569 and 579 contains 553 edition-2021 targets. The evidence-only model reports 33,057 file-argument characters at `D:/a/q`, which fails even before considering the reserve, and 31,951 at `D:/q`. The latter leaves 304 characters after the unchanged 512-character reserve.
- `git diff --check` passed for the change.

Reproduce the controls with the unchanged command in the workflow's `Verify the quality runner` step. Reproduce the complete current metadata model with:

```sh
node results/windows-root584/model.mjs
```

The model records its scope explicitly. It does not claim to have run hosted `cargo fmt`, and it does not replace the workflow's actual physical-root budget record or exact `cargo fmt --all -- --check` command. The long local O: worktree still has the previously recorded aggregate OS206 failure. Separate member-format passes are not an aggregate waiver.

The remaining modeled margin is small. New targets must continue to pass the full inventory tripwire. Do not omit targets, shrink the reserve or label a model as native CI runtime evidence. Tasks 563 and 304 retain the hosted checkpoint and final stabilized-source obligations. No push or workflow dispatch was performed for this repair.
