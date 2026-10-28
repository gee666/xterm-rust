# HTML serialization

`crates/xterm-core/src/serialize/html.rs` ports `HTMLSerializeHandler` and the HTML range dispatcher from `source_repo/addons/addon-serialize/src/SerializeAddon.ts`. The public entry is `serialize::serialize_html`. It accepts the active `BufferAccess` view, dimensions, optional selection, host font/theme snapshot and `HtmlOptions`, returning `Result<Vec<u16>, SerializeError>`.

UTF-16 output preserves lone surrogates. Escaping deliberately follows the source's whole-cell switch: only cells exactly equal to `&` or `<` are escaped. Combined strings containing those units remain unchanged. This is source compatibility, not a promise that arbitrary terminal input produces sanitized HTML.

A selection is used only when requested and no explicit range exists. Range overrides selection and scrollback. Final selection columns are exclusive. Explicit HTML ranges extend through terminal columns. Missing lines still produce empty row wrappers; missing cells are skipped. Row wrappers do not reset the source's previous-cell attribute state, and width-zero placeholders still advance that state despite emitting nothing. Buffer borrow errors propagate instead of becoming successful empty fragments.

`HtmlConfig.font_family`, `foreground` and `background` retain UTF-16 vectors, including lone surrogates and embedded NUL. Global foreground/background distinguish absent values from present empty strings, matching the source's nullish defaults. The header appends these raw vectors without a lossy Rust string conversion. Parsed palette CSS remains valid Rust strings.

`HtmlConfig.ansi_colors` is `Option<[String; 256]>`. `None` selects the headless default ANSI palette; `Some` supplies a complete browser theme snapshot without invalid short-vector states. Palette generation moved unchanged from `xterm-web/src/theme/palette.rs` to `xterm-core/src/color/palette.rs`, with the browser module re-exporting it. The move avoids both duplicate generators and a core-to-browser dependency. Font sizes use the existing `ryu-js` dependency for JS number formatting, including exponent thresholds and negative zero.

## Assertion mapping

The six tests in `tests/serialize_html.rs` preserve the observable HTML assertions in upstream `SerializeAddon.test.ts:227-438`. Their fixtures represent the buffer after upstream write/selection operations, so they verify serialization, not the parser or live addon wiring.

| Rust test | Upstream HTML cases |
| --- | --- |
| `upstream_empty_selection_unsafe_chars_and_explicit_range` | Empty terminal with selection off, no selection, basic selection, unsafe characters, supplied range |
| `upstream_single_flag_style_assertions` | Bold, italic, inverse, invisible, dim, strikethrough |
| `upstream_underline_style_and_color_assertions` | Single, double, curly, dotted, dashed, palette color, RGB color |
| `upstream_combined_style_assertions` | Combined styling and complete span restyling |
| `upstream_foreground_background_theme_and_headless_assertions` | Foreground, background, custom palette, headless fallback |
| `upstream_global_default_custom_and_included_background_assertions` | Default global options, custom options, global background included |

Seven tests in `serialize_html_edges.rs` add exact wrappers, empty and reversed ranges, missing lines, scrollback clamps, multiline/exclusive selections, range precedence, placeholder and cross-row attribute state, whole-cell escaping, malformed UTF-16, decoration ordering, reserved underline styles, RGB padding, palette endpoints, theme overrides, JS font-number formatting and fallible buffer reads.

## Commands and evidence

```sh
cargo test -p xterm-core --test serialize_html --test serialize_html_edges
cargo clippy -p xterm-core --test serialize_html --test serialize_html_edges -- -D warnings
node scripts/serialize-oracle.mjs
cargo test -p xterm-core --test serialize_differential
```

Observed local debug results: 13 tests passed, zero failed or ignored, in `results/serialize/html-tests-pass.log`. Earlier failed test builds are retained as `html-tests.log` and `html-tests-final.log`; the latter caught a test-helper visibility error which was fixed. Concurrent engine/headless API changes blocked later Clippy attempts, retained in `html-clippy-pass.log` and `html-clippy-retry.log`. Those names do not imply success; the logs record failures.

Independent task 87 owns the original suite baseline and deterministic/seeded exact UTF-16 differential corpus in `scripts/serialize-oracle.mjs`, `tests/serialize_differential.rs`, fixture files and `results/serialize/oracle.json`. Its initial observed result was all 44 original cases passing, and 2092 differential cases passing, including 1050 HTML cases. Subsequent corpus expansions must be read from the current report rather than this historical count.

The lossless-option follow-up passed 14 direct HTML tests and both differential test groups in `results/serialize/utf16-tests-final.log`; the differential group covers 3334 source-generated cases. Focused Clippy with warnings denied passes in `utf16-clippy-clean.log`. Earlier failed attempts remain retained. The new raw-option regression checks malformed UTF-16 foreground/font values, present empty background and includeGlobalBackground exclusion.

Task 88 owns live native Rust terminal integration and terminal round trips. Task 131 owns the actual browser/WASM addon binding and three-browser evidence, including activation/proposed-API/lifecycle behavior. These detached-buffer tests do not establish that gate. Native Linux/macOS and actual browser results remain separate requirements. Full equivalence is NOT PROVEN.
