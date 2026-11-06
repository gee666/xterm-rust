---
name: owned-browser-process-check
description: Use when verifying Windows Playwright command cleanup after normal completion, global timeout or cancellation, including detached helpers with dead parents.
---

# Owned browser process checks

Read `docs/windows-owned-browser-processes.md` and `scripts/owned-command-check.mjs` before running.

Use a private O: checkout. Set TEMP, TMP and TMPDIR under its tmp directory. No new package installation or Rust build is needed.

- Run `node --test scripts/owned-command.test.mjs` for argument/stdio/exit and platform-policy checks.
- Run `node scripts/owned-command-check.mjs` for positive and intended old-policy orphan controls. Its outer Windows Job owns every intentional negative orphan. Never invoke the negative fixture directly or kill old PIDs.
- Run the original seven-file native-quality Node command separately. Preserve failures and unchanged deadlines/assertions.
- With a coordinated browser grant, set PLAYWRIGHT_CLI to the existing matching Playwright test CLI and run `node scripts/owned-browser-check.mjs`. This launches only WebKit for cleanup testing, not a Rust terminal parity suite.

Results use unique `results/owned-process562/` attempts. Helpers remove only their freshly owned tmp directories in finally blocks. Commit raw outcomes, exact input pins and concise scope before whole-worktree cleanup. Inspect workers/links and preserve source/reference manifests before removal. Windows Job support is a native platform boundary, not an excuse to claim Linux/macOS or complete application proof.
