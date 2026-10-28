# Utility addon implementation and evidence

Overall equivalence is NOT PROVEN. Task42 and criteria156-160 are still in progress. The controlling branch checklist is `docs/utility-addons-acceptance.md`; exact source and declaration mappings are in `docs/utility-addons-criteria-map.json`.

## Implemented

`xterm-addons` has portable attachment lifecycle, OSC52 dispatch with immediate and fallible asynchronous providers, UTF-8/base64 conversion, CSS fit calculation, wrapped-line URL computation and callback/provider lifecycle. Rust strings that represent JS strings use UTF-16 units. Link indices and cell mapping use those units rather than bytes or Unicode scalar indices. Browser RegExp and WHATWG URL are standard built-ins called by the Rust browser adapter, not the upstream terminal engine.

`native_utility` connects attach and clipboard to actual `CoreTerminal` and public headless terminal state. Callback registration uses cloneable OSC registrars so replies and reentrant registration do not borrow a parsing engine. Clipboard errors retain their typed cause, including a browser JsValue. A pending provider pauses parsing; its reply must precede the next query. Native socket writes are synchronous and expose Pending instead of pretending to implement JS queue timing. The browser `BrowserAttachTerminal` uses the real queued browser writer instead.

Browser adapter files are `crates/xterm-web/src/utility_addons/`. WebSocket handles ArrayBuffer and lossless JS string messages. Fit reads CSS through the owning document's window. The web-link provider bridges Rust link ranges and callbacks to the real BrowserLinkifier. Exported AttachAddon, ClipboardAddon and Base64 classes are implemented in `bindings.rs`; their full exported-terminal test execution remains pending current browser integration builds. FitAddon/WebLinksAddon exported classes still need live renderer/terminal accessors and final binding integration.

## Observed execution

- `node scripts/utility-addons-oracle.mjs` executes unchanged TypeScript source method bodies using Node24.15.0. Fixed seed1112888393 generates1307 base64 cases,512 fit cases,24 clipboard timing cases and1765 window/cell URL cases. Source and corpus hashes are in `results/utility-addons/oracle.json`.
- The oracle also extracts all247 country suffixes,495 hostname cases and3465 path assertions from the unchanged source web-links suite. The generated WASM test is present; do not count these assertions as executed until its retained browser log says so.
- Native policy tests `utility_attach`, `utility_clipboard`, `utility_oracle` passed9 tests. The live parser tests `utility_live_terminal` passed3 tests, including every byte split of an OSC52 set/query stream and an asynchronous clipboard reply before DA1. `results/utility-addons/native.log` and `live-native.log` retain results. Later logs may also contain explicitly failed shared-build attempts, which are not runtime passes.
- `node scripts/utility-addon-browser-check.mjs` observed3 Rust component tests passing on Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The test starts a real WebSocket peer and verifies incoming string/ArrayBuffer data, live DOM CSS fit and browser RegExp/URL behavior. Exact run artifacts are under `results/browser-host/wasm_utility_addons/2026-10-27T23-52-06-670Z`. These3 tests are components, not the complete upstream addon suites or a rendered terminal equivalence claim.
- Independently reviewed task104 handler failures and ran46 status-addon tests, including10 actual live terminal tests. `results/utility-addons/status-independent-review.log` records this independent review. It is not utility-addon completion evidence.

## Remaining checks

Run `wasm_utility_links` for actual Rust buffer wide/combining ranges, all generated hostname assertions and real DOM hover-to-underline/disposal. Run `wasm_utility_bindings` for actual Terminal constructor, addon activation, OSC52 providers and original JS exception identity. Finish default navigator clipboard permissions and failure cases, exported FitAddon/WebLinksAddon wiring, and the unchanged original browser suites against the Rust build. Verify actual rendered underlines and sizing with real font metrics, not injected cell measurements.

Original baseline owner Alaric reports source FitAddon pixel-width bounds failing on Chromium/Firefox while WebKit passes. Retain those original failures and diagnose environment/font metrics; do not weaken source bounds or claim Rust passed an unrun test. Upstream WebKit clipboard skips stay separately reported.

Global fmt, warning-free Clippy, native OS matrices, WASM builds and full workspace gates remain required. Shared API migrations have caused failed attempts, retained in `results/utility-addons/`. Observe fresh complete commands before replacing those statuses. Missing implementation and shared-task waiting are work, not external blockers.

## Commands

Use the utility-addons-check and browser-host-check skills. Keep the shared `.cargo/config.toml` profiles and target directory; check disk before large builds. No source method/assertion edits or browser engine delegation are allowed.
