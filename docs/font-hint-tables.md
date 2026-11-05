# Eager font hint-table construction

Task420 closes the skipped `cvt `, `fpgm` and `prep` constructor reads under342. It does not implement glyph hint execution or complete broader font metadata, rendering or addon equivalence.

The supplied opentype.js visits these tables eagerly in directory order. `fpgm` and `prep` read the recorded number of bytes. `cvt ` allocates an array of half the recorded length and reads signed words. An odd length raises `RangeError: Invalid array length` before data reads. Every duplicate is visited, so a later valid table cannot hide an earlier invalid one. A zero-element uncompressed list never dereferences its offset. Uncompressed reads may consume bytes after the nominal table, since the source uses the whole-font DataView.

Rust validates these reads without retaining unused hint arrays. `FontError::ArrayLength` distinguishes the odd-length failure, and the browser boundary maps it to RangeError. No JavaScript font parser, unsafe code or dependency was added.

## WOFF behavior and review corrections

Decompression precedes list allocation. The source skips the zlib header, ignores its checksum and does not enforce the decompressed output length. Short output stays zero-filled and overflow disappears into out-of-bounds typed-array writes. A standards-strict decoder therefore rejects finite inputs that the source accepts.

Independent review exposed two such failures. A final stored block with a complete header and missing payload is accepted by the source. A partially missing inverse-length header uses JavaScript arithmetic, not simple zero padding. For example, `256 * undefined + 1` is NaN, and `~NaN & 65535` is 65535. A stored length of 65535 therefore passes the source's comparison even when the present inverse low byte is1. In both cases, a cvt directory length of65 must reach ArrayLength after decompression rather than fail earlier as Compression.

The initial strict miniz validator and the subsequent virtual-zero patch are retained as negative controls. The hint-only helper in `font_hint_deflate.rs` walks stored, fixed and dynamic deflate syntax using the source's semantics. It retains no decompressed output, allocates no hint-sized storage and does not require valid backreference history. Stored header arithmetic distinguishes missing words from zero words. Bitwise reads alone turn missing bytes into zero. Source-reserved fixed symbols retain their original read phases.

A third preflight found a buffered block-transition mismatch. The source refills its tag to at least24 bits, consumes a Huffman symbol without refilling between bits, and rewinds stored input only while more than8 buffered bits remain. An exact8-bit remainder therefore skips one byte instead of using canonical byte alignment. Rust now models the source tag, bit count, refill, header-bit and rewind operations directly. The corpus pins both observed mismatch directions and all eight residual bit positions. An exact old canonical-reader negative control fails the retained three-literal transition before restoration.

This is separate from the existing strict loader for tables whose data is used. The source-derived helper carries the tiny-inflate MIT attribution in `licenses/tiny-inflate-MIT.txt`, obtained from `https://unpkg.com/tiny-inflate@1.0.3/LICENSE`, and the existing opentype.js notice. No new package is used.

## Inputs and checks

`crates/xterm-addons/tests/fixtures/font-hint-tables.json` is an intentionally single-line generated fixture. It retains 792 finite constructor outcomes, input hashes and 792 asynchronous addon records, with377 accepted and415 rejected. Cases cover SFNT/raw WOFF bounds, odd lengths, empty lists, invalid offsets, duplicates, eager head-error precedence, later-byte reads, malformed compressed input, ignored wrapper bytes, short output, overflow, stored-header truncation, NaN inverse headers, mixed stored/fixed prefixes, invalid history, reserved symbols and loading-cache behavior. Dynamic Huffman inputs use seed `0x420d3f1a`. A separate multi-stored-block input tests block transitions.

JavaScript and Rust reconstruct inputs independently, and the WASM test compares every reconstructed byte. The Node oracle uses the existing in-memory TypeScript loader, with directory-index resolution for the unchanged addon's `import '.'`. Production source and original assertions are not rewritten. Browser baseline bundling also uses unchanged source. The Rust browser binary invokes the actual LigaturesAddon with a recorded terminal boundary, not a full interactive Terminal rendering test.

The final source Node corpus passes. Native792 plus the separate safety guard pass; all-target/all-feature addon tests pass174 tests across32 summaries, and scoped native/WASM warning-denied Clippy and both web/font dependent WASM library checks pass. Final792-case browser matrices are pending. Earlier528/594-case matrices and2078-test workspace receipts belong to earlier decoder revisions, not the final buffered syntax helper.

Raw evidence is under `results/font-hint-tables/`. Initial receipts cover the528-case corpus and2072-test checkpoint. `union-` receipts and `pre-review/` cover the first rebased528-case version. `repair-` receipts cover the594-case padding version. `syntax-` receipts cover the696-case canonical reader, while `buffered-` receipts cover the current792-case source-buffered walker. Browser reports retain compiled artifact hashes; disposable local binaries are not distribution artifacts and are removed during owned cleanup. Compilation is not counted as a browser pass.

## Separate source nontermination case

The fixture's `nontermination` input has empty dynamic Huffman trees. The original source repeatedly consumes virtual zero bits without reaching an end symbol. `font-hint-nontermination.mjs` records a separate two-second observation after the worker announces parser entry, then terminates and awaits that owned worker. The timeout alone is not a mathematical proof; the source loop is also inspected.

Rust detects the repeated zero-input symbol state and returns `FontError::Compression` instead of hanging. The native and WASM corpus test this guard separately from the792 finite source outcomes. This safety divergence is explicit, not an equivalence pass. Malformed dynamic-code cache reuse across separate source parses and arbitrary hostile streams remain broader342 work.

## Reproduce

Use a private O: worktree, its regular target and owned `tmp/`. Set `SOURCE_PACKAGE_JSON` and `PLAYWRIGHT_MODULE` to existing read-only dependency installations, and `WASM_BINDGEN` to the installed CLI matching0.2.100. Observe the four-heavy/two-browser cap and Cargo jobs2.

```sh
node scripts/font-hint-tables-oracle.mjs
node scripts/font-hint-nontermination.mjs
node scripts/font-hint-tables-browser-source.mjs
cargo test --locked -j2 -p xterm-addons --test font_hint_tables --test font_metadata_constructor
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_hint_tables_browser WASM_TEST_TIMEOUT_MS=180000 node scripts/browser-host-check.mjs
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_hint_tables_browser -- -D warnings
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo fmt --all -- --check
```

The aggregate fmt command has failed with Windows OS206, while individual checks for all five workspace packages pass. These are scoped Windows results, not final stable-union or native Linux/macOS evidence. Both original and copied reference manifests matched all776 hashes before the syntax repair and are rechecked before cleanup.

`--generate` deliberately regenerates the Node source fixture. It is not used to conceal a Rust failure. Browser baseline execution closes its browser/server and removes the owned source bundle in finally blocks. Temporary artifacts remain owned under `O:/xterm-rust-swarm-16/tmp/G420` until review/integration pickup and verified cleanup.

Task342 retains CFF/CFF2, GDEF/GPOS, color tables, variation families and broader malformed/error behavior. Full original addon/Terminal/demo acceptance, stable quality gates, supported feature combinations, native OS evidence and required comparisons remain unfinished. Overall equivalence remains NOT PROVEN.
