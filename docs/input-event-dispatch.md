# InputEvent dispatch

The browser InputEvent listener now evaluates observable properties in the order used by `source_repo/src/browser/CoreBrowserTerminal.ts::_inputEvent`. The first `data` read is a truthiness check, not a string conversion. `inputType`, `composed`, the current keydown state, `screenReaderMode` and keypress duplicate suppression follow in source order. An admitted event clears dead-key state before reading `data` again for delivery.

`KeyboardDispatch::input_source` shares this policy with native callers. `TextInputEvent` remains the immutable UTF-16 representation for callers without getters. The DOM adapter uses fallible Reflect reads, so thrown values retain their identity and getters can synchronously dispatch more keyboard/input events without a held model borrow. A second empty string is sent as empty text rather than replaced by the first value. Lone UTF-16 surrogates are retained.

## Scope and remaining gap

This fixes typed text delivery and the raw first-read guard, including null, undefined, empty text, false, zero, NaN, objects and booleans. It does not establish full keyboard, IME, focus or accessibility parity.

An admitted decorated event whose **second** data getter returns null, undefined or another non-string still cannot be delivered faithfully by the typed CoreService `&[u16]` data contract. Upstream can pass that raw value to onData, whereas the current DOM adapter rejects it. Do not convert it to an empty string or claim this raw-payload behavior is supported. Closing this gap needs raw browser event plumbing with source-faithful disableStdin, scroll, user-input, log and callback ordering. Full input/accessibility closure task319 retains this requirement.

## Verification

- `cargo test -p xterm-web --test input_event_getters --test keyboard_dispatch_callbacks -j2` checks short-circuiting, repeated/empty/UTF-16 payloads, getter reentry, dead-key state before a thrown second getter, duplicate suppression and immutable API compatibility.
- `node --test scripts/input-event-fixture.test.mjs` verifies that the browser oracle fixture matches the unchanged upstream method body, including comments, apart from line endings and surrounding whitespace. It prints source and fixture SHA-256 hashes.
- `WASM_TEST=wasm_input_event node scripts/browser-host-check.mjs` executes real DOM events through the Rust listener. Its guard matrix compares 288 combinations with the original method body, followed by thrown-getter prefixes and identity, changed/empty second data and synchronous keyboard reentry.
- Rerun `wasm_input`, `wasm_input_callbacks` and `wasm_keyboard_live` through the same browser runner to cover existing clipboard/IME/keyboard behavior. These are Rust component tests, not the complete original Terminal browser suite.

The stale component hosts in `support/input_dom.rs` and `support/input_mouse_host.rs` were recovered from retained `swarm/Aisha` commit a6f69be rather than rewritten. They implement the producer's existing fine-grained live option/mouse getters. `wasm_input.rs` also drops a now-unnecessary mutable binding. No upstream source or assertion was changed.

Two existing Rust test-fixture defects surfaced during the rerun. Firefox's synthetic ClipboardEvent constructor discards the DataTransfer initializer's contents, reproduced with Playwright without loading Rust. The component helper now explicitly supplies that clipboardData property; all paste/copy assertions remain unchanged. The changing-keyCode IME case used to expect two reads for 230 then Shift. The unchanged source actually reads three times, testing CapsLock, 229 and Shift. It now runs the original CompositionHelper.keydown body on the same event, asserts the source's three reads and compares Rust with that result. This corrects a Rust test expectation; it does not modify an upstream assertion or disguise a production fix.

Fresh commands, outcomes and remaining verification gaps belong under `results/input-event/`; browser runner output remains under `results/browser-host/`. Reports must distinguish test functions from the 288 expanded matrix cases.
