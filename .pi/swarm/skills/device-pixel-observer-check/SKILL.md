---
name: device-pixel-observer-check
description: Use when changing the WebGL parent-window device-pixel ResizeObserver callback, observation error handling or disposal behavior.
---

# Device pixel observer checks

Read docs/device-pixel-observer.md. This is a browser utility component, not completed WebglRenderer/Addon/Root integration.

Use owned O: target/TEMP/TMP, a current measured team grant, Cargo -j2 and the browser cap. Generate the unchanged-source prelude with SOURCE_ROOT pointing at the original reference and node scripts/device-pixel-observer-oracle.mjs. It bundles DevicePixelObserver.ts and the actual supplied Lifecycle.ts without changing either. The latter's toDisposable is a bare handler, not an idempotent store.

Run native retained test webgl, focused warning-denied native/WASM lib/test Clippy, then WASM_TEST=wasm_device_pixel_observer CARGO_FEATURES=acceptance-hooks WASM_PRELUDE=<owned-prelude> node scripts/browser-host-check.mjs with CLI0.2.100 and existing Playwright. All three engines must execute all three functions and25 directed source traces. Actual parent-window observations must match source. WebKit26.4 rejects the native device-pixel option; retain its rejection/disconnect and independent default-box delivery, not a skipped pass or fallback claim.

Run node scripts/device-pixel-observer-negative.mjs only without concurrent source editors. It temporarily changes the required catch disconnect to optional, expects Chromium2PASS/1FAIL exactly at lifecycle, and restores exact source SHA in finally. Rerun the full3engine matrix and strict lint afterward.

Run node scripts/device-pixel-observer-audit.mjs for the durable receipt, or add --runtime before cleanup to verify all cached compiler/module/binding/native/snippet hashes. The producer uses --record once after evidence is final. Audit the final report's raw native source/Rust values before deleting owned outputs. Keep original hashes, source traces, failures, negative-control and final receipts. Never remove original source or another owner's runtime files. Full WebGL suites stay task306.
