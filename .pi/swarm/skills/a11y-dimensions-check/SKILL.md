---
name: a11y-dimensions-check
description: Use when changing accessibility row dimensions, CSS property assignment, live row traversal or their callback/error ordering.
---

Read `docs/accessibility-dimensions.md`. Work in an owned O: worktree with literal O: TEMP/TMP/TMPDIR and Cargo -j2. Obtain current compiler/browser grants. No new dependencies are required.

Light checks:

- `node scripts/accessibility-dimensions-source.mjs` verifies three unchanged reference method bodies and their combined test module. Use `--write` only after inspecting source boundaries and shared cases.
- `node --test scripts/accessibility-dimensions-source.test.mjs` checks eight source-method scenarios on controlled fields. This is not DOM or Rust runtime proof.
- `node scripts/audit-accessibility-dimensions547.mjs` verifies retained source/evidence pins, raw logs/counts, negative controls, expected diagnostics, exact source restoration and reference hashes. `--runtime` additionally requires every recorded disposable artifact and native executable to remain present and match its hash. It is intended for review before cleanup.

With granted resources, run the native accessibility tests and warning-denied focused native/WASM checks from the document. Run `CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_accessibility node scripts/browser-host-check.mjs` and the corresponding `wasm_accessibility_terminal` target using the existing matching wasm-bindgen 0.2.100 and Playwright installation. The runner closes its browsers/server in finally.

Current suites contain 33 component functions and eight actual-Terminal functions. Eight plus one are new source-derived regressions, not migrated upstream tests. The four warnings per component run come from existing invalid-selection controls. Reject other console errors/warnings. Exact old-source controls retain seven component assertion errors and one Terminal assertion error, not unexplained browser errors.

Preserve initial failures and exact old-source controls. Restore owned production bytes in finally and verify their hashes before the final fixed matrix. Keep source and raw evidence; remove only inspected owned build/browser/scratch artifacts. Full input/accessibility closure, raw nonnumeric font-size behavior, full native quality/platform matrices and overall equivalence remain separate requirements.
