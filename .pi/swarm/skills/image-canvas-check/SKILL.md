---
name: image-canvas-check
description: Use when changing WASM image canvas operations, storage resources, cropping, scaling, placeholders or browser pixel evidence.
---

# Image canvas checks

Run `node scripts/image-canvas-check.mjs` from the worktree with Cargo on PATH. It reuses `browser-host-check.mjs`, compiles the unchanged source renderer into an oracle prelude, and runs the Rust image canvas tests in Chromium, Firefox and WebKit. Keep original source and assertions unchanged. See `docs/image-canvas.md` for scope and commands.

If unresolved shared browser producers prevent compilation, `--isolated-component` explicitly compiles the same production files/tests through a temporary manifest. This mode uses actual core/addon/storage crates, not mocks of the engine. Do not call its result a workspace or full-addon pass. Use `CARGO_TARGET_DIR="$PWD/target"` for follow-up Cargo commands against that manifest to avoid another target directory.

Prerequisites are wasm32-unknown-unknown, wasm-bindgen0.2.100, and matching source esbuild/sixel/Playwright/browser installations. `SOURCE_REPO`, `REFERENCE_NODE_MODULES`, `WASM_BINDGEN`, `PLAYWRIGHT_MODULE` and `BROWSERS` override locations/selection. Missing build artifacts, missing browsers and zero-test exports fail. Browser/server cleanup comes from the existing runner.

Retain `results/image-canvas/verification.json`, source hashes and the browser report/logs. Seed0x272389 runs128 full-pixel draw/extract pairs per browser. Scoped Clippy uses the generated manifest, target wasm32 and `-D warnings`; direct rustfmt checks target the new files. Required workspace failures must stay visible. Terminal lifecycle hooks remain task127 and full protocol/addon wiring task38 until separately implemented and verified.
