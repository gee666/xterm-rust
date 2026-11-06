# Eager CFF2 FDSelect validation

Task559 follows accepted546. The source parses FDSelect only after every FDArray font/private dictionary and local Subrs INDEX has finished. This patch adds that eager constructor phase. Charstrings, CFF1 dictionaries and the other342 families remain open. Overall equivalence is NOT PROVEN.

The Rust validator implements formats0/3/4, preserves endpoint-before-index validation reads, exact case-sensitive diagnostics and source spelling "Sentinal", and accepts equal/descending ranges. The source uses format in the initial-GID message but CFF version in the out-of-range-GID message. A missing maxp leaves numGlyphs undefined. Format0 then reads no glyph entries, and zero-range formats compare undefined next against undefined numGlyphs. Option<u16> retains this distinction from zero glyphs.

DICT operands retain numeric or array values. Parser start-plus-relative arithmetic stays numeric for scalar operands and concatenates for array operands before DataView ToIndex. Byte/word reads share the existing whole-font reader and bounded compressed replay, including offsets beyond32768. FDArray counts are read only after earlier font processing succeeds. The ligature constructor does not use the expanded glyph-to-FD array, so Rust validates its reads/errors without allocating repeated entries. This is not a source resource-error equivalence claim.

A typed public FdSelectError carries bad formats, FD indexes/counts, initial/range GIDs and missing/final sentinels. FontError wraps it and keeps the source's diagnostic text. No dependency, unsafe code, public JS font-engine delegation or source modification is introduced.

Tests reuse the existing independent SFNT/WOFF writer and addon lifecycle scenario. The per-corpus browser driver deliberately follows the predecessor runner so its raw results remain independently attributable; the shared server, container writer and lifecycle logic are not copied. Existing predecessor assertions stay intact. The fixed277 recipes cross CFF/CFF2 tags and SFNT/compressed WOFF containers. They cover format errors, truncations, validation precedence, descending ranges, missing maxp/name/hmtx, font errors before FDSelect, scalar/fractional/NaN/array offsets and compressed prefix crossings. Four controls retain the unchanged good font. Wide format4 cases use FD256/count257, initialGID65536, rangeCount65536 and endpoint16777217, include truncation within the two-byte FD/four-byte endpoint, and select a valid second font. A [] address succeeds through subsequent concatenated Parser reads in a full SFNT constructor.

Source oracle generation/replay, native/WASM gates and three-engine constructors/addon tests are pending. Scope is Windows and browser components only. No full-workspace/native Linux/macOS or full upstream suite claim.

Commands use a private O:target/TEMP/TMP, Cargo-j2 and the coordinator's bounded heavy-job grant. Set SOURCE_PACKAGE_JSON to the original read-only source_repo/package.json, PLAYWRIGHT_MODULE to the installed reference playwright/index.mjs, and WASM_BINDGEN to the matching0.2.100 CLI.

```
node scripts/font-cff-fdselect-oracle.mjs --generate
node scripts/font-cff-fdselect-oracle.mjs
cargo test -j2 -p xterm-addons --all-targets --all-features
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-font-addons --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
node scripts/font-cff-fdselect-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff_fdselect_browser WASM_TEST_TIMEOUT_MS=600000 node scripts/browser-host-check.mjs
```

Only O:/xterm-rust-swarm-16/tmp/worktrees/JulieFdSelect is owned temporary work. Base224d1b33 already includes533 source. Accepted546 is copied privately as899f5028/bff3630a/5ec13162. Exclude all three from new559 pickup. The initially attempted533 replay conflicted and was aborted before any source edits. No main or reference state changed.
