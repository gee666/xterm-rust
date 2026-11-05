# Deferred GSUB directory selection

Task374 fixes a constructor acceptance mismatch on main `ff0bec8`. Full font and port equivalence remain NOT PROVEN. Task342 retains complex optional metadata parsing and broader error/phase parity.

## Defect and fix

The unchanged opentype.js constructor records the final GSUB directory descriptor without reading earlier GSUB data. It reads/decompresses that selected table later. Rust's reader instead read every descriptor's payload and only then retained the last payload. An invalid ignored entry therefore rejected a font that the source accepted.

`font_bytes.rs` now handles GSUB alongside the already-deferred name descriptors. It selects the final descriptor during the directory scan and reads its payload only after the existing required-table checks. This also avoids decompressing ignored data. No production dependency or browser delegation was added.

The source corpus adds seven GSUB inputs to the existing878 cases. Two valid empty GSUB tables establish positive SFNT/WOFF construction. Four cases distinguish ignored malformed first descriptors from selected malformed final descriptors. One WOFF case contains invalid compressed bytes in an ignored first descriptor, at a valid in-bounds offset. It proves decompression is deferred, not merely offset validation.

On unchanged Rust main, three accepted-source cases failed: the ignored invalid SFNT offset, ignored invalid WOFF offset, and ignored invalid compressed WOFF data. `results/font-gsub-directory/before.log` preserves that failing run. `after.log` and `native-final.log` preserve passing runs after the fix. No original assertion was weakened or removed.

## Shared native and browser evidence

The fixture reconstruction code moved to `crates/xterm-addons/tests/support/font_metadata_data.rs`, shared by native and actual WASM tests. The JS oracle independently constructs the corresponding source inputs. The browser test invokes the real Rust `Font::load_buffer`, not a JavaScript parser or terminal engine.

`serde_json` and `miniz_oxide` are already-locked repository dependencies. They are now also target-specific development dependencies of xterm-font-addons, needed to read the same fixture and reconstruct compressed WOFF inputs in the browser test. Cargo.lock gains only those package references; no dependency versions changed. Neither dependency was added to that crate's production dependency list.

Observed on Windows, Rust/Cargo1.85.1, Node24.15.0:

- Source oracle:885 cases,303 accepted,582 rejected,14 WOFF inputs. Source and base-font hashes remain recorded in `results/font-metadata/oracle.json`.
- Native constructor tests:2 passed, including all885 outcome comparisons and typed-error checks.
- Existing ligature tests:7 passed, retaining original/seeded font output, graph, alias/cache and WOFF checks.
- Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4:2 Rust test functions passed per engine. One runs all885 constructor cases; one compares the original async addon-load/fallback/refresh trace.
- Warning-denied xterm-addons all-target/all-feature Clippy and focused font-addon WASM test Clippy passed.
- Exact workspace `cargo fmt --all -- --check` passed from the regular short O: worktree. No junction or drive mapping was used.

The initial WASM test compilation identified a missing direct test-only miniz dependency in the new shared reconstruction module. `wasm-build-initial.log` retains that error; `wasm-build.log` retains the corrected build. The browser runner owns and closes its server/browsers in finally cleanup. `browser-summary.json` retains raw browser outcomes, source/artifact hashes and execution-time paths; generated binaries are temporary, not committed.

Commands from the worktree, with Cargo jobs2 and TEMP/TMP under its O: scratch directory:

```sh
node scripts/font-metadata-oracle.mjs --source=PATH_TO_REFERENCE
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_metadata_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

Use the existing documented wasm-bindgen0.2.100 and Playwright paths for the browser runner. The shell assignments above use POSIX syntax; PowerShell users set the corresponding `$env:` variables.

This change does not fix the source's other eager metadata parsers, malformed cmap/GSUB error precedence, uncompressed declared-length differences, or all font-loading/rendering APIs. It is an acceptance repair with native and three-engine evidence, not a full font or workspace completion claim.
