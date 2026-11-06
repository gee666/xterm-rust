---
name: selection-teardown-check
description: Use when changing selection target assignment or drag teardown to verify source callback ordering, errors and reentrant selection fields.
---

# Selection teardown checks

Run from an isolated worktree with an owned target and scratch paths on the volume authorized by the current brief. Obtain team compiler/browser grants first. Cargo uses `-j2`; do not start more concurrent work than the team allocation permits.

1. `node scripts/selection-teardown-methods.mjs` executes eight unchanged original SelectionService method cases. Set `SOURCE_ROOT` to the retained original reference with its installed TypeScript dependencies.
2. `node scripts/selection-teardown-source.mjs` compares eighteen shared public Terminal cases in Chromium, Firefox and WebKit against the retained fixture. The runner verifies exact embedded source-map contents and closes each browser/server. `--generate` is only for first fixture creation or an investigated source change. Never regenerate to hide a Rust failure.
3. `cargo test --locked -j2 -p xterm-web --test selection_teardown --test selection_service --test selection_service_differential --test selection_model` checks portable state and the retained selection corpus.
4. `WASM_TEST=wasm_selection_teardown node scripts/browser-host-check.mjs` runs the identical public trace through actual Rust/WASM Terminal exports. Set `WASM_BINDGEN` to the locked 0.2.100 executable and `PLAYWRIGHT_MODULE` to the existing installation. Use owned O: `TEMP`, `TMP` and `TMPDIR` if that remains the current brief.
5. Run warning-denied native and WASM library/test Clippy, focused formatting, and retained `wasm_selection`/`wasm_terminal_selection` browser tests. Preserve failing control logs before restoring the exact production bytes.

Reports live under `results/selection-teardown` and the existing browser-host result paths. Fixture cases are custom differential assertions, not original upstream test definitions. This scoped check does not prove full selection, link, DOM removal lifetime or platform equivalence.
