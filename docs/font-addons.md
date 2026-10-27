# Font addon implementation and evidence

Overall font addon equivalence is NOT PROVEN. The CSS parser and fallback matcher have differential evidence. The web-font loader and relayout run as Rust/WASM against real FontFaceSet objects. GSUB shaping, local font access, ligature joiner lifecycle and complete Rust terminal renderer integration remain unfinished work under task 40.

## Implemented contracts

| Reference | Rust | Tests |
| --- | --- | --- |
| `addon-ligatures/src/parse.ts` | `crates/xterm-addons/src/ligatures/parse.rs` | `font_addons::complete_source_parser_assertions_and_seeded_utf16_traces` |
| `addon-ligatures/src/index.ts`, fallback function | `crates/xterm-addons/src/ligatures/mod.rs` | `font_addons::source_fallback_ranges_match_seeded_default_and_custom_patterns`, focused greedy/UTF16 tests |
| `addon-ligatures/src/LigaturesAddon.ts`, default fallback list only | `DEFAULT_FALLBACK_LIGATURES` | All 60 source patterns compared with default Rust matcher |
| `addon-web-fonts/src/WebFontsAddon.ts`, family split/quote/hash/partition | `crates/xterm-addons/src/web_fonts.rs` | Four native tests in `font_addons.rs` |
| Web-font ready/enumerate/select/add/load | `crates/xterm-web/src/font_set.rs` | `web_fonts_browser.rs` and `web_fonts_async_browser.rs` |
| Web-font activation/loadFonts/relayout/dispose | `crates/xterm-web/src/font_addons.rs` | `web_fonts_browser.rs` and `web_fonts_async_browser.rs` |

All reference paths are under `source_repo/addons/`. The five source web-font Playwright cases are not yet migrated unchanged against a complete Rust terminal. The WASM tests use a recording options host to verify property writes, plus real browser font objects. They do not prove measured terminal-renderer relayout or complete terminal integration.

## Language and safety boundaries

Parser inputs, outputs and fallback ranges use raw UTF-16, preserving lone surrogates. `parse_families` returns `Result<Vec<Vec<u16>>, ParseError>`. A non-string input is unrepresentable in this typed Rust API. No browser parser binding is currently exported; any future dynamic binding must reject non-string input. Out-of-range escapes preserve the source error message, and source quirks such as dropping a final quoted component without a separator remain unchanged.

`FallbackLigatures::new` accepts an empty list, which disables fallback. It rejects an empty pattern with `EmptyLigature`. The source matcher loops forever on an empty pattern because its index never advances. The Rust API does not reproduce this hang. This explicit safety deviation is not a claim of exact behavior for invalid zero-length patterns. It remains listed in the full-addon equivalence gaps.

WASM exports are `WebFontsAddon`, `loadFonts`, mutable `initialRelayout`, `activate`, `dispose`, `loadFonts` and `relayout`. Browser loading errors reject promises, not successful empty results. Invalid request values outside the declared string-or-FontFace API reject with a TypeError. The host must expose a live `options.fontFamily` property. This supports a Rust-backed terminal without executing the upstream JavaScript engine.

## Reproduction

Run from repository root with Node 24.15.0, Rust/Cargo 1.85.1, wasm32-unknown-unknown, wasm-bindgen 0.2.100 and the source Playwright installation plus browser binaries. Use the shared Cargo target directory and existing debug/incremental configuration.

```sh
node scripts/font-oracle.mjs
cargo test -p xterm-addons --test font_addons
cargo clippy -p xterm-addons --lib --test font_addons -- -D warnings
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test web_fonts_browser -- -D warnings
WASM_TEST=web_fonts_browser node scripts/browser-host-check.mjs
WASM_TEST=web_fonts_async_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

The oracle defaults to byte-for-byte verification of tracked fixtures. `--generate` is an explicit regeneration operation, not a normal check. On drift, it retains the actual output in `tmp/font-oracle/*.actual` and exits unsuccessfully. It copies source code/tests to `tmp/font-oracle`, changes imports only, runs the 16 original test bodies and assertions, and records reference hashes. Source files and expected assertions are not edited. Node's strict assertion functions replace Chai and report actual assertion calls independently of test-function counts.

The seed is `0xF017CA5E`. The parser corpus contains 8,224 rows, comprising 16 original calls, 16 named extra boundary cases and 8,192 randomized cases. The fallback corpus contains 4,156 rows, comprising all 60 default patterns and 4,096 randomized default/custom pattern cases. Random cases include isolated surrogate units, malformed escapes, JS whitespace, punctuation and empty text/lists. Zero-length patterns are excluded because the source would not terminate.

## Observed results

Windows native `cargo test -p xterm-addons --test font_addons` passes seven Rust test functions. Two functions execute every corpus row above. This is not 12,380 independently named Rust tests. Source parser discovery/execution is 16 test functions and 16 assertions, all passed, none skipped or unrun. These exact counts and hashes are in `results/font-addons/parser-oracle.json`.

The first browser target has eight Rust/WASM tests. All eight passed in each observed browser: Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Exact build commands, browser versions and test exports are in `results/browser-host/web_fonts_browser-results.json`, with timestamped attempts and logs beside it. Failures during shared-tree signature migrations remain retained rather than erased. This is Windows browser-runtime evidence, not native macOS/Linux evidence.

Native and WASM targeted Clippy passed with `-D warnings`. Their logs and native test output are in `results/font-addons/`. Workspace formatting is checked separately and may expose another owner's live edits; retain its output rather than reformatting others' work.

The asynchronous browser target adds disposal-during-load, reactivation-during-load, duplicate request/load order, source/display-excluded descriptor hashing and case-sensitive/unquoted family lookup. Consult its current result artifact rather than assuming it passed from test existence.

## Open work

- Complete `font.ts` legacy/modern local font access, permissions, blob loading and generic-family stop behavior.
- Port all `fontLigatures/` GSUB6/8 processors, contextual rules, tree merge/flatten/walk, glyph mapping, result ownership and cache eviction.
- Wire the `LigaturesAddon` joiner, style, stale family load completion, refresh and disposal to the live Rust renderer.
- Run all four supplied shaping font fixtures and original shaping suites, retain randomized differential counterexamples.
- Reuse original web-font browser assertions against the complete Rust terminal, including CSS font loading and measured relayout.
- Finish default activation and all ready/load/dispose/error race cases with controlled promise tests, not sleeps.
- Obtain independent review and complete whole-workspace/native-platform evidence.
