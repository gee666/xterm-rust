---
name: escape-sequences-check
description: Use when verifying or regenerating the complete C0, C1 and escaped control-string declarations against EscapeSequences.ts.
---

Run `node scripts/escape-sequences-oracle.mjs` from repository root. This is a tiny standard-library-only source check, not a bundler or compiler. It compares all67declared names and exact UTF16values, records source/Rust hashes, and writes results/escape-sequences/source.json. Use --generate only after reviewing an intended upstream change.

Run `cargo test -p xterm-core --test escape_sequence_constants` and targeted Clippy -Dwarnings through the current build-slot policy. Focused rustfmt covers src/data/escape_sequences.rs and tests/escape_sequence_constants.rs. The67constants are generated protocol data, not terminal logic or a JS engine delegate.

Read docs/escape-sequences.md for c0/c1/c1_escaped namespace mapping and the important C1 UTF8byte distinction. A source-text match alone is not a native test or whole-port pass. Preserve original MIT attribution and keep platform/runtime claims scoped.
