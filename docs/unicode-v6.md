# Unicode 6 provider

This is a partial xterm.js port. Terminal equivalence is not proven.

## Source mapping

| Upstream | Rust | Evidence |
| --- | --- | --- |
| `src/common/input/UnicodeV6.ts` | `crates/xterm-core/src/unicode_v6.rs` | `crates/xterm-core/tests/unicode_v6.rs` |
| Combining ranges in `UnicodeV6.ts` | `crates/xterm-core/src/unicode_v6_tables.rs` | Generated without changing source |
| Unicode types in `src/common/services/Services.ts` | `crates/xterm-core/src/unicode.rs` | Property packing tests |
| Static property helpers in `UnicodeService.ts` | `UnicodeCharProperties` methods | Reserved widths and signed extraction tests |
| `UnicodeV6.test.ts` | Full BMP comparison test | Original 65,536 assertions also executed in Node |

The implementation keeps the upstream BMP lookup table and supplementary interval search. `OnceLock` replaces JavaScript's module-global lazy initialization. Combining ranges override wide ranges, including U+302A through U+302F. Width methods accept `u32` instead of `char`, preserving lone surrogates and out-of-range numeric codepoints. Fractional or negative JavaScript numbers are not meaningful terminal codepoints and cannot be passed through this typed Rust contract.

`UnicodeCharWidth` has zero, narrow and wide variants. `UnicodeCharProperties` retains raw packed values, including reserved width 3. `new` masks state to 24 bits and width to two bits. `char_kind` performs JavaScript-compatible signed right shift on raw values. The provider uses the preceding packed width when joining zero-width characters, not just the codepoint's standalone width.

## Reproduction

Requires Node 24 for its built-in TypeScript stripping and Rust 1.85 or later. No npm dependencies are needed for this provider check.

```
node scripts/unicode-v6-oracle.mjs --generate-tables
cargo test -p xterm-core --test unicode_v6
cargo clippy -p xterm-core --test unicode_v6 -- -D warnings
```

The oracle evaluates the actual upstream TypeScript source after removing imports/exports and type syntax. It runs the original `UnicodeV6.test.ts` with assertion callbacks, then records widths across 1,114,112 codepoints and 110 property cases. Run-length encoding keeps the complete expected-width fixture small. Fixture generation executes JavaScript; the Rust test reads those fixtures and executes only Rust provider logic.

Raw source hashes, counts and runtime identity are in `results/unicode-v6/oracle.json`. Current local execution used Windows x64, Node v24.15.0 and rustc 1.85.1. All five Rust tests passed, with no ignored tests, and focused Clippy passed with warnings denied. These results do not claim native Linux/macOS execution, browser support, or complete UnicodeService/addon coverage.

The tables retain upstream copyright and MIT license attribution. Regeneration does not change `source_repo`.
