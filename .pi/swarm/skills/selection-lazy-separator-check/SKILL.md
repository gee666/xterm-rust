---
name: selection-lazy-separator-check
description: Use when checking that non-word selection input does not coerce the unused wordSeparator option, while ordinary word selection remains live.
---

Use an owned O: worktree and an allocated compiler/browser slot for this swarm run. Set `SOURCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to existing read-only dependencies and the locked 0.2.100 CLI. Keep Cargo jobs at two and temporary environment paths inside the owned root.

```sh
node scripts/selection-lazy-separator-source.mjs
WASM_TEST=wasm_selection_lazy_separator node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection_separator_host node scripts/browser-host-check.mjs
WASM_TEST=wasm_selection node scripts/browser-host-check.mjs
```

The first command checks twelve public cases against a committed source-generated fixture, including normal word-string controls. Only `--generate` writes that fixture. The public Rust target has one test expanding those cases. The host target has two SelectionDom tests for ignored separator failures, link priority, normal word lookup and changed separators during word drag. They do not prove dynamic source-timed `indexOf` semantics.

Keep old-code negative controls, source-map checks, raw results and artifact hashes under `results/selection-lazy-separator`. Do not alter or weaken source assertions. See `docs/selection-lazy-separator.md` for exact scope and remaining gaps.
