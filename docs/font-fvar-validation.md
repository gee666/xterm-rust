# Deferred fvar constructor validation

Overall font equivalence is NOT PROVEN. Task419 closes the missing fvar constructor reads in the Rust ligature font loader. Task342 retains the other complex metadata and variation behavior.

## Contract

`font_bytes.rs` selects the last fvar descriptor without reading or decompressing earlier entries. `font_fvar.rs` validates the unchanged opentype.js constructor's version, header, axis and instance reads. Uncompressed data retains the rest of the original font DataView rather than treating the directory length as a boundary. Compressed WOFF data uses its inflated table buffer.

The source skips countSizePairs and flag fields. It does not enforce OpenType axis or instance sizes. Strides of zero, overlapping records, unreferenced offsets and omitted skipped bytes can therefore be valid. With no axes, an instance whose declared size is four reads only its initial two-byte name ID. Other instance sizes trigger the optional PostScript name ID read, even when the declared size is smaller than that read. The Rust validator preserves these decisions instead of adding spec validation.

GSUB constructor parsing precedes fvar validation. Ligature tree construction follows it. A kern warning can mutate the original ArrayBuffer before fvar parsing. The loader uses the refreshed payload with its original descriptor, preserving task406's copied-directory semantics. Header/name/cmap failures still precede fvar errors.

The new module validates observable constructor reads. It does not expose variable-font axis selection or implement shaping with variation coordinates. Those features and the other variation tables remain part of342/325. Native `FontError::Bounds` represents source DataView/typed-array boundary errors. Its Rust display text is not a claim of identical JavaScript RangeError messages. `FontError::Invalid` preserves the source's unsupported-fvar-version message.

## Tests and source provenance

`scripts/font-fvar-oracle.mjs` executes the unchanged installed opentype.js constructor. Its279 directed recipes reconstruct the original Ubuntu Mono font and retain55 accepted and224 rejected outcomes. They cover truncated headers, invalid versions, overlapping and zero strides, optional instance fields, ignored offsets, WOFF compression, duplicate descriptors and earlier missing-table/cmap/GSUB errors. The fixture contains the dependency and original-font SHA256 values. The source dependency is the supplied installed opentype.js, not an invented upstream revision.

The native/WASM assertion helper reuses the existing independent metadata fixture decoder and compares all279 results against typed Rust errors, not merely success versus failure. With fvar validation bypassed but deferred data loading retained, the test fails with214 mismatches. `negative-control.log` records that failure. The source was restored before the passing runs.

The actual-addon scenario uses copied upstream production code only for its oracle. The Rust browser test instantiates `xterm_font_addons::LigaturesAddon`. The shared JavaScript scenario supplies fonts and records joiner ranges, reads and refreshes. It does not implement a font parser or substitute a JavaScript terminal engine. It checks failed-load caching, recovery on a valid family, a warning that corrupts the fvar payload and a warning that changes only the original directory descriptor. The latter must not change the constructor's copied descriptor.

## Commands and current execution

Run from the checkout root with Node24.15.0, Rust/Cargo1.85.1, wasm-bindgen0.2.100 and the existing Playwright installation. Use a private O: target, Cargo jobs2 and the current resource coordinator's browser grant. `SOURCE_PACKAGE_JSON` points to the already installed read-only reference dependency tree.

```sh
node scripts/font-fvar-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --fvar
cargo test --locked -j2 -p xterm-addons --test font_fvar_constructor
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor --test font_buffer_mutations
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_fvar_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_fvar_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

PowerShell users set the corresponding environment variables with `$env:` rather than POSIX shell assignments.

Windows native279-case comparisons pass. The retained four metadata tests, task406 mutation test and seven ligature unit tests pass. Both scoped native and WASM Clippy commands pass with warnings denied. Aggregate workspace formatting reports Windows OS206. All five workspace members pass formatting when invoked separately with `cargo fmt -p PACKAGE -- --check`; this is not labeled an aggregate-command pass.

Both Rust test functions pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine executes the279-case typed constructor comparison and the actual-addon test's five result records. `browser.json` retains the command outcomes and artifact/source hashes. The first browser attempt executed zero Rust tests because wasm-bindgen had not packaged the scenario's transitive helper import. An explicit WASM import now supplies the existing helper through a callback. `browser-packaging-failure.json` preserves that initial failure. No source assertion changed.

Full unchanged suites, real Terminal font/render interactions, native Linux/macOS execution and the final stable workspace matrix remain separate requirements. No performance or full-equivalence claim follows from these constructor cases.

## Artifacts and cleanup

New temporary work belongs only to `O:/xterm-rust-swarm-16/tmp/J`, including target, oracle-generated source copies, browser distributions and TEMP/TMP. Reused dependencies and tools under the reference checkout are read-only and are not part of the deletion set. Source, fixtures and concise logs/reports remain in Git after the owned temporary worktree is removed. No AGENTS.md, credentials, original source or assertions were edited.
