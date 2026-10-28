# Browser logging adapter

Task 145 owns `crates/xterm-web/src/logger.rs`, `tests/wasm_logger.rs`, and the small dynamic-method extension in the core logging policy. Xavier owns the browser Terminal constructor; Athena owns key-aware option conversion. This work closes a real integration gap: passing a raw JsValue as the logger option did not produce the typed sink expected by the Rust log service, and the browser's default endpoint still collected logs in native memory.

## Contracts and wiring

- `logger::from_js` wraps a nonnullish JS logger with a typed CustomLogger plus its original JS backing value. Generic option conversion still sees that original value. Truthiness and strict equality follow the backing value, including false and invalid primitive logger values. Null/undefined continue selecting the default endpoint.
- Options constructor and live setter use this conversion only for the logger key. Generic conversion remains available for property access and coercion and must not recursively wrap logger values.
- The browser constructor installs `logger::console_sink()` through `LogService::set_console_sink`. It does not replace the public logger option or change prefix policy. Native engine defaults remain unchanged.
- Core filtering runs first. Dynamic host method access and binding run next, before lazy parameters. The core's optional `LogSink::prepare` returns a captured call for dynamic methods; fixed native sinks retain their ordinary statically bound `log` implementation.
- Custom methods read the method and its bind property, then read the live logger option again as the bind receiver. A method getter may replace the option during that operation. The raw options store supplies this live receiver through `from_js_with_options`; `from_js` handles standalone values and setters before storage. Getter and overridden-bind behavior remain native JS operations. The bound method is captured before parameter callbacks can replace it. Call plumbing uses `method.call(console, message, ...parameters)`, preserving the original context rules.
- Trace/debug default to console.log, info/warn/error select their corresponding console methods. Default output retains `xterm.js: `; custom output has no prefix. Console method selection occurs before lazy parameters; the current console receiver is used when invoking the selected method.
- JS string messages and code-unit arrays remain lossless. A lone surrogate is not converted through Rust String. Optional JS values retain their original object identity through normal option conversion.
- Getter, bind and call failures retain their exact thrown JsValue in JsCallbackError. LogError and CoreError expose their source chains so the browser API can rethrow that original value rather than a new string-only Error.
- Replacing the logger or logLevel is observed on the next admitted call. Filtered calls never access a logger method, even when that getter would throw. Setting an invalid logger does not throw eagerly when no logging occurs.

## Tests

`tests/log_dynamic_binding.rs` has three native tests. They distinguish binding, lazy evaluation and invocation order; assert capture before mutation and prefix lookup after mutation; preserve getter-error identity; and reject method access for filtered levels.

`tests/wasm_logger.rs` has six real browser Terminal tests plus one direct browser-adapter timing test. They exercise constructor and live logger options, get/set identity, receiver identity, empty and malformed UTF-16 input, exact trace units, off/trace updates, getter and method exception identity, default console routing/prefixes, null fallback, false logger values with deferred TypeError, nested input/state reads from a logger with exact event ordering, and a method getter that replaces the logger before binding. The direct adapter test distinguishes default console callable failure after lazy parameters from custom bind failure before them. Both preserve native JS TypeError. These checks do not substitute a JS terminal engine.

Commands:

```sh
cargo test -p xterm-core --test log_dynamic_binding
WASM_TEST=wasm_logger node scripts/browser-host-check.mjs
```

The shared browser checker uses wasm-bindgen-test in Chromium, Firefox and WebKit, records per-browser versions/outcomes and shuts down its HTTP server and browser processes. It checks disk headroom before builds. Local Windows results do not prove native Linux/macOS execution.

Execution is still in progress. Current core/WASM builds are temporarily failing during coordinated Buffer/terminal migrations, not because those missing implementations are external blockers. Retained attempts include `results/core-services/browser-hook-tests*.log` and `tmp/willie-logger-wasm-check.log`. No successful browser or full-equivalence claim is made until the observed results are recorded.
