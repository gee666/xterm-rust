# TrueType constructor boundaries

Task453 implements the missing TrueType constructor reads in the Rust font reader. Overall equivalence remains NOT PROVEN.

The source is the supplied opentype.js distribution used by the unchanged ligatures addon. Source and UbuntuMono fixture SHA256 hashes, deterministic seed and exact input reconstruction hashes are retained in `results/font-outline/oracle.json` and the constructor corpus.

## Contract

The constructor selects the final glyf and loca directory descriptors. When both exist, it parses loca before preparing lazy glyph loaders, then reads hmtx and assigns cmap names. A CFF or CFF2 fallback is selected only when the TrueType pair is absent. This change does not implement those CFF readers.

The final parsed head.indexToLocFormat strictly equal to zero selects two-byte loca entries. Any other value, including missing head, selects four-byte entries. With maxp present, loca reads numGlyphs plus one entries, even when numGlyphs is zero. Missing maxp leaves the source count undefined and causes no loca entry reads.

Uncompressed reads use the complete original DataView, ignoring the directory's nominal length. A zero-read table does not dereference its offset. WOFF decompression is still performed for loca and glyf before their consumers, even when no body bytes are read. Arbitrary loca offset values and malformed glyph bodies remain lazy and do not cause constructor rejection. Rejecting those values eagerly would be incompatible.

Descriptors and already parsed head/count values stay fixed across earlier warning callbacks. New payload reads use the refreshed buffer from accepted442. An already decompressed table has independent owned bytes. The hmtx boundary shares the same small helper instead of duplicating compression and zero-read behavior.

## Scope and evidence

The source corpus has612 cases,43 accepted and569 rejected, including128 seeded cases with seed0x10CA453. It covers short/long/unknown head formats, EOF boundaries, missing metadata, zero and65535 glyph counts, final descriptor selection, ignored lengths, lazy offset/body acceptance, failure ordering against name/hmtx/kern/GDEF/GSUB/META, and earlier CPAL warning mutations. The source hash is737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484.

The same recipes independently reconstruct bytes in Node and Rust. Every case checks an input hash before comparing exact typed failure causes and synchronous warning messages. Uniform byte arrays use compact fill recipes to avoid retaining megabytes of repeated zeroes.

Four unchanged-source addon scenarios exercise real loading failure, fallback ranges, cached rejection, recovery with a good font, and acceptance of lazy offsets or empty glyph bodies. They are component tests using the real addon and font parser, not complete Terminal rendering tests.

Source commands, run from the owned worktree with SOURCE_ROOT and SOURCE_PACKAGE_JSON pointing to the original read-only reference:

```sh
node scripts/font-outline-oracle.mjs --source=/path/to/source_repo
node scripts/font-outline-addon-oracle.mjs --source=/path/to/source_repo
cargo test -p xterm-addons --test font_outline -j2
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_outline_browser node scripts/browser-host-check.mjs
```

Use --generate only when intentionally constructing reviewed new source-derived inputs, not to hide a mismatch. On Windows with rustc/cargo1.85.1, all178 addon tests across36 targets passed with all features, no failures, ignored tests or warnings. Native addon all-target/all-feature Clippy and focused WASM Clippy passed with warnings denied. The two-test WASM binary compiled. Bypassing only the outline hook produced383 corpus mismatches; restoring it passed. Exact commands and logs are under results/font-outline. Both WASM tests passed in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Each engine replayed all612 constructor recipes and the four actual-addon scenarios, with2 passed functions,0 failures, ignored or filtered tests. results/font-outline/browser.json records the versions and hashes; retainedArtifacts points to durable compiler, generated WASM, binding JavaScript and CSS bytes. The isolated first review found no code defect and independently reran the source corpus/addon scenarios and pinned native executable. Its only requested change was the then-pending browser execution. The resumed isolated review accepted453 after verifying all four browser artifact hashes, source/native/test/Cargo.lock pins and rerunning the native corpus. No concrete defect remained. This is scoped acceptance, not full342 closure or final integration. Full cargo fmt failed with OS206; all five complete member checks passed. Per-member results do not replace the full gate.

## Integration

The private453 branch is stacked on accepted442 tip5f01a73e. Its first453 commit is6762ac34. Do not replay standalone pre-stack aa444356. Integration must apply442 separately before the new453 delta and preserve accepted420 hints,438 GDEF/avar and447 GPOS when unioning font_bytes. The outline hook uses the current payload reader before hmtx.

The implementation has no unsafe code, new dependency or JavaScript parser delegation. Glyph path rendering, CFF/CFF2, other optional metadata, full source suites, final workspace/native OS gates and full port equivalence remain unfinished.
