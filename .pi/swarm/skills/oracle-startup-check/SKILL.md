---
name: oracle-startup-check
description: Use when checking that source-oracle startup failures cannot leave a stale passing receipt, including missing dependencies and blocked temporary directories.
---

# Oracle startup checks

Run from the repository root:

```sh
node scripts/check-oracle-startup.mjs
```

Use `--reflow`, `--serialize` or `--charset` to select individual generators. The default covers all three, with two failure scenarios each.

The checker copies each real oracle body into an isolated `tmp/oracle-startup/` root and changes only its root assignment and local helper import paths. It creates a previous passing receipt, then forces either a broken reference/dependency or a regular file where the temporary directory should be. A deliberately broken fake esbuild package prevents global package fallback, so no compiler or bundler service starts.

A passing check requires an actual nonzero process exit, no timeout or signal, and a receipt changed to the generator's explicit failed/incomplete state. A crash alone cannot pass. No shared source, fixture or evidence file is changed by the probe. The full report includes source hashes, subprocess errors and final receipts under `results/oracle-startup/`.

The check caught an actual reflow ordering bug where `mkdir(temp)` ran before invalidating the old pass. Generator receipts must be written before temporary-directory and external-dependency setup. Oracle source/test hashes, actual command exit status and matching fixture hashes remain required for positive evidence. These six negative probes do not prove the full terminal implementation or replace positive baseline/differential runs.
