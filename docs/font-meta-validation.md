# Deferred META constructor validation

Task407 implements structural `meta` constructor reads in the Rust ligature font reader. Full font constructor parity remains task342. Overall equivalence is NOT PROVEN.

## Behavior

The source selects the final `meta` directory descriptor without extracting earlier duplicates. It reads the selected descriptor at the end of its OpenType constructor, after kern and GSUB parsing. Rust now records that descriptor, uses the existing deferred SFNT/WOFF extraction path, and validates the version, header words, map records and text payload bounds. Native `FontError::Invalid` retains the source's `Unsupported META table version.` message.

The source reads but does not interpret the flags and string-pool offset. It ignores binary `appl` and `bild` payloads, including invalid payload offsets. Empty text does not dereference its offset. Its decoder reads individual bytes without checking UTF8 validity. Rust preserves those accesses and skips, without allocating unused metadata strings or exposing a new metadata API. Ligature lookup does not consume META values.

Uncompressed tables retain the original remaining DataView bytes, not a nominal directory-length slice. Compressed tables use the decompressed buffer. Earlier name/hmtx/cmap/kern/GSUB failures must precede META errors. GSUB parsing is now separate from FontImpl lookup-tree construction, so tree construction happens only after the final metadata validation, as it does upstream. No dependency, unsafe code or JavaScript parser delegation was added.

The existing native WOFF boundary uses checked zlib decompression. One new corrupt selected compressed META input produces native `FontError::Compression`, while the source's permissive inflater returns bytes that then fail its META version check. Both reject loading, but their failure classifications differ. This distinction is explicitly asserted in the shared test helper, not hidden by a general fallback. Native bounds/missing-table errors also use typed Rust representations rather than JavaScript error objects.

Very large metadata strings can hit engine-specific `String.fromCharCode.apply` argument limits in the source. Rust does not construct those unused strings. This patch does not claim matching engine resource-limit failures. CFF/CFF2, GDEF/GPOS, variation/color constructors, complete outline validation and remaining error-message/order contracts remain342. Accepted task406 supplies warning-driven byte refresh. This branch retains its reviewed source and408 SVG validation. META uses the same refreshed original bytes after GSUB parsing. Two new constructor cases change META version1 to2 and2 to1 during the kern warning, asserting rejection and acceptance respectively. The asynchronous addon scenario also retains the exact parser-buffer alias and changes a valid META header to invalid during the warning.

## Evidence

`results/font-meta/` retains scoped Windows receipts. The installed unchanged opentype.js parser SHA-256 is `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`.

- The source oracle has1830 cases,663 accepted and1167 rejected. The original1471 case descriptors/outcomes are unchanged. The pre-union1828 cases are also unchanged. There are359 new META cases covering versions, truncations, map counts/offsets/lengths, ignored binary tags, malformed UTF8 bytes, duplicate descriptors, compression, nominal lengths and earlier-error precedence. All776 reference-manifest files matched their recorded hashes.
- `baseline-corrected.log` records206 actual pre-fix mismatches against the initial1827-case corpus. The selected corrupt compressed payload and two warning-mutation cases were added afterward, bringing the final count to1830. `baseline.log` is an earlier test-helper compile failure, not a behavior result.
- `native-final.log` records four passing metadata/warning test functions, including the pre-union1828 constructor outcomes and exact typed META/kern failure comparisons. `ligatures.log` records seven passing existing ligature test functions.
- Native xterm-addons all-target/all-feature Clippy and focused xterm-font-addons WASM Clippy pass with warnings denied.
- The exact `cargo fmt --all -- --check` command encounters Windows OS206, even after shortening the owned worktree and trying a temporary drive alias. All five complete per-package checks pass in `fmt-partitions.json`. The Q: alias was removed in `finally`; it is not a retained build dependency.
- `browser-first.json` records Chromium148.0.7778.96 and WebKit26.4 passing two Rust/WASM test functions each on the pre-union1828-case artifact. Firefox exceeded its180-second deadline. That failure is retained, not relabeled as a pass. A separately granted300-second diagnostic checked the same WASM/binding hashes and passed both tests in191.62 seconds on Firefox150.0.2. The diagnostic's first launch had an ESM path error before creating a browser; the corrected launch and actual result are retained separately. `browser-union.json` is the final1830-case union artifact. Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 each pass both Rust/WASM test functions with no failures/ignored/filtered tests. Reported durations are18.59,186.94 and21.92 seconds respectively. This includes the shared warning-to-META addon mutation trace. The validated existing runner uses a300-second test deadline; its default remains unchanged.

The accepted406/408 union on current main `e8011592` passes six focused native test functions, comprising metadata4, mutation1 and SVG1, plus seven existing ligature test functions. All1830 META corpus outcomes,39 mutation constructor traces and183 SVG cases are included. Source oracles for metadata, buffer mutations and SVG pass unchanged checks. Warning-denied native all-target/all-feature addon Clippy and all three focused WASM test Clippy checks pass. All five complete formatting partitions pass. Removing only META's refresh load in a temporary negative control produces exactly two mutation mismatches, then the source is restored in `finally`. `native-union.log` retains a first union compile failure from the renamed test error helper; `native-union-final.log` is the corrected execution.

The asynchronous addon test runs the real Rust font reader and actual addon through a boundary host. It compares shared unchanged-source scenarios for bad head/kern/META input and warning-mutated META, fallback ranges, cached failure, no refresh, successful loading of another family and one refresh. This is component evidence, not a full Terminal renderer or original/demo suite pass.

## Reproduction

Use an owned short O: worktree, Cargo jobs2, the locked toolchain/dependencies, matching wasm-bindgen CLI0.2.100, and the existing installed reference dependencies. Set TEMP first, then TMP and TMPDIR separately to the owned worktree's `tmp/` directory. Source and original assertions remain read-only. Coordinate current heavy/browser grants before launching builds or matrices.

```sh
node scripts/font-metadata-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --metadata
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_metadata_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_metadata_browser WASM_TEST_TIMEOUT_MS=300000 node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

On Windows OS206, run `cargo fmt -p PACKAGE -- --check` once for each of xterm-core, xterm-addons, xterm-web, xterm-wasm-support and xterm-font-addons. Preserve the failed exact command separately. Shell assignments above use POSIX syntax; PowerShell uses `$env:`.

New scratch was owned under `O:/xterm-rust-swarm-16/tmp/A407`, safely moved to shorter `O:/xterm-rust-swarm-16/tmp/A` before final checks. Explicit output and TEMP/TMP/TMPDIR remained in that owned O: tree. Browser runners close their servers and child browsers in finally. Compiled WASM/bindings and private Cargo outputs are disposable; source, regression cases and concise evidence are retained before cleanup. No native Linux/macOS run, final workspace release/quality union or full font equivalence is established here.

## Canonical integration

The unmodified canonical helper integrated accepted406,407 and408 source and their receipts into main `013bf319c86e1860b12d225b075362c46800d55d`, based on `3d755b9d`. It also retained the accepted glyph integration's receipt-only commits, without replaying its source. The14 reviewed407 source/test/fixture hashes remained unchanged after rebasing onto the newer render/search union. `rebase-identity.json` records that audit.

`canonical-merge.log` and `canonical-summary.json` record173 passing default-feature native test functions across41 summaries, with zero failures or ignored tests. The gate built/tested the changed addons/font crates, checked affected native all-target dependents and the workspace, and checked applicable WASM libraries. It removed the private Cargo target afterward. These are not1830 test functions, a complete all-feature workspace matrix, or fresh original browser/demo/platform evidence. Full equivalence remains NOT PROVEN.
