# Independent parser review by Viesturs

## DCS task 18

Compared `parser/dcs.rs` and `dcs_handler.rs` with the complete upstream `DcsParser.ts` and its test assertions. Inspected registration identity, reverse mutable-list traversal, idle/fallback identifiers, active-list lifetime after clear, boolean and asynchronous bubbling/cleanup, reset positions, UTF-16 limits, Params snapshots/canonicalization and failure retention.

Two findings were corrected before acceptance:

- Callback-driven removal of required active indices originally succeeded silently. The corrected implementation returns `DispatchError::ActiveHandlerRemoved` and preserves partial state where upstream throws TypeError. Source-confirmed regressions cover each callback loop.
- The mutation test originally held a callback/token reference cycle. It now uses weak token-store ownership. Production registry/registrar/token ownership was also inspected for cycles and borrows across callbacks.

Independently executed `node scripts/verify-dcs.mjs` on native Windows with the repository Cargo profile. `results/dcs/run-0004/verification.json` is complete, passed, and has no changed inputs. All source oracles, formatting, Rust tests and warning-denied Clippy passed. The Rust receipt has 35 DCS tests, five actual escape-parser integration tests and four shared registry tests, no failures or ignored tests. The unchanged original DCS baseline passed all 20 cases; the seeded corpus has 512 traces.

This accepts the DCS implementation scope. It is not proof of live terminal/browser image behavior or Linux/macOS runtime verification.

## Escape parser task 26

Inspected `escape.rs`, `escape_parse.rs`, `escape_resume.rs`, `escape_handlers.rs`, the shared registry and the original source. Reviewed the original-case mapping and native synchronous/asynchronous assertions. The native parser owns actual OSC/DCS/APC dispatch rather than calling JavaScript. All production parser code is safe Rust.

Review covered identifier validation and exact error messages, source fast-path exceptions for custom tables, chunked payload boundaries, Params/collect/join-state changes, cancellation/termination, raw executable arguments, reverse registration, async false exhaustion without fallback, sticky improper continuation, reset while paused, typed callback failures and retained source error causes. The explicit lazy-future executor contract is documented. Invalid arbitrary numeric state/custom table behavior that Rust types cannot represent is not claimed as equivalent.

The first independent run, `results/escape-parser/viesturs-review-1`, found owned async test formatting failures. Max corrected those files. The second independent run, `results/escape-parser/viesturs-review-2`, is source-stable and passed the original 161 tests, fixture/source mappings, full-state 3,334-trace oracle with 92,668 parse calls, scoped formatting and 61 Rust tests. Its Clippy command failed only on concurrent headless queue and move-to-cell warnings outside the parser implementation. Those failures remain in the receipt and were sent to their owners. Max's earlier source-stable attempt 3 includes a clean focused Clippy result. Later source changes add error-cause coverage and do not justify presenting that old receipt as current whole-workspace success.

Separately authored actual APC integration tests pass 280 source-generated traces, including 165 asynchronous pauses, and three exact callback-error/reset controls. They compare callbacks, parser state, collect, Params and preceding join state. See `docs/apc-escape.md`. These tests corroborate the outer parser path but are not an independent review of my own APC task.

The implementation review has no remaining parser-owned finding. Global warning-free Clippy and workspace verification remain required under the final quality tasks and criteria. No platform/browser/full-terminal equivalence follows from accepting this scoped implementation.
