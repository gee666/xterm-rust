# InputEvent dispatch

The browser InputEvent listener now evaluates observable properties in the order used by `source_repo/src/browser/CoreBrowserTerminal.ts::_inputEvent`. The first `data` read is a truthiness check, not a string conversion. `inputType`, `composed`, the current keydown state, `screenReaderMode` and keypress duplicate suppression follow in source order. An admitted event clears dead-key state before reading `data` again for delivery.

`KeyboardDispatch::input_source` shares this policy with native callers. `TextInputEvent` remains the immutable UTF-16 representation for callers without getters. The DOM adapter uses fallible Reflect reads, so thrown values retain their identity and getters can synchronously dispatch more keyboard/input events without a held model borrow. A second empty string is sent as empty text rather than replaced by the first value. Lone UTF-16 surrogates are retained.

## Raw payload delivery

Task356 closes the admitted second-read boundary. A decorated InputEvent may return null, undefined, a number, object or another non-string after its first truthy data read. The DOM listener now delivers that value unchanged through InputHost::data_value. Native text input still uses UTF-16. Neither path converts a raw payload into empty text.

CoreService::trigger_data_event_with shares disableStdin admission, the captured buffer position, scroll request and user-input effects with native input. The raw browser adapter then performs eager String-hint template conversion, Debug logging, lazy Trace split/map/charCodeAt calls and onData delivery. Logging errors and callback throws stop later effects without losing the thrown value. String conversion runs even when logging is disabled; trace callbacks run only when Trace is admitted.

One browser data emitter receives native strings and raw values in subscription order. It supports nested text/raw input and follows native disposal. The browser value is not emitted on the native UTF-16-only data API. This is an explicit language representation boundary, not a conversion of the raw value. Minimal JS glue performs dynamic property access only; Rust owns admission, logging order and dynamic method calls.

InputEventSource supplies lazy admission guards. TextInputEventSource extends it with a UTF-16 payload read for native callers; input_source_with lets the DOM adapter supply raw delivery after the same guards and dead-key update. See docs/raw-input-data.md for the exact evidence and limitations. Full keyboard, IME, focus and accessibility parity remains task319.

## Verification

- `cargo test -p xterm-web --test input_event_getters --test keyboard_dispatch_callbacks -j2` checks short-circuiting, repeated/empty/UTF-16 payloads, getter reentry, dead-key state before a thrown second getter, duplicate suppression and immutable API compatibility.
- `node --test scripts/input-event-fixture.test.mjs` verifies that the browser oracle fixture matches the unchanged upstream method body, including comments, apart from line endings and surrounding whitespace. It prints source and fixture SHA-256 hashes.
- `WASM_TEST=wasm_input_event node scripts/browser-host-check.mjs` executes real DOM events through the Rust listener. Its guard matrix compares 288 combinations with the original method body, followed by thrown-getter prefixes and identity, changed/empty second data and synchronous keyboard reentry.
- Rerun `wasm_input`, `wasm_input_callbacks` and `wasm_keyboard_live` through the same browser runner to cover existing clipboard/IME/keyboard behavior. These are Rust component tests, not the complete original Terminal browser suite.

The stale component hosts in `support/input_dom.rs` and `support/input_mouse_host.rs` were recovered from retained `swarm/Aisha` commit a6f69be rather than rewritten. They implement the producer's existing fine-grained live option/mouse getters. `wasm_input.rs` also drops a now-unnecessary mutable binding. No upstream source or assertion was changed.

Two existing Rust test-fixture defects surfaced during the rerun. Firefox's synthetic ClipboardEvent constructor discards the DataTransfer initializer's contents, reproduced with Playwright without loading Rust. The component helper now explicitly supplies that clipboardData property; all paste/copy assertions remain unchanged. The changing-keyCode IME case used to expect two reads for 230 then Shift. The unchanged source actually reads three times, testing CapsLock, 229 and Shift. It now runs the original CompositionHelper.keydown body on the same event, asserts the source's three reads and compares Rust with that result. This corrects a Rust test expectation; it does not modify an upstream assertion or disguise a production fix.

Fresh commands, outcomes and remaining verification gaps belong under `results/input-event/`; browser runner output remains under `results/browser-host/`. Reports must distinguish test functions from the 288 expanded matrix cases.
