---
name: browser-decoration-check
description: Use when changing live DOM buffer decorations, render callback geometry/lifetime or browser microtask line-cache synchronization.
---

# Browser decoration checks

Read docs/browser-decoration.md and docs/build-slots.md. Reuse the controlled WASM lane with the shared target directory and unchanged profiles. Do not launch another large build below the1GiB reserve.

```sh
node scripts/build-slot.mjs --owner decoration-source --lane wasm --timeout-ms 120000 -- node scripts/buffer-decoration-oracle.mjs
WASM_TEST=wasm_buffer_decoration node scripts/build-slot.mjs --owner decoration-wasm --lane wasm --timeout-ms 420000 -- node scripts/browser-host-check.mjs
```

Use `--generate` on the source oracle only for deliberate fixture generation. It observes the actual unchanged BufferDecorationRenderer and common services in real Chromium DOM. Default execution checks expected bytes without overwriting them. Source BufferDecorationRenderer has no original test file; report the eight derived observation steps separately from original tests.

The Rust test target uses actual browser DOM/RAF/microtasks and includes an exported Rust Terminal integration assertion. The reusable host executes Chromium, Firefox and WebKit, closes all browsers and its server, and retains exact WASM/binding hashes plus failed outputs under results/browser-host. Component passes do not replace full browser/addon E2E, overview canvas rendering or native platform evidence.
