# APC parser port

`crates/xterm-core/src/parser/apc.rs` ports the dispatch contract from `source_repo/src/common/parser/ApcParser.ts`. `ApcParser` owns the APC entry points. Its callbacks, registration identities, UTF-16 accumulation and reverse dispatch use the same implementation as OSC payload dispatch in `parser/osc.rs` and `parser/osc_handler.rs`. This sharing is intentional because those upstream algorithms are identical after the OSC numeric identifier has been parsed. APC bypasses OSC identifier parsing.

## API mapping

| Upstream | Rust | Representation |
| --- | --- | --- |
| `ApcParser` | `ApcParser::new` or `Default` | Portable, single-threaded core |
| `registerHandler` | `register_handler` | `u32` packed identifiers, shared `Rc<RefCell<dyn ApcHandlerTrait>>` identities |
| `clearHandler` | `clear_handler` | Clears future lookup, not the active shared handler list |
| Registration `dispose` | `DisposableHandle::dispose` | Explicit and repeatable; dropping a handle does not unregister |
| `setHandlerFallback` | `set_handler_fallback` | `ApcAction::{Start, Put, End}` replaces string action and optional payload |
| `dispose`, `reset`, `start` | Same snake_case methods | Reset aborts unfinished callbacks; dispose removes registrations and fallback |
| `put(data, start, end)` | `put(&data[start..end])` | Borrowed UTF-32 slice; fallback payloads preserve UTF-16 units |
| `end(success, promiseResult)` | `end(success, previous_result)` | Initial previous result is `true`; returns a pending future or `None` |
| `IApcHandler` | `ApcHandlerTrait` | Same start, put and end callbacks |
| `ApcHandler` | `ApcHandler::new` | Whole-payload callback receives owned `Vec<u16>` |
| Private mutable payload limit in upstream tests | `ApcHandler::with_limit` | Instance-local limit, no global mutation |

The default limit is 10,000,000 UTF-16 units, taken from upstream `ParserConstants.PAYLOAD_LIMIT`, not the stale 50 MB comments elsewhere in upstream. Supplementary code points count as two units. Lone surrogates remain representable.

Rust futures are lazy. Await each pending end result before resuming. Reset retains asynchronous abort cleanup; the integration adapter must drive the futures returned by `take_cleanup`. A previously returned end future remains owned by its caller and is not silently cancelled. This explicit executor contract replaces JavaScript's automatic promise scheduling without choosing a runtime for the portable core. An outer terminal/browser adapter must honor it before claiming integrated event-order equivalence.

## Original assertion mapping

All 20 original `it` bodies run with their assertions unchanged under `scripts/check-apc.mjs`. Only imports/exports and TypeScript syntax change for Node evaluation. Nested setup/cleanup hooks run and asynchronous tests are awaited. The script does not edit `source_repo`.

| Original group and cases | Rust assertions in `tests/apc.rs` |
| --- | --- |
| handler registration, setApcHandler | `set_apc_handler` |
| handler registration, clearApcHandler | `clear_apc_handler` |
| handler registration, addApcHandler | `add_apc_handler` |
| handler registration, addApcHandler with return false | `add_apc_handler_return_false` |
| handler registration, dispose handlers | `dispose_handlers` |
| factory, called once on end true; not called on end false | `factory_end_success_and_abort_sync_async`, synchronous iterations |
| factory, disposable | `factory_dispose_sync_async`, synchronous iteration |
| factory, respect return false | `factory_return_false_sync_async`, synchronous iteration |
| factory, exact payload limit; limit plus one | `factory_payload_limit_and_overflow`, both iterations |
| mixed sync/async, two layouts, first handles or all bubble, four cases | `mixed_sync_async_dispatch`, four iterations |
| async factory, called once; not called on abort | `factory_end_success_and_abort_sync_async`, asynchronous iterations |
| async factory, disposable; respect return false | `factory_dispose_sync_async` and `factory_return_false_sync_async`, asynchronous iterations |
| reset during payload, then reuse | `reset_aborts_active_handlers` |

The 11 Rust tests preserve the original expected callback sequences and payloads while parameterizing repeated cases. `tests/apc_edges.rs` adds five tests for idle fallback, UTF-16 payload boundaries, clearing an active registration, asynchronous abort cleanup, and reset during a pending callback.

`tests/apc_differential.rs` compares complete callback and fallback logs on 256 operation traces, each with 80 operations. The seed is `0xa9c2025`. Operations include registration, token disposal, clear, reset, restart, success/abort, disposal, idle calls, empty chunks and non-BMP/lone-surrogate input. The committed fixture stores the full traces and expected logs. A failure prints its seed, index and operations. This corpus covers synchronous dispatch; the original four mixed asynchronous cases and focused reset tests cover asynchronous continuation separately.

## Commands and evidence

```sh
node scripts/check-apc.mjs
cargo test -p xterm-core --test apc --test apc_edges --test apc_differential
cargo clippy -p xterm-core --test apc --test apc_edges --test apc_differential -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/parser/apc.rs crates/xterm-core/tests/apc.rs crates/xterm-core/tests/apc_edges.rs crates/xterm-core/tests/apc_differential.rs
```

Observed locally on Windows with Rust 1.85.1 and Node v24.15.0. Original baseline passed 20/20 with no skips. Rust passed 17 test functions: 11 assertion-migration tests, five edge tests, and one differential corpus test covering 20,480 operations. Focused Clippy and formatting passed. Raw output and versions are under `results/apc/`; `oracle.json` records the source and fixture SHA-256 values and every original case name.

These results verify the APC subparser only. They do not demonstrate outer escape-parser integration, WASM/browser operation, native Linux/macOS runtime behavior, or full terminal equivalence.
