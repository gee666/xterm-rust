# Lossless accessibility event acceptance

Task 168 verifies the existing event pipeline. Inspection found that the producer, enum and browser adapter already use UTF-16. The suspected scalar-only gap in post 95 was stale. Do not replace working production types or claim a new migration occurred. Add missing direct delivery/error assertions and fix only actual failing behavior.

References are `source_repo/src/common/InputHandler.ts`, its `print` method, `source_repo/src/common/input/TextDecoder.ts`, `source_repo/src/common/Event.ts`, `source_repo/src/browser/AccessibilityManager.ts`, and the headless/browser terminal event adapters.

## A1. Exact producer payload

- Normal printable BMP text emits exactly its UTF-16 units. Supplementary scalars emit a two-unit payload in one character event, not two replacement characters or two independent events.
- Lone low surrogates and an unpaired high surrogate followed by a non-low unit survive streaming UTF-16 decoding. A high surrogate pending at a chunk boundary does not emit prematurely; the next chunk resolves it with the source behavior.
- Custom charset replacements intentionally use the first UTF-16 unit, even when the mapping contains a supplementary pair or multiple BMP units. Empty replacements preserve the original input character. A first replacement unit NUL remains a one-unit accessibility payload, not an empty string.
- Include high and low surrogate replacements, BMP text, empty values, embedded NUL, valid pairs and multi-unit replacements. Ordinary input BOM and soft hyphen retain source drop/skip behavior.
- With screenReaderMode false, character events are absent. With it true, each event fires before the corresponding cell/cursor mutation, and no terminal-state borrow spans a callback. Existing print-call snapshot/reentry semantics are owned by task 153, not changed here.

## A2. Native headless forwarding and callback failure

- Raw native/headless event observers receive the same owned `Vec<u16>` without a String round-trip. Neither a registered public observer nor a browser accessibility adapter may replace lone surrogates with U+FFFD.
- Returning an actual callback error preserves the same `Rc` error identity through EmitError, parser/write errors and standard error sources. Do not stringify or substitute a generic error.
- A throw on the first accessibility character aborts before writing that cell and before later characters/events. Inspect cursor and cell state after the error. Do not treat failure as handler false or successful completion.
- Disposing the throwing subscription permits subsequent independent writes according to the source parser/write lifecycle. Do not replay a failed chunk implicitly.

## A3. Actual WASM callback conversion

- Exercise the real Rust terminal/headless producer and `terminal::events::callable`/UTF-16 conversion under wasm-bindgen tests, not a substitute JavaScript terminal.
- JavaScript listeners receive exact code units, including lone surrogates, pairs, NUL and empty/multi-unit charset behavior. An additional test must open the real Rust browser Terminal and route its native output through `WeakBrowserServices`' actual `AccessibilityHost::subscribe` implementation, so a lossy browser-only mapping cannot pass by testing the generic converter alone. Native-to-JS strings use `JsString` and raw UTF-16 helpers, not wasm-bindgen Rust String conversion.
- JavaScript-thrown object, Symbol, primitive number, null and undefined values retain strict identity through `JsCallbackError` and `to_js_error`. A thrown value's toString must not be invoked merely to forward it.
- Tests must remain in a cfg(test,wasm32) unit module so private production event helpers need not become new public APIs. The browser runner must execute those tests explicitly and record engine/version/test counts.
- This event conversion does not establish the whole AccessibilityManager DOM tree, live-region refresh, keyboard echo, focus/selection or browser terminal integration. Those remain task 72 and browser end-to-end requirements.

## A4. Source and evidence

- Reuse the existing charset-terminal source oracle and snapshots for actual source producer behavior, including its custom-map first-unit output. Add targeted source callback-failure probes if existing evidence does not cover exact thrown identity/partial state.
- Retain exact commands, source/implementation hashes, discovered/mapped/executed/passed/failed/skipped/unrun counts and raw results. No original direct accessibility-surrogate test count is invented for new regression cases.
- Run native debug/release tests, targeted warning-free Clippy/fmt, WASM compilation and actual supported-engine unit tests. Missing or paused execution is unrun, not passed. Honor the coordinated disk/build pause and do not use a new target directory to evade it.
