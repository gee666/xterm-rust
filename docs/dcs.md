# DCS parser

`crates/xterm-core/src/parser/dcs.rs` ports `src/common/parser/DcsParser.ts` from the supplied xterm.js reference. `dcs_handler.rs` ports its buffered callback adapter. Both retain the upstream MIT attribution.

## Rust binding

| Upstream | Rust |
| --- | --- |
| `DcsParser` | `parser::dcs::DcsParser` |
| `IDcsHandler` | `DcsHandlerTrait` behind `Rc<RefCell<dyn DcsHandlerTrait>>` |
| `registerHandler`, returned disposable | `register_handler`, explicit `DisposableHandle::dispose` |
| `clearHandler`, `setHandlerFallback` | `clear_handler`, `set_handler_fallback` |
| `hook`, `put`, `unhook`, `reset`, `dispose` | Same snake-case names; dispatch methods return `Result<_, DispatchError>` |
| Packed identifier | `i32`, retaining JavaScript bitwise packing |
| Fallback action and data | `DcsAction::{Hook(Params), Put(Vec<u16>), Unhook(bool)}` |
| `Uint32Array`, start/end | A checked caller-selected `&[u32]` slice |
| Callback string | `&[u16]`, preserving lone UTF-16 surrogates |
| `DcsHandler` | `DcsHandler::new` with the source 10,000,000 UTF-16-unit limit |
| Test-only static limit mutation | `DcsHandler::with_limit` on each instance |
| `boolean` or `Promise<boolean>` | Shared `parser::handler::HandlerResult` |

`DcsParser::registrar()` provides a weak, callback-safe `DcsRegistrar`. Its register/clear methods return `None` only after parser drop; explicit parser disposal leaves it reusable. OSC, DCS and ESC/CSI use the same `handler::registry` implementation. Tokens capture weak list and handler identities so a callback can own its disposal token without a reference cycle. Tokens do not extend a detached callback's native lifetime.

Registration handles have explicit disposal, not unregister-on-drop. Repeated disposal removes the first matching identity each time, including duplicate registrations. Clearing a registration lookup does not detach the list already selected by `hook`. Disposing the parser removes handlers without invoking their cleanup, as upstream does. Borrowing ends before list callbacks so handlers can dispose registration handles during dispatch. If that removes an index still needed by reverse traversal, Rust returns `DispatchError::ActiveHandlerRemoved`, matching the upstream TypeError rather than silently skipping it. The four hook/put/unhook/reset mutation counterexamples are retained in both languages.

Handlers run newest first. Every active handler receives hook and payload. A successful unhook result causes older handlers to receive `unhook(false)` for cleanup. A false result keeps bubbling. Pending cleanup must finish before invoking the next older handler. Reset skips handlers already unhooked before a pending result.

The buffered adapter clones nonzero or multi-value parameters. It deliberately canonicalizes a single zero or empty parameter list to `[0]`, including the source behavior of dropping subparameters on that zero. Payload overflow clears the builder and prevents callback invocation for the entire sequence. Supplementary codepoints count as two UTF-16 units.

## Scheduling

On the initial `unhook(success, previous_result)`, pass `true` as `previous_result`. Propagate its `DispatchError` before examining the result. `Ok(None)` means complete. For a returned pending `HandlerResult`, await its `resolve()` method, then call `unhook` again with the same success flag and resolved boolean. `PendingFallible` represents JS promise rejection. On error, stop without resuming ordinary dispatch; explicit reset is the recovery boundary. `Failed` represents a synchronous callback exception and stops bubbling. Both error paths retain buffered data and Params, as upstream does. Typed event callback errors retain their actual cause.

Do not process subsequent terminal input while ordinary unhook dispatch is paused.

JavaScript promises start eagerly. Rust futures are lazy. `reset` and the implicit reset inside `hook` therefore retain asynchronous cleanup rather than dropping it. Call `take_cleanup` and schedule every returned future on the embedding application's single-threaded executor. The queue remains available after parser disposal. The application owns futures already returned by `unhook`; reset does not cancel them. Callback implementations must perform any eager prefix before returning `Pending`. The portable core does not select a runtime or spawn background threads. This is an explicit language binding contract, not a claim that an unpolled future executes itself.

Directly reborrowing the same mutable handler during its callback is not supported by the `RefCell` binding. Registration disposal and mutation through `DcsRegistrar` do not reborrow the parser and are supported. Real escape-parser integration is tested separately below. Full terminal/browser scheduling integration still requires its own evidence.

OSC, DCS and APC share `HandlerResult`, the UTF-32 conversion and limited UTF-16 builder. Their small dispatch loops stay protocol-local because their entry/reset states differ. Sharing the callback type avoids an executor dependency while keeping protocol state visible.

## Verification

Run from the repository root with Node.js 24.15 or later and Rust 1.85.1 or later:

```sh
node scripts/check-dcs.mjs
cargo test -p xterm-core --test dcs
cargo clippy -p xterm-core --test dcs -- -D warnings
```

The oracle uses Node's TypeScript transformer and the shared `scripts/parser-oracle-common.mjs` runner, removing only imports and exports before executing in one VM realm. It runs all 20 upstream test bodies and assertions unchanged, including their before/after hooks. A small registration layer replaces Mocha and Node's strict assertion replaces Chai. It does not edit `source_repo`. `results/dcs/oracle.json` records source SHA-256 hashes, runtime version, each test outcome and corpus hash.

The oracle also emits 512 traces with seed `0xdcc512`. Each has four sequences and five payload chunks per sequence. They cover reversed dispatch, mixed sync/async handler results, empty chunks, malformed UTF-32 values, supplementary and lone-surrogate conversion, registration during payload, disposal during payload, clearing active lists, success/abort, and synchronous reset. The generated TSV is protocol test data and intentionally exceeds the ordinary source-file line-size convention. The Rust replay compares every report in order, not just the final result. Regenerate it only with `node scripts/check-dcs.mjs --generate`.

`tests/dcs.rs` retains all five upstream registration cases, four mixed async cases and the reset case. `tests/dcs_cases/callback.rs` retains six factory cases; its first four each run both sync and async, covering the four upstream async factory cases too. Added cases check canonicalization, cloned params, UTF-16 overflow, duplicate identity disposal, clear-during-payload, reset while paused, and retained asynchronous reset cleanup. `tests/dcs_cases/oracle.rs` replays the generated traces.

`node scripts/verify-dcs.mjs` captures exact commands, input hashes, toolchain versions and raw oracle/test/fmt/Clippy logs. Each invocation reserves a distinct `results/dcs/run-NNNN/` directory, writes an incomplete false summary before commands, retains failures, and rejects input changes during verification. `results/dcs/verification.json` mirrors the current invocation; use its `runDirectory` and hashes when citing evidence. The detailed source-branch checklist is `docs/acceptance/dcs.md`.

The same verifier runs the real outer parser checks in `tests/escape_dcs.rs`. `scripts/dcs-integration-oracle.mjs` loads fresh read-only reference modules through the shared TypeScript/VM loader and compares 657 traces from 15 streams, three handler configurations, every split boundary, 1,695 decoded chunks and 402 pending continuations. It also retains four explicit reset/rejection/exception/mutation regressions. Callback order, payload batch boundaries, final parser state, collect, Params/subparams and preceding-join state are compared. The checklist is `docs/acceptance/dcs-integration.md`. Regenerate that fixture only with `node scripts/dcs-integration-oracle.mjs --generate`.

These commands run locally on native Windows. Counts and current pass/failure status are in each run's JSON and logs, not inferred from file existence or an older summary. Parser-only evidence does not establish complete terminal integration, browser behavior, native Linux/macOS results or full xterm equivalence.
