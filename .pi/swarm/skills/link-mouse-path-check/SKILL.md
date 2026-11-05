---
name: link-mouse-path-check
description: Use when checking Linkifier mousemove composedPath reads, raw classList methods, live path mutation and error identity against the unchanged original and the opened Rust terminal.
---

# Link mouse path checks

Reuse the existing link-check and browser-host-check runners. Set SOURCE_ROOT to the read-only reference and PLAYWRIGHT_MODULE to its installed Playwright index.mjs. WASM_BINDGEN must name the existing 0.2.100 CLI. Use an owned O: worktree, target, TEMP and TMP, Cargo -j2, and a granted heavy/browser allocation.

```
node scripts/link-mouse-path-methods.mjs
node scripts/public-link-provider-source.mjs --mouse-path
WASM_TEST=wasm_link_mouse_path node scripts/browser-host-check.mjs
node scripts/link-mouse-path-negative.mjs
WASM_TEST=wasm_link_mouse_path node scripts/browser-host-check.mjs
```

The first command executes unchanged TypeScript _handleMouseMove in memory with only geometry and provider observation supplied by the test. The shared 26-case program also runs through an opened original Terminal and an opened Rust Terminal, using actual MouseEvents, public provider registration and DOM exception delivery. The fixture is generated from the source method trace and checked against all three original engines. Never alter it to hide a Rust mismatch.

The negative driver compiles exact production files from e64f4870, requires the intended Chromium runtime failure, and restores exact working bytes in finally with SHA-256 equality. The final fixed three-engine run is mandatory. Browser/server and compiler subprocesses are bounded and owned by existing runners. Preserve reports and pins before deleting the owned worktree; do not clean reference/dependency roots.
