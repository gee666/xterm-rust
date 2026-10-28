# APC escape-parser integration

`crates/xterm-core/tests/apc_escape.rs` exercises the actual Rust `EscapeSequenceParser`, including its APC subparser and asynchronous continuation state. It does not call the APC subparser in place of the outer parser. This is task 74, covering V6 in `docs/criteria/apc.md`. Native-terminal and browser API verification remain separate task 96 and V7.

## Oracle and scope

`scripts/apc-escape-oracle.mjs` loads unchanged upstream modules with the shared `escapeReference` loader. Original source files are read-only. The output includes each loaded source hash, corpus hashes, exact scenario counts and runtime version under `results/apc-escape/oracle.json`.

The regular corpus has 280 traces from 14 input scenarios, five chunk widths and four handler configurations. It contains 1,424 input chunks, 1,589 calls to the original parse method including resumes, and 165 asynchronous pauses. Whole input, one-codepoint chunks and widths 2, 3 and 7 all use the same input bytes/codepoints. The assertions compare the full ordered callback/fallback/print/execute logs plus current parser state, packed intermediates, Params/subparams and preceding join state after every operation. Each pending callback records parser state and the paused codepoint index before its future resolves.

The scenarios cover:

- ESC _ and C1 APC entry, direct G and digit identifiers, one and two intermediates.
- Matching registrations and unmatched identifiers using fallback.
- ESC backslash and C1 ST termination, including split terminators.
- CAN and SUB abort, interruption by a new CSI sequence, and text following termination.
- Empty payloads, ignored controls, TAB/LF/CR and other admitted controls, DEL and printed ASCII.
- Supplementary Unicode, lone surrogates, maximum Unicode and malformed u32 values, preserving JavaScript UTF-16 conversion.
- No handlers, one synchronous handler, mixed asynchronous handled/cleanup handlers and all-false bubbling.
- Reset of unfinished entry/payload and executor-driven asynchronous cleanup.

Three additional source-oracle scenarios preserve synchronous callback failure, rejected callback promise, and reset while end is paused. They compare exact partial callback logs and state. A rejected callback is not resumed as a false result. Reset while paused preserves ownership of the returned future, drains older cleanup callbacks, then resumes the same input chunk at the source's saved position. Error paths use `ParseError::Dispatch(DispatchError::CallbackFailed)` rather than panics or ignored errors.

The fixture files are generated data, intentionally containing long rows. Do not hand-edit expected reports. A Rust failure includes the scenario and handler configuration. The committed input operation trace identifies its exact chunks.

## Commands

```sh
node scripts/apc-escape-oracle.mjs
cargo test -p xterm-core --test apc_escape
cargo clippy -p xterm-core --test apc_escape -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/tests/apc_escape.rs crates/xterm-core/tests/parser_support/future.rs
```

Use `node scripts/apc-escape-oracle.mjs --generate` only for intentional reference changes. Verify fixture changes against the original source, not a preferred Rust output.

Observed on native Windows with Node v24.15.0 and Rust 1.85.1: both Rust test functions pass. They replay all 280 regular and three control/error scenarios. The latest shared-tree Clippy attempt is unsuccessful because other terminal/headless integration files do not compile, recorded in `results/apc/clippy.log`. Earlier passing subsystem checks do not override that result. No Linux/macOS or browser result is claimed.
