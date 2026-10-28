---
name: live-keyboard-check
description: Use when checking live DOM keyboard fields, repeated getters, callback mutations and exact thrown values across Rust keyboard encoders and browser dispatch.
---

# Live keyboard getter checks

Reuse the TypeScript-erasure method from verify-keyboard and verify-kitty-keyboard. `node scripts/protocol-live-oracle.mjs` checks the reviewed Kitty/Win32 getter fixture; `--generate` regenerates it from unchanged source after review. This source-only command does not compile Rust. The corpus has 4,521 success paths and 74,171 getter-throw boundaries, 78,692 total rows. Its output and hashes are under results/protocol-live. Win32 inherited-object map/coercion behavior remains separately tracked by tasks184/185.

After obtaining a resource-managed build turn, run `cargo test -p xterm-core --test protocol_live`, the existing immutable Kitty and Win32 checks, and `cargo test -p xterm-web --test keyboard_dispatch --test keyboard_dispatch_callbacks --test keyboard_dispatch_getters`. Preserve all earlier failure logs. The legacy live oracle is separately owned by task155.

Run actual DOM tests with `WASM_TEST=wasm_keyboard_live node scripts/browser-host-check.mjs`. The shared harness builds Rust WASM and executes real wasm-bindgen-test functions in installed Chromium, Firefox and WebKit, retains versions/artifact hashes/assertion results, and closes its server/browsers. Do not call a WASM compile a browser runtime pass. Coordinate this through the shared build-slot lease, keep the shared target directory and honor the current disk pause/reserve.

Branch and remaining-gap documentation is docs/live-keyboard-event.md. Task177 owns lazy InputEvent/nullable data; task202 owns dynamically decorated fields outside declared DOM primitive types. Source and browser evidence for those tasks must not be inferred from the typed-field getter tests here.
