---
name: unicode-browser-check
description: Use when checking the public Rust-backed Unicode browser view, UTF-16 version names and provider callback errors.
---

# Unicode browser checks

Run `node scripts/unicode-browser-oracle.mjs` to verify the source-derived registry fixture. Only use `--generate` after inspecting a deliberate source/trace change.

Run `WASM_TEST=unicode_public_api node scripts/browser-host-check.mjs` with the existing browser-host-check prerequisites. This executes actual Rust bindings in all three browser engines. Results are under results/browser-host/unicode_public_api. Set WASM_BINDGEN and PLAYWRIGHT_MODULE as documented by that skill when needed.

Run the five native Unicode targets listed in docs/unicode-browser-api.md. Use the unchanged Fit and Unicode addon suites through scripts/e2e-upstream.mjs for whole-package checks. They have known remaining failures, recorded in that document. Never replace missing APIs with fixture-only behavior or treat component checks as complete E2E proof.
