# Escape sequence parser

`crates/xterm-core/src/parser/escape.rs` ports `source_repo/src/common/parser/EscapeSequenceParser.ts`. `escape_parse.rs` owns the synchronous action loop and CSI/executable fast paths. `escape_resume.rs` owns reverse handler dispatch and continuation. `escape_handlers.rs` validates identifiers and exposes registrations. Handler ownership is shared with the OSC/DCS/APC parsers through `parser::handler::registry`, not copied per protocol.

The parser takes borrowed UTF-32 slices. A slice replaces the upstream array plus explicit length. Empty slices are valid. It uses the exact VT500 data from task 53 and bounded Params from task 8. No browser or OS dependency is required. All parser production code is safe Rust. Default Params capacity justifies the constructor's single `expect`; input-dependent failures use typed errors.

## Calling contract

`EscapeSequenceParser::parse(data, previous_result)` returns `Result<Option<HandlerResult>, ParseError>`. `None` means that chunk finished or its error handler deliberately aborted it. `Some` owns a pending callback future. Resolve it through `HandlerResult::resolve()`. On `Ok(handled)`, call parse again with the identical chunk and `Some(handled)` until it completes. On rejection, stop without resuming. The saved frame remains available for diagnosis or explicit reset. Never turn a rejection into `false`.

Reset aborts active payload handlers, clears parameters/collect/join state and preserves registrations. If paused, the next parse skips the saved codepoint and continues from the initial state. This is upstream behavior, not permission to supply a different chunk. Missing continuation values produce a sticky improper-continuation error until reset. Callback failures during resumed dispatch retain the saved frame instead of silently replaying bytes.

Rust futures are lazy. Drain and drive `take_cleanup()` before resolving the current pending callback and at completed-chunk executor turns. These are the reset/start abort callbacks that JavaScript promises schedule immediately. A terminal adapter must define executor ordering explicitly. See the OSC restart regression in `results/osc/restart-cleanup-counterexample.json`.

`preceding_join_state` is an `Rc<Cell<u32>>`. A PRINT callback can update it without aliasing the mutably borrowed parser. Execute/finished dispatch resets it as upstream does. `pause_info()` exposes kind, chunk position, handler position and transition for diagnostics and exact source test assertions.

## Registration and errors

`FunctionIdentifier` uses borrowed strings for optional prefix/intermediates and the final byte. It retains source validation order and error text, including UTF-16 length checks before ASCII range checks. CSI/DCS final bytes start at 0x40; ESC/APC at 0x30. APC ignores prefixes.

CSI/ESC callbacks share `CommandHandler`, an `Rc<RefCell<dyn FnMut(&Params) -> HandlerResult>>`; ESC ignores its Params argument. Callback identity is stable. Newest registrations run first. True stops, false bubbles and synchronous exhaustion calls fallback. Upstream asynchronous false exhaustion does not call fallback, and Rust preserves that distinction.

Disposable tokens require explicit disposal. Dropping a token does not unregister. Weak captures avoid a callback-own-token cycle. Cloneable `csi_registrar()` and `esc_registrar()` handles support callback-driven registration without borrowing terminal state or the parser loop. They report a dropped parser explicitly and never keep it alive. The OSC parser provides its own equivalent handle.

`set_print_handler_checked` and `set_execute_handler_checked` let native/browser backends stop on recoverable errors without panicking. Original infallible setters remain available. Synchronous command errors use `HandlerResult::Failed`. `DispatchError::Callback` retains the actual event error, including a browser-owned error wrapper. Typed errors are distinct from a handler returning false.

`set_action_boundary_handler` is a Rust adapter hook. It delivers queued events after completed synchronous actions and before later codepoints. Pending actions are flushed by the adapter after parse returns, keeping future ownership explicit even if an event listener fails. Core parsing does not select an executor, hold a terminal-state borrow across a callback or spawn detached tasks.

Custom tables retain upstream fast-path bypasses. A missing entry in a short table maps to IGNORE/GROUND, matching JavaScript's undefined-to-zero bitwise coercion. Invalid caller-defined PARAM mappings return typed errors instead of accepting arbitrary digit values. A custom string-end transition that combines into an invalid state also returns an error. Normal VT500 input never takes these boundary errors. Rust state enums cannot represent arbitrary invalid numeric states. These invalid-configuration boundaries are explicit, not evidence of arbitrary custom-table equivalence.

## Verification

Run the commands in `.pi/swarm/skills/escape-parser-check/SKILL.md` from the repository root. Source hashes, deterministic seed and counts are in `results/escape-parser/oracle.json`. The seed is `0xe5ca9e01`. The fixture has 2,822 exhaustive state/class cases and 512 randomized stream traces, totaling 92,668 parse calls. Each snapshot compares state, collect, full Params/subparams, preceding join state and complete ordered PRINT/execute/CSI/ESC/OSC/DCS/APC/error callback logs. Payload chunk boundaries, malformed bytes, lone-surrogate/wide UTF-32 values, numeric limits and single-codepoint chunks are included. Retained fixture rows reproduce any mismatch. The generated text fixture is intentionally larger than the normal source-file line limit.

The unchanged original suite has 161 tests, 161 passed and zero pending/failed in `results/escape-parser/original.json`. Task 90 maps 147 synchronous original cases to shared Rust assertions and 21 generated literal examples. The other 14 original async cases map to seven shared Rust functions, with exact original full/chunked callback results and pause tuples, in `results/escape-parser/async-mapping.json`. Additional tests cover rejection, sticky failures, actual unresolved futures, checked callbacks, mutable registration, ownership cycles and event boundaries.

These are parser-scoped results, not proof of a complete terminal port. The real terminal, browser/WASM hosts, addons, platform runtime evidence and whole-workspace quality gates remain separate acceptance work. See `docs/escape-parser-acceptance.md` for the full scoped checklist and `docs/escape-upstream-tests.md` for synchronous migration details.
