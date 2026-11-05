---
name: browser-decoration-check
description: Use when changing live DOM buffer decorations, render callback geometry/lifetime or browser microtask line-cache synchronization.
---

# Browser decoration checks

Read docs/browser-decoration.md. Use an owned worktree/private target on the run's authorized temporary volume. Coordinate the current heavy-job/browser limits and preserve shared profiles. The former global build queue is retired; do not invoke it.

```sh
node scripts/buffer-decoration-oracle.mjs
CARGO_BUILD_JOBS=2 WASM_TEST=wasm_buffer_decoration node scripts/browser-host-check.mjs
CARGO_BUILD_JOBS=2 WASM_TEST=wasm_terminal_decoration node scripts/browser-host-check.mjs
```

Set PLAYWRIGHT_MODULE and WASM_BINDGEN to existing installed tools when outside the worktree. For the source oracle, reuse installed source-only dependencies and remove any owned dependency junction before whole-worktree cleanup. Every command needs a bounded timeout.

Use `--generate` on the source oracle only for deliberate fixture generation. It observes the actual unchanged BufferDecorationRenderer and common services in real Chromium DOM. Default execution checks expected bytes without overwriting them. Source BufferDecorationRenderer has no original test file; report the eight derived observation steps separately from original tests.

The Rust test targets use actual browser DOM/RAF/microtasks. The eight-test component target includes an exported Rust Terminal integration assertion. The two separate actual-Terminal regressions check pre-open cache sync/reset/disposal, public render ordering, alternate buffer, resize, opened reset and terminal disposal cancellation. The reusable host executes Chromium, Firefox and WebKit, closes all browsers and its server, and retains exact WASM/binding hashes plus failed outputs under results/browser-host. Component passes do not replace full browser/addon E2E, overview canvas rendering or native platform evidence.
