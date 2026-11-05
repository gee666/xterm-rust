# TrueType horizontal metric validation

Task381 fixes a constructor rejection gap left after fixed metadata339 and deferred GSUB374. Full font and port equivalence remain NOT PROVEN; task342 retains complex metadata and CFF-specific horizontal-metric validation.

## Source behavior and repair

The source constructor selects the final hmtx descriptor, then reads it after name/outlines processing. For TrueType fonts it reads one four-byte advance-width/side-bearing pair while the glyph index is below both maxp.numGlyphs and hhea.numberOfHMetrics. It reuses the last pair for subsequent glyphs without reading their remaining side-bearing bytes. Rust must reproduce these source reads rather than enforce a stricter OpenType-spec table length.

The old Rust reader only required an hmtx directory entry. It accepted truncated metric bytes and a selected out-of-bounds descriptor that the source rejected. `font_bytes.rs` now records the final descriptor and validates the source-required metric bytes for the TrueType outline path. Missing hhea behaves like the source's undefined metric count, which makes its numeric comparison false. Compressed WOFF data is still decompressed before the glyph loop, even when that loop reads no metric bytes.

Zero reads matter. With zero metrics, the source never dereferences an uncompressed hmtx offset, even if it points outside the file. Rust preserves this acceptance. However, an invalid compressed WOFF offset still fails during decompression preparation. For nonzero reads, uncompressed directory lengths do not constrain the original DataView; following bytes may satisfy the read. The regression corpus includes both SFNT and WOFF versions of that distinction.

This implementation shares the existing bounded WOFF extraction path and introduces no dependencies, unsafe code, browser delegation or successful placeholder behavior. CFF/CFF2 glyph counts and lazy glyph-outline failures remain task342, not silently claimed by this TrueType check.

## Evidence and counts

The unchanged-source oracle adds65 deterministic hmtx inputs to the earlier885 cases. The matrix varies metric counts0/1/2/65535 and byte lengths0/1/3/4/7/8/5184 across SFNT and WOFF, plus duplicate descriptors, zero-read invalid offsets, compressed preparation and ignored nominal lengths.

`results/font-hmtx/before.log` records31 source-rejected inputs accepted by the previous Rust constructor. That initial failing run had948 cases. Two accepted directory-length cases were then added, producing the final950-case corpus. No earlier case or assertion was removed or weakened. `native-final.log` records the corrected950-case run.

Observed Windows evidence:

- Source oracle:950 case outcomes,337 accepted and613 rejected,47 WOFF cases. Source/font/fixture hashes are in `results/font-metadata/oracle.json`.
- Native metadata:2 Rust test functions passed, including all950 outcome comparisons and explicit bounds/type/missing-table assertions.
- Existing ligature regressions:7 Rust test functions passed, including seeded font outputs, graph/cache/alias checks and WOFF decoding.
- Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4:2 Rust test functions passed per engine. One executes all950 constructor comparisons, the other checks original async addon fallback/failure/cache/refresh behavior.
- Warning-denied all-target/all-feature xterm-addons Clippy, focused font-addon WASM test Clippy and exact workspace formatting passed.

The shared Rust fixture reconstruction module introduced in374 is used by native and browser tests. The JavaScript oracle constructs source inputs independently and invokes the unchanged installed opentype.js parser. Browser tests call the actual Rust font reader. Raw runner output and source/artifact hashes are preserved in `results/font-hmtx/browser-summary.json`. Generated binaries are temporary and execution-time paths may disappear after owned cleanup.

Commands use the same documented tool/reference setup as `docs/font-gsub-directory.md`:

```sh
node scripts/font-metadata-oracle.mjs --source=PATH_TO_REFERENCE
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_metadata_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

All new scratch/build output stayed in the owned `O:/xterm-rust-swarm-16/tmp/L374` worktree with Cargo jobs2. Browser runners close their own servers/browsers. This scoped evidence is not a final native-platform/workspace release matrix, complete original addon suite result, or proof of full font parity.
