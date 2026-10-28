---
name: rust-browser-e2e
description: Use when running unchanged original Playwright suites against the actual Rust WASM terminal and addons, or checking their fixture server.
---

# Rust browser suite execution

Use an isolated agent worktree and its own Cargo target. Requirements are the Rust WASM target, wasm-bindgen CLI 0.2.100, and the reference tree's installed Playwright and esbuild dependencies. Set `WASM_BINDGEN` to an existing matching CLI if it is not on PATH. Do not copy credentials.

```sh
node --test scripts/e2e/*.test.mjs
node scripts/e2e-upstream.mjs --suite addon-fit --list
node scripts/e2e-upstream.mjs --suite addon-fit --project Chromium
node scripts/e2e-upstream.mjs --suite core --timeout-ms 1200000
node scripts/demo-e2e.mjs --case demo-05
node scripts/demo-e2e.mjs --case demo-06
```

The default runtime command builds the debug Rust library. `--no-build` is only for an already built matching library. `--grep PATTERN` explicitly selects a smaller scope. Omitting `--project` retains every original browser project.

The runner reads original TypeScript tests directly through Playwright. It changes the fixture's engine script and server address, not assertions, browser projects or test settings. A classic deferred script compiles actual WASM asynchronously. A per-navigation pending image delays the load event until bindings are published. This avoids Chromium's 8 MB synchronous compilation limit without changing original assertions or browser flags. Its embedded WASM bundle is test transport, not the production distribution or a size benchmark.

Logs, Playwright JSON and failures are under `results/e2e/upstream/`. Configs and child temporary files are under root `tmp/e2e/`. Zero discovered tests, zero executed runtime cases, build errors and test errors fail. Discovery is not execution. Server and child cleanup are bounded; no global build lease is needed.
