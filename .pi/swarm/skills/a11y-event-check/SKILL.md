---
name: a11y-event-check
description: Use when checking lossless accessibility character delivery or callback exception identity through native and WASM event adapters.
---

Read docs/a11y-events.md and docs/acceptance/a11y-events.md. The producer/enum already use UTF16; do not perform a duplicate String migration. Tests cover existing behavior, not new original-suite counts.

Under the current build-slot policy run scoped `cargo test -p xterm-core --test a11y_utf16`, targeted Clippy -Dwarnings, and `node scripts/a11y-event-oracle.mjs`. The oracle uses shared loadTerminalReference and unchanged InputHandler.parse for7producer+6erroridentity cases. Inspect source.json and exact leased receipts under results/a11y-events/status.json.

For actual WASM conversion set WASM_TEST=lib and invoke scripts/browser-host-check.mjs in a WASM lease, with CARGO/PATH pointed at the existing toolchain. Fivecfg(test,wasm)unit tests must execute in Chromium/Firefox/WebKit, including actualWeakBrowserServices forwarding, not only generic converter tests. Compile-only or resource-blocked attempts are not passes.

Honor resource guards, shared target/profiles and current no-release local policy. Queue release checks through coordinated quality work. Retain every failed attempt; never use a snapshot or lossy String conversion to hide a UTF16 or thrown-value identity gap.
