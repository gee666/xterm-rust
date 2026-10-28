# Browser continuation

Worktree: `C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Xavier`, branch `swarm/Xavier`. It contains the combined producer stack. Nothing has been merged into main from this branch. Do not restore old main WIP or copy unrelated files.

## Verified

- `cargo check -p xterm-web --target wasm32-unknown-unknown --features acceptance-hooks -j2` passes without warnings.
- `WASM_TEST=unicode_addon_bindings node scripts/browser-host-check.mjs` passes2/2 in Chromium148, Firefox150, WebKit26.4. Task213 accepted by swarm_review;7 native Unicode addon tests also pass.
- Node server/pinning helper tests pass9/9.

The0.2.100 wasm-bindgen CLI is available through `WASM_BINDGEN=P:/workshop/Windows_Worshop/oira666/xterm-rust/tmp/wasm-tools/wasm-bindgen-0.2.100-x86_64-pc-windows-msvc/wasm-bindgen.exe`. Put Cargo and Git on PATH as described in docs/workspace.md. Use this worktree's private target, not build-slots.

## What is connected

The actual Rust terminal is wrapped by queued WriteBuffer scheduling. ByteView implements the retained parse-source interface; bounded reads happen when the parser requests them, not at enqueue. Root owns real CoreBrowserService, shared character measurement, RenderService, active backend replacement, Viewport, selection, input, localization and addon management. Public events use the shared dispatch context and source-shaped disposal tokens. Public modes snapshots have just been added.

Root factory wiring is in terminal/browser_bindings.rs, browser_services.rs and viewport_host.rs. Central scrolling is in viewport_services.rs. RenderService owns frame scheduling, synchronized-output and current backend state. Do not reintroduce a second fixed-DOM render pipeline or native fallback after an opened viewport exists.

## Remaining tasks

Task31 is not complete. Run actual original browser suites and repair demonstrated failures. Sophie197 reported matching cells/buffers and6 pixel frames in one Chromium mode case, but missing public modes and Unicode getters. Modes are now implemented; the public Unicode API is still missing. Foreign provider/version UTF16 work is separately tracked. Complete public marker/decoration wrappers and remaining receiver/coercion gaps with their owners. Full equivalence remains NOT PROVEN.

Task118 still needs real browser write FIFO, callback/error/reentry/disposal and large paused-tail tests. The implementation uses source-global JS timer callbacks owned by JS GC; tokens have no destructor cancellation. Fallible cancellation propagates original exceptions. Source debug/trace view identity and detached/resized tails need direct verification.

Task183 routes InputScroll to shared selection refresh and actual viewport synchronization, distinct from public onScroll. It needs the real ED3 browser assertions against source, including no extra public scroll event. Native erase fixtures are not sufficient.

Task72 is handed to Achilles. Its producer d3e3400 is already stacked. Achilles reported25 component and3 real-Terminal accessibility cases passing in three browsers; let that owner finish review.

## Known follow-ups

- Public modes getter is in terminal/modes_api.rs. Run Sophie197 again after taking the latest commit.
- Hook `_acceptanceSnapshot` and `_acceptanceOnInternalEvent` are default-off acceptance-hooks only, read actual engine state and do not drain events.
- GC ownership cycles remain task181. Do not claim that explicit disposal or JS-owned callback transfer alone proves collection equivalence.
- Native default test gating for some WebGL/scrollbar tests still blocks other merge gates; coordinate with their owners rather than weaken the merge script.
- Source errors and raw getter phases matter, but prioritize actual failing upstream tests. No further evidence infrastructure is needed.
