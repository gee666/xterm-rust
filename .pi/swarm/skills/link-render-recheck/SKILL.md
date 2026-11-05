---
name: link-render-recheck
description: Use when checking live link range getters during render invalidation, repeated row predicates, current-link reentry, leave callbacks and provider requery phases.
---

# Link render rechecks

Read `docs/link-render-recheck.md`. Reuse the read-only original and existing shared browser runner. Do not launch compilers or browsers without the current coordinator's allocation.

```sh
node scripts/link-render-recheck-methods.mjs
node scripts/public-link-provider-source.mjs --render-recheck
WASM_TEST=wasm_link_render_recheck node scripts/browser-host-check.mjs
```

Set SOURCE_ROOT and PLAYWRIGHT_MODULE to the supplied original. Set TEMP/TMP and CARGO_TARGET_DIR inside the owned O: worktree. The source methods run unchanged TypeScript in memory. The original/public and Rust/public programs are identical and assert eight cases before fixture comparison. Direct Rust controller checks add two mouseleave-reentry cases. Preserve the second-check false case: it skips leave but still re-asks providers. Preserve exceptions and numeric falsey-bound short circuits. Do not treat scoped passes as full selection/link parity.
