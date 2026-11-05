---
name: scrollbar-construction-check
description: Use when checking full Terminal scrollbar getter order, initial overview creation, or Viewport-only versus composed getter traces.
---

# Scrollbar construction checks

Read `docs/scrollbar-construction.md`. Run the original Terminal oracle with the supplied dependency tree:

```text
REFERENCE_ROOT=/path/to/source_repo node scripts/scrollbar-construction-oracle.mjs
WASM_TEST=scrollbar_construction_browser node scripts/browser-host-check.mjs
WASM_TEST=lib node scripts/browser-host-check.mjs
WASM_TEST=overview_ruler_browser node scripts/browser-host-check.mjs
WASM_TEST=overview_terminal_browser node scripts/browser-host-check.mjs
```

Both paths reuse `tests/scrollbar_construction_cases.js` with independent exact expected traces. The source oracle bundles unchanged actual source modules and records their hashes. Rust tests use the real public Terminal and check thrown-symbol identity. Initial visibility must not suppress the overview width getter. Later option updates intentionally short-circuit that getter when hidden. The old four-read expectation belongs to the isolated Viewport option literal; full Terminal construction also runs the overview creation and canvas sizing getters.

Use a private O: worktree and target. Obtain a browser allocation before running, keep Cargo at -j2, and set TEMP first, then TMP and TMPDIR. The source runner cleans its unique scratch and closes its browsers/server. Retain JSON and concise logs, not generated browser bundles. These directed tests do not establish full layout or terminal equivalence.
