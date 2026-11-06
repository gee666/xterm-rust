---
name: clear-sparse-check
description: Use when checking public terminal clear after marker callbacks leave sparse current rows, including reset/resize/activation and exact errors.
---

Reuse `scripts/clear-sparse-oracle.mjs` and shared `terminal-snapshot.mjs`, not a replacement engine. Run `node scripts/clear-sparse-oracle.mjs` from the producer checkout with `XTERM_SOURCE_DEPENDENCIES` pointing to an existing reference dependency directory if needed. It verifies 776 immutable source files pre/post, runs both unchanged public clear tests and compares 280 callback/full-state/identity fixtures. Use `--generate` only for reviewed fixture changes. Its owned scratch is under worktree tmp and removed in finally.

Run `cargo test -j2 -p xterm-core --test clear_sparse`, existing buffer_terminal_differential and full focused core gates after the coordinator grants a compiler slot. Keep original assertions and callback sentinel identity. Test real old production code as a negative with finally restoration; do not mutate expected fixtures. Sparse current rows must remain absent in slot0 while the remaining viewport fills and scroll fires. Errors before or during scroll retain source partial state. See docs/clear-sparse.md. This is scoped evidence, not full equivalence or native OS/browser proof.
