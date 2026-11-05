# Eager SVG font constructor validation

Task408 repairs SVG-table constructor acceptance in the Rust ligature font reader.
It does not implement SVG glyph rendering or complete font metadata task342.
Overall equivalence remains NOT PROVEN.

## Behavior

`crates/xterm-addons/src/ligatures/font_svg.rs` checks the bytes read by the
supplied opentype.js `parseSvgTable`. `FontTables::parse` invokes it for each SVG
directory entry during the eager metadata phase, before deferred name, outlines,
hmtx, kern and GSUB processing. An invalid earlier duplicate is not ignored.
Uncompressed reads use the remaining original font buffer, not the directory's
nominal table length. Compressed WOFF reads use the decompressed buffer.

Only the version word is read for nonzero versions. Version zero reads the list
offset, entry count, glyph bounds, document offsets and lengths in source order.
The source constructs a document view even for reversed glyph ranges. A
zero-length document still requires an in-bounds offset. Repeated document
offsets reuse the first view, so later lengths are read but not range-validated.
The implementation retains offsets for this check without allocating unused SVG
document/glyph maps. Checked arithmetic propagates `FontError::Bounds` instead
of panicking on terminal-provided font data. No dependency, unsafe code or
JavaScript font-parser delegation was added.

Native errors deliberately use the existing typed bounds/missing-table errors.
This is not a claim that their displayed messages equal every engine's JavaScript
RangeError or TypeError text.

## Evidence

`results/font-svg/` retains scoped Windows evidence. The source oracle records
183 constructor cases, 68 accepted and 115 rejected. It executes the unchanged
installed opentype.js constructor, SHA-256
`737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`.
The separate SVG fixture leaves the existing1471 metadata cases and assertions
unchanged. Independent JavaScript and shared native/WASM builders reconstruct
the same descriptor/payload inputs. All776 reference-manifest files were
verified unchanged.

`baseline.log` records113 actual pre-fix mismatches, including109 incorrectly
accepted fonts and four wrong earlier-error results. `native.log` records one
passing Rust test function comparing all183 cases and two retained kern controls.
`metadata.log` records four passing existing metadata/warning functions, including
all1471 original constructor outcomes. `ligatures.log` records seven passing
existing ligature functions. These are case and test-function counts, not a count
of distinct upstream assertions.

Native addon all-target/all-feature Clippy and scoped font-addon WASM test Clippy
pass with warnings denied. The exact `cargo fmt --all -- --check` command fails
with Windows OS206, including after safely shortening the owned physical path to
`O:/xterm-rust-swarm-16/tmp/C`. The same command also fails under installed
Rust1.97.1. Both workspace package partitions pass formatting checks, covering all
five members. These passes do not erase the exact-command failure.

`browser.json` records two passing Rust/WASM test functions in each of
Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine compares all183
constructor cases and two kern controls, then runs a shared source/Rust async
addon-loading scenario. There are zero failed, ignored or filtered browser tests.
The scenario replaces an ignored DSIG descriptor with a truncated SVG descriptor.
It keeps fallback ranges, avoids refresh, caches the rejected family, and loads a
different valid family successfully. `addon-oracle.json` retains the unchanged
source addon hashes and its independently executed expected trace. This is an
actual Rust addon with a terminal boundary host, not a full interactive Terminal
renderer test. Former absolute build paths and artifact hashes in the browser
report are provenance; generated binaries are disposable after review.

## Reproduction

Use an owned short O: worktree, Cargo jobs2, the existing reference Node
dependencies and wasm-bindgen CLI0.2.100. Set TEMP first, then TMP and TMPDIR to
that owned worktree's tmp directory. Set SOURCE_PACKAGE_JSON and PLAYWRIGHT_MODULE
to existing reference dependencies and WASM_BINDGEN to the matching CLI. Observe
the team-wide compiler/browser limits.

```sh
node scripts/font-svg-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --svg
cargo test --locked -j2 -p xterm-addons --test font_svg_constructor
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_svg_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_svg_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
cargo fmt -p xterm-core -p xterm-addons -- --check
cargo fmt -p xterm-web -p xterm-font-addons -p xterm-wasm-support -- --check
```

The shell environment examples use POSIX assignment syntax. PowerShell users set
corresponding `$env:` values. Toolchain observed here is Rust/Cargo1.85.1,
Node24.15.0 on native Windows. The final workspace/platform matrix, unchanged
complete original suites, live Terminal rendering, and the remaining CFF,
variation, GDEF/GPOS/color and outline constructors are not established by these
scoped checks. Temporary build/browser artifacts must be removed after review
and preservation of source, fixtures and concise receipts.
