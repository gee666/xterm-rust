# Lossless accessibility event verification

Task 168 started from a reported scalar-only event gap. Source inspection showed that this report was stale: `terminal::print` already calls the lossless `string_from_code_point`, `TerminalEvent::AccessibilityChar` already owns `Vec<u16>`, headless raw observers forward that vector, and the browser `AccessibilityHost` clones it into its UTF-16 announcement model. No production enum migration was made.

The added tests exercise those existing paths directly. They are new regression checks, not newly discovered original upstream tests. The detailed contract is `docs/acceptance/a11y-events.md`, criteria 421-423, plus the broader charset/WASM/public-event criteria 188, 208 and 234.

## Native and unchanged-source checks

`crates/xterm-core/tests/a11y_utf16.rs` contains seven tests:

1. Whole and split UTF-16 produce one event for an astral pair and retain independent lone-surrogate events.
2. A final high surrogate stays pending across an empty chunk until the next non-low unit.
3. Custom charset replacements retain the source's first-unit rule, including empty, multiple-BMP, astral-pair, high/low surrogate and NUL-first values.
4. Disabled screenReaderMode suppresses character notifications.
5. BOM and soft hyphen follow the source drop/skip behavior.
6. Malformed UTF-8 surrogate encodings disappear without U+FFFD, while a valid astral scalar remains a pair.
7. A native callback failure retains the same Rc error object, stops before the cell/cursor write and does not implicitly replay the failed chunk after subscription disposal.

`scripts/a11y-event-oracle.mjs` reuses the shared read-only `loadTerminalReference` bundler. It executes the actual original InputHandler synchronously for seven matching producer cases. Six further checks throw an opaque object, Symbol, number, string, null or undefined and assert strict identity, the first raw payload, the unchanged pre-write cell/cursor and a successful independent recovery write. Calling InputHandler directly avoids confusing this synchronous producer contract with the original JS write queue's event-loop behavior.

On native Windows with Rust 1.85.1 and Node v24.15.0, all seven native tests and all thirteen source checks passed. Focused Clippy passed with `-D warnings`. The controlled slot receipts and precise status are in `results/a11y-events/status.json`. The source report includes every bundled source hash and outcome. No dedicated original accessibility-surrogate test count is invented.

## WASM unit coverage

`crates/xterm-web/src/terminal/a11y_event_tests.rs` is a cfg(test,wasm32) unit module. It accesses the real private callback converter without widening production Rust APIs or adding test-only JavaScript terminal exports. Five declared test functions cover:

- Actual native producer to `terminal::events::callable` to JavaScript strings, checked by raw UTF-16 iteration.
- The exact custom-charset first-unit and NUL payloads.
- Disabled announcements and pending high-surrogate behavior.
- Strict identity of six thrown JS values, including an object whose toString throws, plus pre-cell abort and recovery after disposal.
- A real browser Terminal opened into the document, forwarding through `WeakBrowserServices`' actual `AccessibilityHost::subscribe` implementation. This checks both lone-surrogate output and a JS failure across that real host adapter, so a future lossy browser-only mapping cannot pass by testing the generic converter alone.

The first WASM runner attempt failed before compilation because cargo was not on the inherited PATH. The second used an explicit CARGO path and failed in an unrelated BrowserWriteScheduler receiver-signature migration. Neither reached browser execution. All five browser unit functions remain unrun until a successful receipt exists. Subsequent build requests also encountered the 1GiB resource guard, which stopped commands before execution. No browser pass is inferred.

## Rerun commands

Use the existing shared Cargo target and resource policy. Examples from the repository root:

```text
node scripts/build-slot.mjs --owner a11y-native --lane native -- cargo test -p xterm-core --test a11y_utf16
node scripts/build-slot.mjs --owner a11y-source --lane native -- node scripts/a11y-event-oracle.mjs
node scripts/build-slot.mjs --owner a11y-clippy --lane native -- cargo clippy -p xterm-core --test a11y_utf16 -- -D warnings
```

Set `CARGO` or PATH to the existing toolchain when necessary. For the browser suite, set `WASM_TEST=lib` and run `node scripts/browser-host-check.mjs` under a WASM build slot. The runner records Chromium, Firefox and WebKit separately and must execute the five a11y functions, not merely compile them. New local release runs are not authorized by the current scoped slot policy; release checks remain explicitly unrun here and belong to coordinated quality execution.

These checks do not establish the complete AccessibilityManager DOM tree, focus/selection behavior, live-region refresh timing, native Linux/macOS runtime or whole-terminal equivalence. Those remain separate acceptance gates.
