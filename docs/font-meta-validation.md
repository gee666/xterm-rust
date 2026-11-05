# Deferred META constructor validation

Task407 implements structural `meta` constructor reads in the Rust ligature font reader. Full font constructor parity remains task342. Overall equivalence is NOT PROVEN.

## Behavior

The source selects the final `meta` directory descriptor without extracting earlier duplicates. It reads the selected descriptor at the end of its OpenType constructor, after kern and GSUB parsing. Rust now records that descriptor, uses the existing deferred SFNT/WOFF extraction path, and validates the version, header words, map records and text payload bounds. Native `FontError::Invalid` retains the source's `Unsupported META table version.` message.

The source reads but does not interpret the flags and string-pool offset. It ignores binary `appl` and `bild` payloads, including invalid payload offsets. Empty text does not dereference its offset. Its decoder reads individual bytes without checking UTF8 validity. Rust preserves those accesses and skips, without allocating unused metadata strings or exposing a new metadata API. Ligature lookup does not consume META values.

Uncompressed tables retain the original remaining DataView bytes, not a nominal directory-length slice. Compressed tables use the decompressed buffer. Earlier name/hmtx/cmap/kern/GSUB failures must precede META errors. GSUB parsing is now separate from FontImpl lookup-tree construction, so tree construction happens only after the final metadata validation, as it does upstream. No dependency, unsafe code or JavaScript parser delegation was added.

The existing native WOFF boundary uses checked zlib decompression. One new corrupt selected compressed META input produces native `FontError::Compression`, while the source's permissive inflater returns bytes that then fail its META version check. Both reject loading, but their failure classifications differ. This distinction is explicitly asserted in the shared test helper, not hidden by a general fallback. Native bounds/missing-table errors also use typed Rust representations rather than JavaScript error objects.

Very large metadata strings can hit engine-specific `String.fromCharCode.apply` argument limits in the source. Rust does not construct those unused strings. This patch does not claim matching engine resource-limit failures. CFF/CFF2, GDEF/GPOS, variation/color constructors, complete outline validation and remaining error-message/order contracts remain342. Warning-driven mutation of deferred bytes is separate task406 and needs a combined integration test with META after that fix lands.

## Evidence

`results/font-meta/` retains scoped Windows receipts. The installed unchanged opentype.js parser SHA-256 is `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`.

- The source oracle has1828 cases,662 accepted and1166 rejected. The original1471 case descriptors/outcomes are unchanged. There are357 new META cases covering versions, truncations, map counts/offsets/lengths, ignored binary tags, malformed UTF8 bytes, duplicate descriptors, compression, nominal lengths and earlier-error precedence. All776 reference-manifest files matched their recorded hashes.
- `baseline-corrected.log` records206 actual pre-fix mismatches against the initial1827-case corpus. The selected corrupt compressed payload was added afterward, bringing the final count to1828. `baseline.log` is an earlier test-helper compile failure, not a behavior result.
- `native-final.log` records four passing metadata/warning test functions, including all1828 constructor outcomes and exact typed META/kern failure comparisons. `ligatures.log` records seven passing existing ligature test functions.
- Native xterm-addons all-target/all-feature Clippy and focused xterm-font-addons WASM Clippy pass with warnings denied.
- The exact `cargo fmt --all -- --check` command encounters Windows OS206, even after shortening the owned worktree and trying a temporary drive alias. All five complete per-package checks pass in `fmt-partitions.json`. The Q: alias was removed in `finally`; it is not a retained build dependency.
- `browser-first.json` records Chromium148.0.7778.96 and WebKit26.4 passing two Rust/WASM test functions each. Firefox exceeded its180-second deadline. That failure is retained, not relabeled as a pass.

The asynchronous addon test runs the real Rust font reader and actual addon through a boundary host. It compares shared unchanged-source scenarios for bad head/kern/META input, fallback ranges, cached failure, no refresh, successful loading of another family and one refresh. This is component evidence, not a full Terminal renderer or original/demo suite pass.

## Reproduction

Use an owned short O: worktree, Cargo jobs2, the locked toolchain/dependencies, matching wasm-bindgen CLI0.2.100, and the existing installed reference dependencies. Set TEMP first, then TMP and TMPDIR separately to the owned worktree's `tmp/` directory. Source and original assertions remain read-only. Coordinate current heavy/browser grants before launching builds or matrices.

```sh
node scripts/font-metadata-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --metadata
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_metadata_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser WASM_TEST_TIMEOUT_MS=180000 node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

On Windows OS206, run `cargo fmt -p PACKAGE -- --check` once for each of xterm-core, xterm-addons, xterm-web, xterm-wasm-support and xterm-font-addons. Preserve the failed exact command separately. Shell assignments above use POSIX syntax; PowerShell uses `$env:`.

New scratch was owned under `O:/xterm-rust-swarm-16/tmp/A407`, safely moved to shorter `O:/xterm-rust-swarm-16/tmp/A` before final checks. Explicit output and TEMP/TMP/TMPDIR remained in that owned O: tree. Browser runners close their servers and child browsers in finally. Compiled WASM/bindings and private Cargo outputs are disposable; source, regression cases and concise evidence are retained before cleanup. No native Linux/macOS run, final workspace release/quality union or full font equivalence is established here.
