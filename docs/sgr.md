# SGR attribute processing

`terminal/sgr.rs` ports `InputHandler.charAttributes`, `_updateAttrColor`, `_extractColor`, `_processUnderline` and `_processSGR0` from the supplied source. It uses the existing AttributeData and Params implementations, with no additional production dependency.

`apply(&mut AttributeData, &Params)` uses the source default of enabled kitty bold/faint controls. `apply_with_options` accepts the live boolean for kittySgrBoldFaintControl. Both return unknown parameters in source order. Engine task #28 routes these through its logger as `Unknown SGR attribute: {code}.`, rather than losing diagnostics in a successful no-op. The computation has no terminal, browser or operating-system dependency.

Color extraction retains the source accumulator and parameter-advance rules. It handles colon, semicolon, mixed separators, missing fields and malformed unsupported modes, including the rare extra accumulator slot. This is intentionally not replaced with a more permissive ANSI color parser. Color bytes use source bit masking, not clamping. Extended attributes clone only on the source paths, preserving URL id and dropping payload through the source clone contract. Existing cell references keep their old object.

## Reproduction

```text
node scripts/check-sgr.mjs
cargo test -p xterm-core --test sgr
cargo clippy -p xterm-core --lib --test sgr -- -D warnings
rustfmt --check crates/xterm-core/src/terminal/sgr.rs crates/xterm-core/tests/sgr.rs
```

The Node checker uses the reference's installed esbuild and TypeScript packages. It bundles the unchanged InputHandler class and actual Params/AttributeData dependencies into `tmp/sgr/original.mjs`. A constructor-free test object supplies only the attribute, option and diagnostic fields used by the unchanged SGR methods. The adapter recognizes SGR-only strings for those methods. It does not pretend to execute the complete parser or terminal constructor.

The checker executes the original assertion bodies in the exact contiguous `text attributes` and `colon notation` sections of InputHandler.test.ts. It preserves all 35 cases and 1,162 runtime assertions, using strict Node equality on the same primitive/array values. It captures 332 source-validated attribute transitions for Rust replay. Exact case names and step counts are retained in `results/sgr/oracle.json`; unexpected discovery counts fail rather than silently shrinking coverage.

An additional 1,046 directed transitions and 12,000 seeded transitions use seed 1397183025. They cover raw packed flags, existing URL/payload state, every byte-valued SGR parameter, color targets, truncated/extra/mixed subparameters, both kitty option values and unknown diagnostics. Rust replays all 13,378 rows from `tests/fixtures/sgr-oracle.tsv`, comparing full fg/bg/raw extended bits, URL id, payload presence, clone/share identity and unknown-parameter order. This generated oracle data exceeds 350 lines by design. `--write-fixtures` regenerates it; normal checks only verify the committed fixture.

The Rust suite also checks source-style flag set/reset assertions, all 256 palette values, RGB/basic/bright/default transitions, protected-bit preservation, kitty enable/disable, SGR0 URL/variant/payload clone behavior and underline-color packed-bit quirks. Replay of full-state records is reported separately from the 1,162 original JavaScript assertions, not miscounted as 35 Rust test functions.

The original SGR0, extended underline and underline-color persistence describe blocks are not executed by this scoped adapter. Their full parser/buffer assertions remain engine integration work under task #28 and criterion #253. The direct Rust boundary tests and differential rows cover their helper semantics but do not replace their actual terminal/cell assertions. Browser rendering and native Linux/macOS runtime checks also remain separate. Full terminal equivalence is NOT PROVEN.

Detailed branch acceptance is in `docs/acceptance/sgr.md`, criteria 251 through 253. Raw commands, source/fixture hashes, counts and any integration failures are under `results/sgr/`.
