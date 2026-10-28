# Named control strings

`crates/xterm-core/src/data/escape_sequences.rs` ports all 67 declarations in `source_repo/src/common/data/EscapeSequences.ts`. The original xterm.js copyright and MIT notice remain. The file is generated protocol data, not a JavaScript execution dependency.

| Source namespace | Rust namespace | Declarations |
| --- | --- | --- |
| `C0` | `xterm_core::data::escape_sequences::c0` | 34 |
| `C1` | `xterm_core::data::escape_sequences::c1` | 32 |
| `C1ESCAPED` | `xterm_core::data::escape_sequences::c1_escaped` | 1 |

Every name stays uppercase and every value is a static Rust string with the same UTF-16 code units as the original JavaScript string. C0 includes Space and Delete in addition to U+0000 through U+001F. C1 includes U+0080 through U+009F. Escaped ST remains the two-character ESC/backslash string, distinct from the single-unit C1 ST.

Rust strings are UTF-8. C0 strings and escaped ST have the same byte values as their code units. Each C1 string has one UTF-16 unit but two UTF-8 bytes. For example `c1::CSI` contains U+009B and encodes as bytes C2 9B, not a raw single byte 9B. Use the string with the terminal string API or its encoded bytes with UTF-8 input. Do not treat the first UTF-8 byte as the control code. These values do not depend on platform line endings, locale or user configuration.

## Checks

`scripts/escape-sequences-oracle.mjs` inventories the complete supplied declaration groups and compares every Rust name/value in source order. It parses only the supported string-literal escapes through JSON decoding; it invokes no compiler, bundler or eval. An unfamiliar source inventory or literal fails verification instead of silently omitting it. `--generate` explicitly regenerates the native data; normal execution only checks it.

The verifier records the original source hash, group counts, all names and UTF-16 units, UTF-8 byte lengths, tool version and platform in `results/escape-sequences/source.json`. The result currently matches all 67 declarations. No dedicated original EscapeSequences test suite is claimed.

`tests/escape_sequence_constants.rs` names every declaration and checks all C0/C1 code units, group endpoints, Space/Delete, exact UTF-8 encoding and both ST representations. The three native test functions are separate from source-text verification.

From the repository root:

```text
node scripts/escape-sequences-oracle.mjs
cargo test -p xterm-core --test escape_sequence_constants
cargo clippy -p xterm-core --test escape_sequence_constants -- -D warnings
rustfmt --edition 2021 --check crates/xterm-core/src/data/escape_sequences.rs crates/xterm-core/tests/escape_sequence_constants.rs
```

Use the current controlled build-slot policy for Cargo checks. Source-only verification uses only standard Node APIs and writes less than 20MiB, so it requires no build lease. Current native execution remains unrun until the shared export and resource headroom allow it. A source-text match is not a native build/test pass or whole-terminal proof.
