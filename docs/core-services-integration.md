# Core and log engine integration

Task 68 owns `crates/xterm-core/tests/core_services_integration.rs`. The production terminal engine belongs to task 28; these checks call that engine rather than constructing substitute buffers or a second parser. The six test cases close the native integration portion of criteria 22 and 23. Browser/WASM bindings and the full terminal differential suite remain separate work.

## Integration cases

1. Write enough actual text and line feeds to create scrollback, scroll to the top, and send malformed UTF-16 user input through `CoreTerminal::input`. Assert scroll-to-bottom happens before the user-input observer and before the forwarded Data event. Verify the active buffer viewport, Debug/Trace logs and exact units. Change scrollOnUserInput live, repeat, and assert the viewport stays put.
2. Through the same real terminal, non-user data and binary events preserve lone surrogates and do not scroll or produce user-input notifications. Live disableStdin suppresses both kinds, scroll requests, user-input notifications and logs. Re-enabling stdin admits an empty data event.
3. Send actual parser escape sequences for insert mode, application cursor keys, bracketed paste and hidden cursor. Read the shared CoreService state, then reset the terminal. Modes and Kitty flags/stacks reset, cursor initialization and hidden state survive, no redundant parser RequestReset notification is emitted, and the original Data forwarding subscription still works.
4. Parse text plus CSI 6 n. The cursor-position reply must be `ESC [ 1 ; 3 R` after two printed cells. Data observers and a custom logger read the real terminal state during callbacks without a mutable state borrow crossing either callback. Updating logLevel to off suppresses later logs. The custom logger receives records instead of the engine's default log collection.
5. A custom logger returns an opaque native error. `CoreTerminal::input` returns the same error through TerminalError, CoreError and LogError. Both `Error::source` traversal and Rc identity remain intact. The failed send does not forward Data.
6. Keep CoreService and LogService handles after terminal disposal. Repeated dispose succeeds, terminal input rejects as disposed, retained core handles cannot notify removed listeners or the terminal event queue, and log threshold updates are disconnected. Dropping the terminal releases its state even while core/logger handles remain alive.

The tests prompted two engine fixes, both owned by Coleman: explicit disposal of subscription handles and core/logger services; and deferred built-in replies/logging outside the TerminalState borrow. The latter avoids a RefCell panic when an observer reads the state. Moving internal work must not be taken as proof of full upstream event-time equivalence. Public/headless event-time assertions and original snapshot differential work have separate criteria.

## Commands and evidence

```sh
cargo test -p xterm-core --test core_services_integration
cargo clippy -p xterm-core --lib --test core_services_integration -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/tests/core_services_integration.rs
```

Initial execution is still in progress. `results/core-services/engine-tests1.log` retains a compilation failure during the Buffer shared-cleanup API migration. `engine-tests2.log` then passed five integration cases and exposed an incorrect expectation in the new public-reset test. Source CoreTerminal.reset does the reset directly and does not emit the parser's RequestReset event. The test now asserts that no redundant request occurs. No upstream assertion was changed. The corrected final run passes all six integration tests and twenty service tests in `engine-tests3.log`. `engine-clippy1.log` records a successful warning-denied Clippy run for the library, both unit targets, integration target and oracle example. `engine-oracle1.log` records 20,616 successful service differential operations. Independent integration review is still required.

The service-level original-source oracle remains `node scripts/check-core-services.mjs`, with 3 original cases, 4 original assertions and 20,616 operations at seed 0x434f5245. Those are counted separately from these six new Rust integration tests. Source behavior references are `source_repo/src/common/CoreTerminal.ts`, `source_repo/src/common/services/CoreService.ts`, `source_repo/src/common/services/LogService.ts`, and the CSI/DEC handlers in `source_repo/src/common/InputHandler.ts`.

Default native engine logs are retrieved with `drain_logs`; a host console adapter chooses where to display the provided route and UTF-16 message. These tests prove default record routing and custom native callbacks, not browser console methods, JS exception binding identity, native Linux/macOS execution, the complete public callback API, or full xterm equivalence.
