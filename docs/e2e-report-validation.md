# Reject incomplete E2E success reports

Task 583 continues swarm2 on base `14139449bae2792afb019f42e8438b074938cfe7`. Overall equivalence remains NOT PROVEN.

## Defect and change

Both actual entrypoints, `scripts/e2e-upstream.mjs` and `scripts/demo-e2e.mjs`, previously accepted an exit-zero command when a raw report contained one passing case beside an unrun case, beforeAll fallout, an unknown final result status, or an unexpected failure. A malformed non-array `errors` field could also make the catch block throw before retaining a summary.

The shared `validateTestReport` now checks report arrays and result statuses, requires every discovered runtime case to finish, rejects unexpected results and suite errors, and requires actual execution. Discovery remains explicitly unexecuted. Original declared skips and expected failures remain valid. `testCounts` retains its existing categories, including failed expected-failure cases, and both entrypoints count before validation so rejected reports retain partial accounting. They also keep their independent command-status, interruption, source/build/fixture identity and server-shutdown checks.

Demo fixture identity now includes the shared validator module. No terminal engine, original source, assertion, deadline, browser project, package guard or supported skip changed.

## Reproduce the controls

From an isolated checkout with the existing reference esbuild dependency available:

```sh
node --experimental-vm-modules --test scripts/e2e/*.test.mjs
node --test scripts/e2e/*.test.mjs
```

The first command exposes every VM entrypoint case individually. The default command uses one bounded VM child for those cases because Node requires its experimental VM-module flag. Neither command launches a browser. The entrypoint controls execute the real runner source with injected command, filesystem, build and server boundaries. Their cleanup assertions check the runner's close call, not actual browser-process shutdown.

Windows x64, Node v24.15.0 results:

| Scope | Tests | Passed | Failed | Skipped/cancelled |
| --- | ---: | ---: | ---: | ---: |
| Original runners, final injected entrypoint controls | 14 | 4 | 10 | 0 |
| Fixed complete E2E Node suite, direct VM cases | 37 | 37 | 0 | 0 |
| Fixed complete E2E Node suite, default command | 24 parent | 24 parent | 0 | 0 |

The default command's single VM-child parent executes 14 passing entrypoint cases. Do not add its child cases to the direct 37-test total. Ten old failures comprise eight false-success controls and two malformed-error summary controls, one set for each actual entrypoint. The four unchanged positive/command-failure controls pass before and after.

`results/e2e-report583/` retains all raw attempts and source/log hashes. Initial Windows path matching in the injected filesystem was wrong, so the first original/fixed controls include six upstream setup failures. The corrected controls and final runs are separate files; those initial failures were not hidden. The old-runner reproduction helper snapshots and restores only the three owned runner files. It belongs in an isolated owned checkout, not a shared integration tree.

The validator also accepted the retained complete core574 raw report unchanged: 1,434 discovered, 1,194 executed/passed, 240 original skips, zero failed/unrun. This was a read-only report check from `archive/Chagatai566574-complete`, not a new browser run. Original reference hashes were checked for all 776 files.

No Cargo, new browser matrix or native OS runtime ran in this task. Full E01-E05, all-addon/application matrices, GPU behavior and native Windows/Linux/macOS acceptance remain open under 327/330 and their implementation tasks. Task327 now depends on integration576 for a fresh matching stable package.

## Temporary work

The creator timed out during the first checkout. No Git/Node worker remained, and its owned empty stale index lock was inspected before removal. The owned index was recovered and a scripts-only sparse checkout completed. The partially written tracked PNG was restored from the exact base, then sparse state reapplied to a clean tree before any edit. No shared tree or source was reset.

The only disposable root is `O:/xterm-rust-swarm-16/tmp/worktrees/TylerE05`. It contains one dependency junction at `source_repo/node_modules` to the original P: dependency directory. Unlink that junction before whole-root removal. Cleanup completion is reported separately after independent review and durable archive custody; this paragraph is not a deletion receipt.
