---
name: serialize-scrollback-array
description: Use when verifying primitive numeric browser ANSI scrollback errors against the unchanged source, with source-restoring negative controls and exact engine diagnostics.
---

# ANSI scrollback error checks

Read docs/serialize-scrollback-array.md. Run results/serialize-scrollback-array/reproduce.mjs from an owned O: worktree with `quality`, then `proof`, then `audit`. Reuse the existing reference dependencies and wasm-bindgen0.2.100; do not install dependencies or change source assertions. The proof swaps only the owned Rust serializer source, snapshots the owned package/build pointer, requires fresh actual web artifacts and restores entry state on failure. Its baseline fbea6472 is the task498 probe-only commit, not a general latest-main baseline. Reconcile that commit before reusing this negative control on newer code.

Get current309 grants before heavy commands. Quality uses one sequential Cargo-j2 compiler lane. Proof builds sequentially and runs the three-engine serializer matrix plus unchanged source/Rust suites, so it requires one browser/compiler lane within the existing cap. Never restore the retired build queue. Put TEMP, TMP and TMPDIR under the owned worktree's tmp/runtime and set CARGO_TARGET_DIR explicitly. Every command needs a bounded timeout.

Inspect raw exit codes, exact error messages, counts, source/package identity and pins. Baseline failures must remain visible; do not normalize engine diagnostics or filter assertions. Full346/322 parity remains separate. After independent review, preserve source/evidence commits and clean only the owned O: worktree after link/process/reference checks.
