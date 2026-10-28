---
name: mouse-fields-check
description: Use when testing shared Rust mouse builtin policies against raw JavaScript fields, accessors, coercion and thrown values without waiting for the complete browser Root.
---

Run `node scripts/mouse-fields-oracle.mjs` to verify the checked-in 1,125-case unchanged-source fixture. Never use `--generate` to hide a mismatch. The shared test-only host module retains exact field/coercion logs and thrown sentinel identity. Original tests and source are not modified.

Run the five core mouse targets in docs/mouse-fields.md, then set `WASM_PACKAGE=xterm-wasm-support WASM_TEST=wasm_mouse_fields` and run `node scripts/browser-host-check.mjs`. Its default package remains xterm-web. Supply WASM_BINDGEN/PLAYWRIGHT_MODULE for external installations and NODE_PATH for an older worktree's shared source dependencies. Use all three default browser engines, not just Chromium. The test reports one Rust assertion containing all 1,125 raw cases.

The shared policies run in Rust; JavaScript glue contains only language/property operators. Check exact UTF-16, read/write/coercion order, mutation and failure identity. Run focused formatting and warning-free Clippy for native/WASM libraries. Raw registry/receiver/live mode integration remains task171 and is not established by this component pass.
