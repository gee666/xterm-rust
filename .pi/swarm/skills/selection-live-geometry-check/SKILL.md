---
name: selection-live-geometry-check
description: Use when changing selection callback geometry or selection-change deduplication after resize or active-buffer changes.
---

# Selection live geometry checks

Read docs/selection-live-geometry.md. Run in an isolated worktree with the current team grants and owned temporary paths.

- `node scripts/selection-live-geometry-methods.mjs` runs four unchanged-source teardown/refresh cases.
- `node scripts/selection-live-geometry-source.mjs` checks eight public resize/throw/repeated-selection cases in all three engines. Set SOURCE_ROOT and PLAYWRIGHT_MODULE to existing installations. Generate a fixture only for initial creation or an investigated source change.
- `cargo test --locked -j2 -p xterm-web --test selection_live_geometry --test selection_teardown --test selection_service --test selection_model --test selection_service_differential` retains native assertions.
- `WASM_TEST=wasm_selection_live_geometry node scripts/browser-host-check.mjs` runs the same eight cases on real Rust/WASM. Set locked WASM_BINDGEN and reuse the installed browser tools.
- Retain wasm_selection_teardown, wasm_selection and wasm_terminal_selection, strict native/WASM Clippy, member/full formatting and full xterm-web tests. Preserve old-code failures before exact restoration.

Reports live under results/selection-live-geometry. These scoped cases do not establish complete selection/link or cross-platform equivalence.
