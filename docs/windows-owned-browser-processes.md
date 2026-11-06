# Owned Windows browser processes

Task562 fixes a real failure after a demo06 global timeout. A WebKitNetworkProcess outlived its parent and held the temporary worktree open. Reese's original Git removal255 and empty-directory EBUSY failure remain in the prior cleanup evidence. That cleanup is already complete and must not be replayed.

## Why a Job Object

A process-tree snapshot cannot reliably find a descendant after an intermediate parent exits. Windows `taskkill /T` only sees the tree that exists when invoked. Searching every WebKit executable, guessing from command lines or stopping a reused PID is not safe ownership.

Demo and unchanged-upstream Playwright commands now opt into `runCommand`'s `containDescendants` option. On Windows, `owned-command.mjs` starts the repository's fixed PowerShell wrapper with a JSON data plan in a private environment variable. The wrapper removes that variable before starting the requested executable. It never evaluates command text.

The small C# boundary uses the Windows Job Object API. It creates an unnamed, non-inheritable job handle, sets `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` and assigns the wrapper itself before launching the command. Every later descendant inherits the job, including detached helpers whose parents exit. No breakaway flags are enabled. Job assignment errors fail before the requested command starts.

The handle stays rooted until wrapper process exit. Windows then closes it and terminates all remaining members. Closing it earlier would kill the wrapper before it could preserve the requested executable's exit code. On normal completion, the wrapper waits for the requested executable and exits with its code. On timeout or cancellation, the existing runner stops the wrapper and closing its job also terminates orphan descendants. Ordinary commands keep their existing direct-spawn policy. Unix process-tree behavior is unchanged.

This native boundary is needed because Node does not expose Windows Job Objects. It adds no npm/Rust dependency, Rust unsafe operation, terminal platform dependency or JS terminal-engine delegation. It requires Windows with nested Job Object support and the existing Windows PowerShell/.NET Framework toolchain. Missing PowerShell, compile or job-assignment failures are failures, not silent direct-spawn fallbacks. The process-start arguments follow Windows quoting rules and preserve Unicode, empty arguments, quotes and trailing backslashes. Child stdout/stderr continue to use the runner log handles.

## Verification commands

Run from an owned O: checkout with Node24, Git and Cargo in PATH. Put TEMP, TMP and TMPDIR under its tmp directory. The PowerShell wrapper compiles its small C# adapter during startup. No Rust compilation is needed for these process tests.

```sh
node --test scripts/owned-command.test.mjs
node scripts/owned-command-check.mjs
# Existing runner contracts, separately from C# startup tests:
node --test scripts/quality-process.test.mjs scripts/quality-policy.test.mjs scripts/quality-snapshot.test.mjs scripts/quality-managed.test.mjs scripts/build-queue-retired.test.mjs scripts/target-relocation.test.mjs scripts/windows-quality-worktree.test.mjs
# After a browser resource grant, with an installed matching Playwright CLI:
PLAYWRIGHT_CLI=/absolute/path/to/@playwright/test/cli.js node scripts/owned-browser-check.mjs
```

The lifecycle controls deliberately create an intermediate process that exits and a detached descendant that remains active. Normal exit, timeout and cancellation must remove that descendant without touching a separate test-owned sentinel. Removing the attempt directory must succeed. An outer Job contains the control runner itself. The old-policy negative control disables only the inner containment; all three unchanged orphan assertions must fail. The outer Job safely removes those intentional negative-control orphans when the control runner exits. No arbitrary PID is killed. Do not run the negative fixture directly without this outer owner.

The WebKit check is browser-process cleanup evidence only, not Rust terminal behavior. It exercises normal completion, an actual Playwright global timeout and cancellation after a real network helper is observed. Each attempt records browser version, owned live process descendants, command outcome, later absence and successful attempt-directory removal. It does not change upstream assertions or run the retired global queue.

## Current results and limits

Raw results are under `results/owned-process562/`. The initial Windows quoting/stdio/exit test passed. The initial lifecycle control run passed six positive tests and produced exactly three intended old-policy failures. After adding caller-wiring/provenance assertions, the final native control run passed all seven positive tests; its old-policy control retained four passes and exactly the three intended orphan failures. Both fixture parents were removed, with identical source pins before/after. Existing57 verifier tests passed in their original separate invocation with no failures/skips/cancellations.

A first combined60-test invocation retained one existing early-exit readiness failure. It reported the unchanged2s readiness deadline rather than the expected before-readiness classification. Concurrent C# startup took6.7s instead of2.7s; the cause of the classification difference is not established. The original separate57 command then passed. Neither its assertion nor deadline was edited. This retained failure is not presented as a passing combined run.

Native Windows WebKit26.4 passed the real process-lifetime check under the coordinator's single-browser grant. Normal completion exited0. The actual Playwright12s global timeout exited1 and retained its JSON failure. Controlled cancellation after network-helper observation exited1 and retained the cancellation result. All24 captured worker, WebKitNetworkProcess, GPU, Web and console-helper process identities were absent after their commands returned. All three attempt directories and the fixture parent were removed. Total observed command duration was21.9s. The browser allocation was released after completion; no Rust build or CI action was performed.

Independent review and final owned-worktree removal are still pending. Full Rust fmt/Clippy/tests/release, other native OS runtime and full browser/application equivalence are not established by these tooling tests. Check task563/304 and the full family tasks separately. Overall equivalence remains NOT PROVEN.
