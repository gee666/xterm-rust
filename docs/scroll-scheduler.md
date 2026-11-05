# Reentrant scrolling schedulers

Task454 removes the scheduler borrow from the real browser queue-source callback. Viewport resolves its current browser window at every animation request. That resolution can invoke user getters and reenter the same Scrollable. The previous mutable scheduler borrow survived that callback, so a nested animation request or cancellation panicked instead of following the source's partial-state ordering.

`FrameScheduler::request_frame_shared` and `cancel_frame_shared` let callback-capable adapters copy independent handles before entering host code. Their defaults retain existing exclusive scheduler implementations and their non-reentrant contract. Scrollable uses these hooks. BrowserScrollScheduler clones its queue source and weak target before calling the queue source. Cancellation only marks the queue item canceled and invokes no external callback. Frame delivery remains asynchronous. Last-owner drop has exclusive access and cancels its outstanding handle as before.

The shared engine retains the pending-handle assignment receiver before requesting a frame. Reentrant replacement must not redirect the outer request's handle to the new operation. Cancellation likewise retains its operation/handle until the callback returns. Errors preserve their original scheduler value and the source's partially updated state. Neither hook changes raw state, clock, event or arithmetic handling.

## Verification

Four native regressions exercise nested requests, nested cancellation, request failure after replacement and cancellation failure after dimension changes. The raw differential adds 84 source/Rust programs across request/cancel callbacks, initial/replacement/reuse/tick/dimension/immediate paths, and mutation/disposal/frame/throw reactions. The actual BrowserScrollScheduler differential adds 18 synchronous queue-source programs against unchanged source, including thrown-object identity. It uses the real browser queue and weak Scrollable target, not the raw test scheduler. The queue is canceled and released without waiting for a frame.

The test-only oracle is generated from unchanged `source_repo/src/browser/scrollable/scrollable.ts` and upstream Event/Lifecycle. Generated oracle files exceed 350 lines because they bundle that intact reference and its test corpus. Production never imports them. Source-only execution and retained task443 pins are recorded separately from new Rust execution. Full task301 raw Element/state-object/event/numeric-clock-binding migration remains open. Full equivalence is NOT PROVEN.

All 83 focused Windows native tests passed. Native and WASM strict focused Clippy passed. The initial and restored final three-engine browser runs passed all 6,619 raw programs and 18 actual browser queue-source programs per engine, five Rust test functions per engine, with zero ignored or filtered tests. Engines were Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. These are scoped tests, not the full workspace or native Linux/macOS matrix.

The negative control replaced BrowserScrollScheduler's clone-before-callback dispatch with the old exclusive borrow. Chromium failed the actual browser test at `browser queue/initial/smooth`, where nested scheduling returned an error instead of succeeding. It produced exactly one failed Rust test. The original source was restored in finally and its exact SHA-256 matched the pre-mutation pin. The complete final browser matrix, 83 native tests and strict focused native/WASM Clippy then passed again. No assertion, fixture expectation or watchdog was weakened.

Reports, commands, raw browser logs, source/native executable/artifact hashes and restoration receipts are under `results/scroll-scheduler`. All five member-specific format checks passed. Exact aggregate `cargo fmt --all -- --check` still fails with Windows OS206 before formatting. This is retained separately and is not called a workspace formatting pass.

Run in an owned O: worktree with bounded grants, `TEMP`/`TMP`/Cargo target inside it and Cargo `-j2`:

```text
node scripts/raw-scroll-state-source.mjs --live
node scripts/raw-scroll-state-source.mjs --scheduler
cargo test -p xterm-web --all-features --lib --test scrollable --test viewport_controller --test scroll_clock --test scroll_scheduler -j2
cargo clippy -p xterm-web --all-features --lib --test scrollable --test viewport_controller --test scroll_clock --test scroll_scheduler -j2 -- -D warnings
cargo clippy -p xterm-web --all-features --target wasm32-unknown-unknown --lib --test raw_scrollable_browser --test browser_scroll_scheduler -j2 -- -D warnings
WASM_TEST=raw_scrollable_browser node scripts/browser-host-check.mjs
WASM_TEST=browser_scroll_scheduler node scripts/browser-host-check.mjs
```

Use the matching installed wasm-bindgen 0.2.100 CLI and existing Playwright/esbuild modules. The browser runner owns server/browser cleanup in finally. The runner's generated browser artifacts remain available through independent review, then are removed with the owned worktree. Durable reports preserve their hashes and historical paths. Source-only counts are not substituted for Rust execution.
