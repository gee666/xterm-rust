# Reentrant render scheduling

Task468 uses the shared FrameScheduler hooks introduced by task454 in RenderDebouncer. BrowserFrameScheduler snapshots the current-window source and callback before invoking either window resolution or requestAnimationFrame. Cancellation likewise releases the scheduler borrow before resolving the current window or calling cancelAnimationFrame. A late-handle hook reads the pending ID after resolving the window and cancellation property, matching JavaScript argument evaluation. The browser passes undefined if those lookups cleared the pending frame. No dependency or unsafe code is added.

The source assigns the pending frame only after a successful request. A nested request can therefore schedule another frame, but the successful outer request still overwrites the pending handle. Cancellation checks for a pending frame before entering the host but resolves its argument only after window and method lookup. Reentry during either lookup can clear or replace the ID. The current pending slot is cleared only after successful cancellation. Nested replacement is cleared on successful outer cancellation and retained on failure. Rows and callbacks survive disposal as before. Last-owner drop still has exclusive access and reports cleanup failures.

## Verification scope

Five native regressions cover nested requests, repeated cancellation, successful and failed cancellation after replacement, and failed outer requests after nested scheduling. Existing debounce tests and the retained seeded source trace remain unchanged.

The test-only oracle bundles the unchanged source RenderDebouncer with a synchronous virtual-window corpus. Its 66 programs compare request and cancellation property getters and calls, current-window source reentry, refresh/callback/disposal/frame delivery, window replacement, and thrown-object identity against the actual Rust BrowserRenderDebouncer. Virtual frames are delivered explicitly and never timed. A second browser test uses the real browser Window and RAF to verify canceled rows survive and render before zero-time refresh callbacks. No JS terminal engine executes Rust behavior. WindowSource remains an infallible typed native callback. Throwing window-source/getter parity and raw JS row values are not established by this task.

Full317/318 browser/root/rendering closure, raw scrolling task301 and overall equivalence remain NOT PROVEN. These component tests do not establish the full workspace, Linux/macOS runtime matrix or unchanged full upstream suites.

Run with TEMP/TMP/Cargo target in an owned O: worktree, matching installed wasm-bindgen0.2.100, existing esbuild/Playwright, bounded resource grants and Cargo-j2:

```text
node scripts/render-scheduler-source.mjs
cargo test -p xterm-web --all-features --lib --test debounce --test debounce_errors --test debounce_differential --test render_scheduler -j2
cargo clippy -p xterm-web --all-features --lib --test debounce --test debounce_errors --test debounce_differential --test render_scheduler -j2 -- -D warnings
cargo clippy -p xterm-web --all-features --target wasm32-unknown-unknown --lib --test render_scheduler_browser --test render_service_browser -j2 -- -D warnings
WASM_TEST=render_scheduler_browser node scripts/browser-host-check.mjs
WASM_TEST=render_service_browser node scripts/browser-host-check.mjs
```

All 74 focused Windows native tests passed with no failures, ignored tests or filtering. Strict focused native/WASM Clippy and both WASM test builds passed. Initial and restored final Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 runs each passed the 66-program differential and actualWindow RAF test. Existing render_service_browser passed its one test per engine.

The exclusive-request negative control failed exactly one Chromium Rust test during nested scheduling. A second early-handle negative failed exactly one test when nested disposal during source-window resolution cleared the pending ID. Original production bytes were restored in finally after each control and SHA-256 matched. Complete final browser, native and strict-lint checks passed afterward. No assertions or watchdogs were weakened.

Exact cargo fmt --all -- --check still fails with Windows OS206 before formatting. All five member-specific format checks passed. This is not a full workspace formatting pass.

Commands, raw logs, source and native executable pins, browser reports, restoration records and generated artifact hashes are retained under results/render-scheduler. Negative compiler target paths were reused by subsequent builds, while their per-run generated WASM/bindings remain available through review. Source-only corpus execution is recorded separately and never counted as Rust coverage.
