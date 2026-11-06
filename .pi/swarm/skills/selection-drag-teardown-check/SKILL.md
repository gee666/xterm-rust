---
name: selection-drag-teardown-check
description: Use when changing selection DOM listener ownership or drag timer cancellation to verify host order, callback identity, failure partial state and recursive selection.
---

# Selection DOM teardown checks

Use an owned isolated worktree and private target/TEMP on the volume authorized by the current brief. Obtain compiler and browser resource grants before starting. Reuse the existing Playwright installation and locked wasm-bindgen 0.2.100 tool. Cargo uses `-j2`.

1. Run `node scripts/selection-drag-teardown-source.mjs` with `SOURCE_ROOT` pointing to the retained original reference. This compares nine source Terminal traces in Chromium, Firefox and WebKit against the checked-in fixture. `--generate` is only for initial fixture creation or an investigated source change. Never regenerate to hide a Rust failure.
2. Run `WASM_TEST=wasm_selection_drag_teardown node scripts/browser-host-check.mjs`. Set `PLAYWRIGHT_MODULE`, `WASM_BINDGEN`, owned `TEMP`/`TMP`/`TMPDIR`, `CARGO_TARGET_DIR` and `CARGO_BUILD_JOBS=2`. One actual Terminal test function expands the same nine cases.
3. Rerun `wasm_selection_teardown`, `wasm_selection_live_geometry`, `wasm_selection` and `wasm_terminal_selection` through the same runner.
4. Run web all-target/all-feature native tests and warning-denied Clippy. Run WASM library and the five browser test targets through warning-denied Clippy. Run workspace formatting and member formatting separately; preserve exact failures rather than replacing a failed workspace result with a scoped pass.
5. Preserve raw traces, unchanged fixture/test/program hashes and old-production negative controls before cleanup. Browser-host artifacts are temporary. Copy concise reports/logs, pin compiler/served WASM/bindings and delete only owned temporary outputs after review.

These cases exercise real DOM drag setup and public selection calls, not fake selection services. Existing repeated-mousedown orphan interval behavior, full lifecycle closure and full family equivalence remain separate work. See `docs/selection-drag-teardown.md`.
