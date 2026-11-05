---
name: render-scheduler-check
description: Use when changing RenderDebouncer or BrowserFrameScheduler host request/cancel dispatch, to check reentry, late handle reads and original exceptions against unchanged source.
---

# Render scheduler checks

Read docs/render-scheduler.md and reuse browser-host-check. Use an owned O: worktree and TEMP/TMP/target, obtain sequential compiler/browser grants and Cargo-j2. Keep source_repo/assertions unchanged and use existing esbuild/Playwright/wasm-bindgen0.2.100.

Run scripts/render-scheduler-source.mjs. The bundled source-only corpus must match the original SHA and all66programs. This is not Rust evidence. Run native lib/debounce/debounce_errors/debounce_differential/render_scheduler tests and strict native Clippy. Run strict WASM lib/render_scheduler_browser/render_service_browser Clippy, then WASM_TEST=render_scheduler_browser and render_service_browser through scripts/browser-host-check.mjs.

Preserve66source/Rust virtual-window programs plus actualWindow RAF ordering per engine. Cancellation resolves window+method before reading the latest pending ID. A cleared ID is undefined and a replacement ID must be canceled. Dispose only clears its pending slot after success. WindowSource remains typed/infallible, so these tests do not prove throwing window-source getters or raw JS row values.

Negative control must restore exclusive browser request dispatch and fail nested scheduling. Restore exact original bytes in finally, verify SHA, rerun complete final matrix/native/lint. Preserve commands/logs/artifact/source pins before owned-link/process-safe cleanup. Full317/318/301 stay open.
