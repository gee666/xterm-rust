---
name: attach-disposal-check
description: Use when checking Attach disposal that synchronously reactivates the addon and appends listener handles.
---

Use the existing `utility-addons-check` commands for broad addon regressions. For the live disposal iterator specifically:

```sh
node scripts/attach-disposal-reentry-oracle.mjs
cargo test --locked -j2 -p xterm-addons --test attach_disposal_reentry --test utility_attach
WASM_TEST=wasm_attach_disposal_reentry node scripts/browser-host-check.mjs
```

The source oracle strips TypeScript types from unchanged AttachAddon methods. It verifies the full reference manifest before and after running 54 shared socket/recording-terminal scenarios. Do not use `--generate` without checking the source-derived expected traces. The WASM test runs the same scenarios through actual Rust Terminal and AttachAddon exports. It needs the normal wasm-bindgen 0.2.100 and Playwright configuration, not a WebSocket peer. Each native/WASM function expands 54 scenarios; do not add expanded scenarios to function counts.

Use a private O: worktree/target and O: TEMP/TMP, Cargo jobs2, and an explicit current resource grant. Browser suites must finish or close their owned processes before releasing the grant. Retain raw results, exact source/build pins and negative-control failures before cleaning owned build files. These checks do not establish complete Attach lifecycle, raw removal-error behavior, original complete browser suites or full-port equivalence.
