# Font metadata constructor validation

Overall font equivalence is NOT PROVEN. Task339 adds fixed metadata validation to the existing Rust ligature font reader. Task342 retains the complex metadata and exact JavaScript error-ordering work.

## Implemented behavior

`crates/xterm-addons/src/ligatures/font_metadata.rs` validates the source constructor's `head`, `hhea`, `maxp`, `OS/2`, `post`, `ltag` and deferred `name` reads. It checks the head magic number, version-dependent field lengths, language-tag/string bounds, recognized name-language and encoding paths, and post custom-name strings. Unsupported name encodings skip string reads, as upstream does. UTF-16 name strings with an odd byte length still read their final two-byte unit.

Uncompressed metadata retains access to the original font's remaining bytes rather than enforcing the directory's declared table length. Signed post string lengths can move the cursor backwards, including into preceding tables. Compressed WOFF metadata uses its decompressed table buffer. This preserves the source's boundary behavior instead of adding stricter OpenType-spec validation.

`font_bytes.rs` selects the last name directory descriptor without reading or decompressing earlier entries, then requires the deferred name table, an outline-table combination and hmtx presence. For TrueType fonts, `font_cmap.rs` checks referenced glyph indices against the parsed maxp glyph count. TrueType hmtx validation now reproduces the source's metric byte reads, zero-read offsets and WOFF preparation timing. See `docs/font-hmtx-validation.md`. Constructor errors now include a typed `FontError::Type` for two source failures. The browser font loader maps this variant to a JavaScript TypeError. In particular, the source's nonempty post2.5 constructor fails because it reads the nonexistent glyphNameIndex array. Rust preserves that failure rather than repairing the upstream parser.

`font_name_languages.rs` contains source-derived language membership data. `scripts/generate-font-mac-tables.mjs` generates and verifies it alongside the existing Mac decoding tables. Attribution remains in `licenses/opentype-js-MIT.txt`.

## Evidence

All commands ran on Windows with Node24.15.0 and Cargo jobs limited to2. New build and scratch outputs stayed in the owned O: worktree. `results/font-metadata/` contains command logs and the exact browser runner report. The latter retains execution-time paths and artifact hashes; temporary binaries are disposable, not committed.

```sh
node scripts/font-metadata-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --metadata
OPENTYPE_SOURCE=PATH_TO_REFERENCE/node_modules/opentype.js/dist/opentype.mjs node scripts/generate-font-mac-tables.mjs
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_metadata_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

Use the existing documented wasm-bindgen0.2.100 and Playwright configuration for the browser runner. Shell environment assignments above use POSIX syntax. PowerShell users set the corresponding `$env:` variables before invoking Node.

The source oracle executes the unchanged installed opentype.js2 constructor on reconstructed fonts with specific table mutations. The fixture records source/font hashes and each mutation's accepted result or exact source error. The current corpus has950 cases,337 accepted and613 rejected, including47 WOFF cases. Four cases cover duplicate name descriptors with invalid early or final entries. Seven later GSUB cases include ignored invalid offsets and compressed data;65 additional hmtx cases cover TrueType metrics and deferred descriptors. Both Rust native tests pass, including all950 outcome comparisons and explicit typed-error assertions. Seven existing ligature tests pass, preserving the original/seeded font outputs, cache/graph behavior and positive WOFF regressions. These are test-function counts, not assertion counts.

The shared addon loading scenario compares five recorded fields against unchanged original addon execution. The later task381 run passes both the complete950-case Rust constructor corpus and the async addon-loading test in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. `results/font-hmtx/` retains the latest browser report and metric-read failures. `results/font-gsub-directory/` retains the earlier885-case run and before/after GSUB failure receipts. Bad head metadata keeps fallback ranges, does not refresh, and caches the failed load. Changing to a valid font family loads successfully and refreshes once. This is an addon boundary-host test, not a real Terminal renderer test.

The full `cargo fmt --all -- --check` command passes after safely moving the owned worktree from `O:/xterm-rust-swarm-16/tmp/worktrees/Liam16` to the shorter `O:/xterm-rust-swarm-16/tmp/L`. No junction or drive mapping was used. Native addon all-target/all-feature Clippy and the focused WASM test Clippy pass with warnings denied.

## Remaining contracts

The first independent review found an acceptance mismatch for duplicate name records. Rust read an invalid early descriptor that the source ignored. Data access and decompression now happen only after selecting the final name descriptor. `native-review-fix.log` records the corrected878-case run. Earlier874-case and three-browser receipts predate this pure Rust selection fix. Task374 subsequently reran all885 then-current constructor cases and the addon-loading test in all three browsers, including the duplicate-name cases. Task381 reran the expanded950-case corpus in all three engines. See `docs/font-gsub-directory.md` and `docs/font-hmtx-validation.md`.

Task342 retains eager CFF/CFF2, GDEF/GPOS, kern, color/SVG/meta and variation parsing; full glyf/loca/hmtx validation; missing-head/post downstream behavior; malformed cmap/GSUB versus metadata error precedence; exact compression failure behavior; and exact JavaScript errors beyond the two new TypeError cases. Native typed bounds/missing-table errors are not identical to every upstream JavaScript message. Task335 retains raw character-joiner semantics. Full original suites, real terminal font/addon interactions and native Linux/macOS execution remain separate requirements. These scoped passes do not establish full port equivalence.
