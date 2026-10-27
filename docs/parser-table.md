# VT500 transition table

`xterm_core::parser::table` ports the `TransitionTable` class and `VT500_TRANSITION_TABLE` definition in `source_repo/src/common/parser/EscapeSequenceParser.ts`, plus the state/action constants in `Constants.ts`. The source initializer comes from the xterm.js authors' VT500 parser adaptation. Its copyright and MIT notices remain in the generated data and Rust implementation.

`ParserState` and `ParserAction` are typed enums with the original numeric discriminants. `STATE_COUNT` replaces the source's `STATE_LENGTH` sentinel, which is not a state. The enums expose `ALL` and `from_code` for exhaustive iteration and checked decoding. `PAYLOAD_LIMIT` is the source's 10,000,000-unit protocol limit for OSC and DCS.

`Transition` holds `action` and `next`. `packed` encodes action in the high byte and next state in the low byte. `from_packed` rejects invalid action/state codes. `TransitionTable::new(length)` creates an all-zero custom table, just like a new Uint16Array. `vt500()` and `Default` construct the original 4257-entry table. The packed entries are readable but not independently mutable, so callers cannot insert invalid action or state values.

`set_default`, `add`, and `add_many` correspond to the source's `setDefault`, `add`, and `addMany`. Additions use `u8` code values and typed states/actions. Custom writes outside table capacity return `TableIndexError` instead of the JavaScript typed array's ignored writes. `add_many` validates the whole group before mutation, so an invalid group leaves the table unchanged. This is an explicit safety boundary mapping, not an assertion that arbitrary JavaScript custom-table writes behave identically.

`get(byte, state)` accesses a raw byte slot and returns `None` if the custom table does not contain that slot. `for_codepoint(codepoint, state)` first maps all values at or above U+00A0 to the source's `NON_ASCII_PRINTABLE` slot. This classification matches the escape parser's lookup. Invalid Unicode scalar values are still numerical parser input and use the same slot, rather than panicking. The default table's last state includes entries only through byte 0xA0, exactly like the source's compact allocation.

## Generated data and checks

The Rust engine reads native constant data in `parser/vt500_table.rs`. It does not run or delegate to a JavaScript parser. `scripts/vt500-oracle.mjs` extracts the complete original table initializer and its `TransitionTable` class before the `EscapeSequenceParser` class. Node's TypeScript transform handles enums; only type imports are removed. No transition rule is translated or rewritten by the generator.

The generator supplies a compact Rust array and a line-oriented test fixture. The duplication is intentional. Rust tests use a separately stored expected snapshot; the Node check compares both representations against a fresh execution of the upstream initializer. The 4257-line test fixture exceeds the normal source file limit because it is generated protocol data, not hand-written logic.

Run from the repository root with Node 24 or later:

```text
node scripts/vt500-oracle.mjs
cargo test -p xterm-core --test parser_table
cargo clippy -p xterm-core --test parser_table -- -D warnings
```

To regenerate after reviewing an upstream change, run `node scripts/vt500-oracle.mjs --generate`, then rustfmt the generated Rust file. The normal check ignores Rust formatting but checks every numeric table entry and every fixture line. `results/vt500/oracle.json` records the source and constants hashes, platform, tool version, state/action counts and compared-entry count.

`tests/parser_table.rs` compares all 4257 entries and all addressable byte/state slots, exercises every action/state packing combination, checks invalid packed values, confirms Unicode classification across every state, checks representative CSI/OSC/DCS/APC transitions, and tests custom-table mutation and invalid-write behavior. The generator independently checks source custom-table initialization, default replacement, single addition and multi-addition.

This verifies table data and operations only. Full escape-parser state transitions, handler registration, async continuation and terminal integration belong to task 26 and are not established by the table tests. Native Linux/macOS execution and browser integration remain separate verification requirements.
