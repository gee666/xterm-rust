# Eager font hint-table construction

Task420 closes the skipped `cvt `, `fpgm` and `prep` constructor reads under342. It does not implement glyph hint execution or complete broader font metadata, rendering or addon equivalence.

The supplied opentype.js visits these tables eagerly in directory order. `fpgm` and `prep` read the recorded number of bytes. `cvt ` allocates an array of half the recorded length and reads signed words. An odd length raises `RangeError: Invalid array length` before data reads. Every duplicate is visited, so a later valid table cannot hide an earlier invalid one. A zero-element uncompressed list never dereferences its offset. Uncompressed reads may consume bytes after the nominal table, since the source uses the whole-font DataView.

Rust validates these reads without retaining unused hint arrays. `FontError::ArrayLength` distinguishes the odd-length failure, and the browser boundary maps it to RangeError. No JavaScript font parser, unsafe code or dependency was added.

WOFF decompression runs before list allocation. Unlike the shared strict WOFF table loader, the source hint reader skips the zlib header and ignores its checksum and decompressed length. Short output stays zero-filled and overflow disappears into out-of-bounds typed-array writes. The hint validator uses the existing miniz raw streaming decoder and discards output in fixed-size chunks. It does not allocate storage proportional to the recorded list length. Each decoder iteration must consume input, emit output or finish. Failure and no progress return a typed compression error.

The source reads missing stored-block payload as undefined, which its typed-array writes convert to zero. The validator supplies a finite virtual-zero tail after the actual compressed bytes. Its limit is the protocol's maximum 16-bit stored payload plus the four-byte stored header, enough to finish that final block or reject a later all-zero header. It does not follow the source into unbounded virtual input. This distinction is local to discarded hint data, not a change to all other font tables.

## Inputs and checks

`crates/xterm-addons/tests/fixtures/font-hint-tables.json` is an intentionally single-line generated fixture. It retains 594 constructor outcomes, input hashes and 594 asynchronous addon records, with 282 accepted and 312 rejected. Cases cover SFNT/raw WOFF bounds, odd lengths, empty lists, invalid offsets, duplicates, eager head-error precedence, reads into later bytes, malformed compressed input, ignored wrapper bytes, short output, overflow, missing stored-block bytes and source loading-cache behavior. JavaScript and Rust reconstruct inputs independently. The WASM test compares every reconstructed byte.

The Node oracle uses the existing in-memory TypeScript loader, with directory-index resolution added for the unchanged addon's `import '.'`. Production source and original assertions are not rewritten. Browser baseline bundling also uses unchanged source. The Rust browser binary invokes the actual LigaturesAddon with a recorded terminal boundary. This is not a full interactive Terminal rendering test.

The final source baseline passes all 594 constructor outcomes and 594 addon records in Node and each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. The actual Rust browser binary passes two test functions per engine. They replay all 594 constructors, compare both input builders byte-for-byte and check all 594 real addon load/cache records.

On base main926d8355, which includes accepted406/407/408, the final repaired workspace passes 2078 tests across 433 summaries, with zero failures or ignored tests. Native workspace and focused WASM Clippy pass with warnings denied. The aggregate fmt command fails with Windows OS206, while each of all five package checks passes. Both original and copied reference manifests match all 776 hashes. These are scoped Windows checkpoints, not final stable-union or native Linux/macOS evidence.

Independent review found that the first strict decoder rejected finite missing stored-block payloads accepted by the source, and replaced odd cvt's ArrayLength error with Compression. The fixture adds 66 exact header/payload truncation and maximum-length cases, including both reported 64/65-length examples. The exact original decoder fails the retained `cvt  WOFF stored truncated5 length64` case. Restoration of the bounded virtual-zero repair passes all 594. A separate negative control removes only the eager hook and fails at `cvt  SFNT bytes0 length1`, then restores the source in finally.

Read `results/font-hint-tables/` for raw logs, browser reports and producer hashes. Initial receipts cover the earlier 528-case corpus and 2072-test checkpoint. `union-` receipts and `pre-review/` browser records cover the first rebased 528-case version. `repair-` receipts and current browser reports cover the final 594-case version. Browser reports retain exact compiled artifact hashes, but disposable local binaries are not distribution artifacts and are removed during owned cleanup. Compilation is not counted as a browser pass.

## Reproduce

Use a private O: worktree, its regular target and owned `tmp/`. Set `SOURCE_PACKAGE_JSON` and `PLAYWRIGHT_MODULE` to existing read-only dependency installations, and `WASM_BINDGEN` to the installed CLI matching 0.2.100. Observe the four-heavy/two-browser cap and Cargo jobs2.

```sh
node scripts/font-hint-tables-oracle.mjs
node scripts/font-hint-tables-browser-source.mjs
cargo test --locked -j2 -p xterm-addons --test font_hint_tables --test font_metadata_constructor
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_hint_tables_browser WASM_TEST_TIMEOUT_MS=180000 node scripts/browser-host-check.mjs
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_hint_tables_browser -- -D warnings
cargo test --locked -j2 --workspace --all-targets --all-features
cargo fmt --all -- --check
```

`--generate` deliberately regenerates the Node source fixture. It is not used to conceal a Rust failure. The browser baseline runner closes its browser/server and removes its owned source bundle in finally blocks. Temporary build artifacts remain owned under `O:/xterm-rust-swarm-16/tmp/G420` until review/integration pickup and verified cleanup.

## Remaining scope

Task342 retains CFF/CFF2, GDEF/GPOS, color tables, variation families and broader malformed/error behavior. Arbitrary hostile deflate streams and their source nontermination cases are not proven equivalent by this corpus. The decoder remains safe and progress-checked. No source hang is deliberately reproduced. Full original addon/Terminal/demo acceptance, stable quality gates, supported feature combinations, native OS evidence and required comparisons remain unfinished. Overall equivalence remains NOT PROVEN.
