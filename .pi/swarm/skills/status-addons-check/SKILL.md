---
name: status-addons-check
description: Use when validating program-status OSC7501 and progress OSC9 native/WASM behavior against xterm.js.
---

Run from an isolated worktree. Keep source_repo unchanged.

```
node --experimental-transform-types scripts/status-addons-oracle.mjs
cargo test -p xterm-addons --test status_addons_differential --test program_status_store --test program_status_addon --test progress_upstream --test status_live_terminal
WASM_TEST=wasm_status_addons node scripts/browser-host-check.mjs
WASM_TEST=wasm_status_coercion node scripts/browser-host-check.mjs
```

The source oracle runs 38 original pure cases and retains 2,215 parser comparisons plus 2,048 progress operations at seed0x750109. Native targets contain 50 cases. The browser suites contain 21 binding cases and 9 coercion/identity cases, including original assertions and review regressions. Use existing WASM_BINDGEN/PLAYWRIGHT_MODULE overrides when required.

Results are in results/status-addons/ and results/browser-host/. Report the command and outcome briefly. These addon checks do not establish whole-terminal or native OS equivalence. See docs/status-addons-acceptance.md for scope.
