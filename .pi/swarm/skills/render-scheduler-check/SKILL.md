---
name: render-scheduler-check
description: Use when changing RenderDebouncer or BrowserFrameScheduler dispatch and callback iteration, to check reentry, array identity, late handle reads and original exceptions against unchanged source.
---

# Render scheduler checks

Read docs/render-scheduler.md, docs/render-window-source.md and docs/render-callback-iteration.md. Reuse scripts/browser-host-check.mjs. Use an owned O: worktree and TEMP/TMP/target, obtain sequential compiler/browser grants and Cargo-j2. Keep source_repo/assertions unchanged and use existing esbuild/Playwright/wasm-bindgen0.2.100.

Run scripts/render-scheduler-source.mjs. The bundled source-only corpus must match the original SHA and all72 host programs,42 window-failure programs and64 nested callback programs. This is not Rust evidence. Run native lib/debounce/debounce_errors/debounce_differential/render_scheduler/render_callback_iteration tests and strict native Clippy. Run strict WASM lib/render_scheduler_browser/render_service_browser Clippy, then WASM_TEST=render_scheduler_browser and render_service_browser through the existing browser runner.

Preserve all178 source/Rust virtual-window programs plus actualWindow RAF ordering per engine. Cancellation resolves window+method before reading the latest pending ID. A cleared ID is undefined and a replacement ID must be canceled. Dispose only clears its pending slot after success. FallibleWindowSource returns original getter failures, including falsey primitives. Raw JavaScript row values remain outside these tests.

A nested successful frame replaces the current callback list, but an outer for-of iterator retains its original array and sees appends to that array. Outer success clears the current list, including post-nesting additions. Outer errors retain the current list. Browser dispatch must permit samecallback reentry, since source per-request closures can be nested by a reentrant host.

Negative controls must restore exclusive browser request dispatch, early cancellation-handle resolution, the old callback Vec and mutable browser callback as appropriate. Each must fail its directed regression. Restore exact original bytes in finally, verify SHA, and rerun complete final native/browser/lint checks. Preserve commands/logs/artifact/source pins before owned-link/process-safe cleanup. Full317/318/301 stay open.
