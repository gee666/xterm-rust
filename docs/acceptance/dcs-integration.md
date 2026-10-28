# DCS through the real escape parser

This extends the isolated DCS checklist in `docs/acceptance/dcs.md`. The test entry is `EscapeSequenceParser::parse`, not a direct DcsParser call. Ownership is `tests/escape_dcs.rs` and DCS-specific reference traces. Max owns the outer parser and common async-family tests.

Required checks:

- Introduce DCS with both ESC P and C1 DCS. Terminate with both ESC backslash and C1 ST. Parse prefix/intermediates and semicolon/colon Params exactly, including zero defaults and malformed headers that must not invoke a registered handler.
- Split every deterministic stream at every UTF-32 codepoint boundary, plus byte-sized decoded chunks. Include splits between ESC and P, header digits, subparameter separators, intermediate bytes, ESC and backslash, and supplementary/lone-surrogate payload values.
- Deliver every payload chunk to the actual DCS handler exactly once. Preserve C0/DEL treatment from the reference's read-ahead rules. Record PRINT/EXECUTE/ESC/CSI callbacks after termination so the test proves the outer parser resumed.
- Distinguish ordinary ST completion from CAN/SUB abortion, including following printable text. BEL is not an OSC-style DCS terminator. Reset during DCS aborts the active handler and leaves a usable parser. Dispose does not invent end callbacks.
- Run newest-first registered handlers, false bubbling and true cleanup through real parser input. Retain pending unhook continuation at the actual terminating codepoint; resume the same chunk without repeating prefix/payload or dropping suffix characters. Reset while paused must clear the old continuation and transfer reset cleanup exactly once.
- Compare complete ordered callbacks and final outer parser state, collect, Params and preceding-join state with a fresh bundle of read-only reference TypeScript. Preserve source hashes, deterministic corpus counts and generated fixture hash. Do not use a stale reference build or call JavaScript from the Rust implementation.
- Propagate a callback-removal DispatchError through ParseError instead of swallowing it. Verify partial-state recovery by reset and a subsequent valid stream.
- Run focused Rust integration tests and warning-free linting under the existing shared workspace profiles. Preserve command failures, not only later success.

Not claimed by these checks: a complete terminal, native UI, browser/WASM integration, addon behavior or other-platform runtime evidence. Those remain separate work and cannot be inferred from parser-only results.
