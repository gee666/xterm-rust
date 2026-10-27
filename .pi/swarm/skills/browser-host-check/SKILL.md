---
name: browser-host-check
description: Use when running Rust wasm-bindgen-test assertions in Chromium, Firefox and WebKit without WebDriver or upstream engine delegation.
---

# Browser WASM checks

Run `node scripts/browser-host-check.mjs` from the repository root with cargo on PATH. It compiles the existing `wasm_host` test target to the shared Cargo target folder, generates web bindings with wasm-bindgen 0.2.100, serves the generated files on an ephemeral loopback port, and executes every exported Rust test through the wasm-bindgen-test runtime in each browser. It always closes browsers and the HTTP server. Builds are bounded to180s and abort below512MiB repository disk headroom.

Requires wasm32-unknown-unknown, wasm-bindgen CLI0.2.100, source_repo's Playwright installation and browser binaries. The CLI default is the repo-local tmp/wasm-tools downloaded0.2.100 Windows release. Set WASM_BINDGEN for another installation and PLAYWRIGHT_MODULE for another matching Playwright module.

Set WASM_TEST to another xterm-web integration-test name, BROWSERS to a comma-separated browser list, CARGO for a cargo path. Do not treat a Chromium-only run as a three-browser pass.

Results and exact commands are in results/browser-host/. Generated files are under tmp/browser-host/. These are component tests, not proof of a complete exported Terminal. No source_repo engine is imported by the served page.
