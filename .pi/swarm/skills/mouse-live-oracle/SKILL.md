---
name: mouse-live-oracle
description: Use when validating MouseService live event getters, callback mutations and short-circuit host reads against unchanged JavaScript methods and Rust DOM bindings.
---

# Mouse live-read verification

Reuse browser-mouse-check for the complete original15-test baseline and pure coordinate/wheel/report corpus. This skill adds observer cases, not replacement assertions.

Run `node scripts/mouse-live-oracle.mjs` from repository root with Node24 supporting stripTypeScriptTypes. It invokes no Cargo, bundler or browser and writes under20MiB, so the source-only check requires no build lease. It executes unchanged MouseService/MouseCoords/getCoordsRelativeToElement/getWindow methods after annotation/import/DI erasure, compares the ten-case fixture byte-for-byte and records exact reference hashes in results/mouse-live/source.json. Use --generate only for reviewed source changes.

Run actual Rust browser assertions in the agent's automation-created worktree:

```
node scripts/check-mouse-live.mjs
```

The suite currently plans24 Rust tests, including the ten source-driven ordered traces and live getter/error/recursion/ownerDocument cases. Inspect actual reported counts; a build failure means tests unrun. The checker runs the source fixture and actual browser target; keep evidence short: command, result and concrete failures. Source metadata groups plain render fields to the corresponding MouseReadHost methods. Complete original suites, real gesture recognition and full Terminal integration remain separate gates.

Use the worktree's own small debug target and shared downloaded registry. Check disk and delete that target after merging. Do not bypass merge build/test failures. See docs/mouse-live-reads.md and docs/browser-mouse-dom.md for scope and remaining evidence.
