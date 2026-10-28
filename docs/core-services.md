# Core and log services

The source modules are `source_repo/src/common/services/CoreService.ts`, `CoreService.test.ts`, `LogService.ts` and `Services.ts`. Acceptance cases are enumerated in `docs/core-service-acceptance.md` and criteria 22 and 23. This is scoped service evidence, not full terminal equivalence.

## Rust contracts

`CoreService::new` accepts shared `OptionsService` and `LogService` instances plus a callback that reads the active buffer's `ybase` and `ydisp`. The engine can activate buffers without replacing this callback. It is not called when stdin is disabled. Options are read at the same points as upstream. A scroll listener disabling stdin does not cancel the already admitted data event.

Modes, DEC modes and Kitty keyboard state are public typed `RefCell` values. Keep borrows short and release them before triggering callbacks. Cursor initialization uses `CursorInitialization`: `get` performs a truthiness check, `set(bool)` stores a typed boolean, and `value` retains the exact constructor option value. Null/undefined become false; nonboolean primitives and object identity are not discarded. Reset replaces modes and Kitty storage but preserves both cursor flags and subscriptions.

`on_data` and `on_binary` carry UTF-16 units, not Rust UTF-8 text. Data, binary, embedded NUL and lone surrogates are preserved. Trace logging records UTF-16 code units. Event dispatch failures return `CoreError` and stop subsequent callbacks. `CoreError::source` and `LogError::source` preserve traversal to the exact host callback error for browser bindings, without converting it to a new string-only error. Explicit disposal and Drop remove event listeners. As upstream, calling a disposed service can still log; it does not recreate disposed events.

`LogService::new` accepts the options service and an adapter-owned `LogSink`. The sink receives the logging level, console method route, UTF-16 message and typed optional parameters. Default console output prefixes `xterm.js: `. Custom loggers are installed using `OptionValue::object(CustomLogger(Rc<dyn LogSink>))` and receive no prefix. The host implements actual stdout/console behavior; the shared core does not depend on browser or OS APIs. A custom sink dispatches its five methods using `record.level`.

`LogParameter::Lazy` covers infallible callbacks, and `TryLazy` returns recoverable errors. Dynamic sinks use `prepare` to capture a host method before lazy parameters run; fixed native sinks keep their statically bound `log` call. `set_console_sink` installs an adapter-owned default endpoint without modifying the custom logger option or prefix policy. Parameters run once, left to right, only after a message passes its threshold. The target method is selected before lazy parameters run; prefix policy is read afterward, preserving the source behavior when a parameter replaces the logger. Sink errors propagate. `LogError::Host` retains an opaque native/browser error by shared identity rather than forcing a UTF-8 string conversion. Unsupported logger payloads return `InvalidLogger`, not a successful fallback.

Log-level property conversion is fallible and uses the JS string hint. Constructor conversion errors return immediately. A live update uses `OptionsService::try_on_specific_option_change`: the option commits before its callback, but a failed conversion returns the original error from `set`, leaves the previous threshold intact and stops later listeners. Disposal unregisters this callback without disabling direct logging methods, matching source lifecycle behavior.

Do not create strong self-retaining callback cycles. Capture a Weak pointer where a callback is stored by the object it calls. No service borrow crosses a callback or sink invocation.

## Test and oracle mapping

- `tests/core_service.rs` has eleven tests. It retains the three original constructor/reset cases and their four assertions. Added cases cover all mode fields, independent instances, a 32-combination disabled/user/scroll/viewport matrix with ordered data and binary events, exact UTF-16, live option truthiness, recursive sends, errors and disposal. Raw cursor initializer identity and original typed event callback errors are tested separately.
- `tests/log_dynamic_binding.rs` adds three tests for dynamic binding before lazy parameters, getter failures and filtered method access. Together with the eleven core and nine base logging tests, the current native unit total is twenty-three.
- `tests/log_service.rs` has nine tests. It covers all 30 threshold/message pairs, console methods, custom replacement, malformed UTF-16, lazy order and failure, option coercion and immediate errors, raw writes, disposal, recursive sinks and sink errors.
- `examples/core_service_oracle.rs` reads operation traces and executes Rust only.
- `scripts/check-core-services.mjs` executes the unchanged source service implementations and original test bodies. It erases module/type syntax and constructor dependency decorators in memory. Minimal mocks preserve the exact constructor fields consumed by the three source tests. No source test file is rewritten. There is no original LogService test suite in this reference.
- Seed `0x434f5245` produces 20,000 random operations after deterministic input/viewport/UTF-16 and log threshold matrices. The final run compares 20,616 complete mode/cursor/Kitty snapshots and ordered event/log traces. It also checks live ToPropertyKey failure and disposal. Reentrancy and fallible lazy/sink cases have explicit Rust tests rather than being miscounted as randomized differential coverage.

## Observed commands

Native Windows x64, Node v24.15.0, rustc 1.85.1. Commands ran with the repository's disk-saving debug/test configuration intact:

```sh
node scripts/check-core-services.mjs
cargo test -p xterm-core --test core_service --test log_service --test service_injection
cargo clippy -p xterm-core --lib --test core_service --test log_service --test service_injection --example core_service_oracle -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/core_service.rs crates/xterm-core/src/log_service.rs crates/xterm-core/tests/core_service.rs crates/xterm-core/tests/log_service.rs crates/xterm-core/examples/core_service_oracle.rs
```

The initial scoped commands passed for nineteen core/log tests. After the Event error migration, `final-tests3.log` records twenty passing core/log tests and `final-oracle3.log` records 20,616 passing differential operations. `final-clippy3.log` records a shared headless disposal Result warning, reported to its owner and corrected. The final current-tree scoped Clippy and formatting runs passed, retained in `final-clippy6.log` and `final-fmt.log`. `results/core-services/summary.json` records seed, counts, versions, upstream and implementation hashes. `oracle4.log`, `tests5.log`, `clippy4.log` and `fmt.log` retain observed output. The combined Rust command also ran nine service-injection tests for independent review of task 65; they are not counted as core/log tests. Earlier `attempt1-build.log`, `oracle2.log`, `tests3.log`, `clippy2.log`, `oracle3.log`, `tests4.log` and `clippy3.log` retain integration compilation failures encountered while other modules changed.

No Linux/macOS runtime, browser logging sink, full terminal differential, workspace-wide lint/test gate or real engine service integration is proved by these commands. Task 68 owns actual engine integration tests. Its six native cases and runner are documented in `docs/core-services-integration.md`. Task 145 separately owns the browser logger and live option conversion, documented in `docs/browser-logger.md`. Overall equivalence remains NOT PROVEN.
