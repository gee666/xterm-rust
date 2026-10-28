# ANSI serialization implementation

The acceptance checklist is `docs/serialize-acceptance.md`. Full addon equivalence is NOT PROVEN. Task 37 implements the ANSI serializer and terminal-state orchestration; task 103 implements HTML, task 87 owns the unchanged-source oracle, and task 88 owns live native/browser addon integration and round-trip checks.

## Source mapping

| Source | Rust |
| --- | --- |
| SerializeAddon.ts BaseSerializeHandler traversal | serialize/ansi.rs serialize |
| equalFg, equalBg, equalUnderline, equalFlags | serialize/style.rs |
| StringSerializeHandler _diffStyle | serialize/style.rs diff |
| _nextCell | serialize/ansi.rs Handler::next |
| _rowEnd, forced-wrap repair | serialize/ansi.rs row_end and wrap_separator |
| _serializeString cursor and attribute restoration | serialize/ansi.rs finish |
| _serializeBufferByScrollback, _serializeBufferByRange | serialize/terminal.rs and serialize/mod.rs options |
| normal/alternate dispatch, _serializeModes, _serializeScrollRegion | serialize/terminal.rs |
| IBuffer API read operations | serialize/buffer.rs BufferAccess for native Buffer |

All Rust paths above are under `crates/xterm-core/src/`. The source is `source_repo/addons/addon-serialize/src/SerializeAddon.ts`. Upstream's MIT notice remains in source headers and `licenses/xterm-js-MIT.txt`.

## Rust representation

`serialize_buffer` and `serialize_terminal` return `Result<Vec<u16>, SerializeError>`. UTF-16 output retains lone surrogates and combined cell strings. Converting to `String::from_utf16_lossy` is an application choice, not part of the serialization comparison. A `SerializeState` borrows normal and alternate buffer views, current attributes, dimensions and modes for one synchronous call. It does not retain detached terminal state or callbacks.

`BufferAccess` reads native `Buffer` cells directly. Cell, line-length and wrap reads use `try_borrow`; a caller holding a mutable line borrow receives `BorrowedLine`, not a panic or omitted output. Missing lines return `None` separately. The serializer does not call host callbacks while traversing cells.

A reversed ANSI range and a scrollback count selecting zero rows return `InvalidRange`. Upstream's handler allocates an array using end minus start, which throws `Invalid array length` in those cases. Native row indices and counts use integer Rust types. JavaScript fractional/NaN/infinite option validation and marker conversion still require the browser adapter in task 88; these types alone do not prove those boundaries.

## Focused tests

`cargo test -p xterm-core --test serialize_ansi --test serialize_native` currently defines 18 tests. The 14 ANSI tests cover restored cursor style, the three upstream explicit-range expectations, all five underline styles, palette/RGB underline colors, empty/internal/colored null cells, natural and forced wraps, wide placeholders and lone surrogates, ANSI/bright/palette/RGB colors, flag order, cursor movement, alternate/mode/margin order, and range errors. Four native tests cover live cell updates, all three fallible read operations under mutable borrow, recovery after releasing the borrow, missing-line behavior, and the source error for an explicit range extending past the buffer.

These constructed buffers isolate serialization from parsing. They are not a claim that all original test bodies have run against a Rust terminal. Task 87's independent fixture comparison supplies source-output evidence and task 88 must supply actual same-size Rust round-trips and addon lifecycle tests. The unchanged upstream baseline has 44 source tests, with exact outcomes recorded by task 87 under `results/serialize/oracle.json`.

The expanded independent oracle exposed directed fixture 1254, where an ANSI range extended past the last buffer row. Rust initially returned partial success while the source threw a TypeError reading the absent next row's `isWrapped` property. The serializer now propagates `MissingWrappedLine` with the source message. The failing input and original incorrect UTF-16 output are retained in `results/serialize/counterexamples/range-past-buffer.json`. No expected fixture was weakened. All 3334 source-output comparisons, including 1663 ANSI and 1671 HTML cases, passed debug and release after the fix. Live WASM error-class mapping remains part of task 88.

The local environment is native Windows with rustc 1.85.1, Cargo 1.85.1 and Node v24.15.0. Rustfmt and focused test/Clippy logs are retained under `results/serialize/`. Some intermediate runs are compiler failures in other shared modules during active API migrations; they are not reported as passing gates. Linux/macOS, full workspace gates and actual browser/WASM addon execution remain separate unproven checks.
