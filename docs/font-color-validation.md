# Eager CPAL and COLR constructor validation

Overall font equivalence is NOT PROVEN. Task442 closes color-table constructor reads and diagnostic phases under342. It does not implement color glyph rendering, palette APIs, variable-font shaping, other missing metadata or the full LigaturesAddon acceptance suites.

## Loader contract

The source parses every CPAL and COLR descriptor eagerly in directory order. Unlike deferred tables, an earlier malformed duplicate is not ignored. Both parsers read the version before warning on any nonzero version. CPAL reads signed header counts and creates the palette-index array before the color array. Negative counts fail with `Invalid array length` at that phase. COLR reads unsigned counts, then base records before layer records. Zero-count loops never dereference their offsets. Directory lengths do not bound uncompressed source DataViews.

The loader now separates fixed initial directory descriptors from the current refreshed payload buffer. Immediate cmap parsing records its decoded state before a later diagnostic can change its bytes. A warning may mutate current table payloads, later metadata or directory bytes. Already consumed versions and decoded metadata stay fixed. Payload reads after a successful callback use refreshed bytes; descriptor selection does not. Already inflated WOFF tables keep their independent bytes. Returning `None` from a later native callback retains the previous refreshed buffer, rather than reverting to the original snapshot. Callback errors abort before later reads and retain the original host error.

Deferred kern, GDEF, GSUB, fvar, avar and META retain their integrated order and callback state. The private DeferredTable no longer stores an unused original reader. Its offset and fixed-descriptor `load_from` operations remain. The accepted420 hint-table hook is not privately replayed here; its eventual union must pass the current payload reader and original descriptor reader.

Early color callbacks exposed a WOFF boundary defect. The source skips the two-byte zlib header and ignores its Adler32 trailer. The old Rust call validated both. The existing miniz dependency now decodes the raw deflate portion after the same two-byte skip. Four retained cases mutate a later compressed cmap header or checksum and still load in both implementations. This does not claim complete malformed-inflater equivalence. The source inflater can tolerate other malformed streams that the checked Rust boundary still rejects.

## Evidence and limits

The oracle calls the unchanged installed opentype.js parser. It retains dependency and Ubuntu Mono SHA256 values, all676 input SHA256 values, expected warnings and constructor outcomes. The corpus has163 accepted inputs,497 rejections and16 warning aborts. It includes SFNT/WOFF headers and record extents, signed counts, empty offsets, eager duplicates, deferred-error precedence, descriptor mutation, earlier/later cmap state, callback aborts and both warning orders. Shared recipes do not contain a JavaScript parser in Rust tests.

Native and WASM tests independently rebuild each input through the existing shared metadata decoder, then reorder only directory records for early-warning probes. Every input must match the source FNV32 before testing. Exact source-compressed payloads avoid a repeated-compression test bottleneck. All672 earlier inputs, outcomes and warning lists remain byte-identical after adding the four compressed-cmap regressions. No assertion was dropped.

On base main3aa04692, the676-case native test passes. The retained GDEF1329, avar477, metadata1830, fvar287, SVG183 and buffer-mutation39 corpora pass through nine other constructor test functions. Seven ligature unit tests also pass. Native all-target/all-feature addon Clippy and focused WASM test Clippy pass with warnings denied. The actual WASM binary builds. These are scoped checks, not a full workspace or native multi-OS result.

Two negative controls demonstrate sensitivity. Skipping color-table parsing yields569 mismatched rows. Restoring the old zlib-wrapper decoder yields exactly four mismatches. Production code was restored and the676-case test passed again. Complete logs are in `results/font-color/`.

The source addon probe has twelve records. It exercises CPAL/COLR rejection caching, successful warning continuation, warning reentry, payload and descriptor mutation, object/number/null/undefined throw retention and good-family recovery. Its expected fixture comes from copied upstream addon production code with dependency resolution adapted, not from Rust output. The browser test creates the actual Rust LigaturesAddon. Both test functions pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 with no failed, ignored or filtered tests. Each engine compares all676 constructor inputs and twelve addon records. The original60-second watchdog is unchanged. `browser.json` records exact test, compiler, WASM, binding and CSS hashes; generated executables are temporary test artifacts, not a claimed release distribution.

Exact `cargo fmt --all -- --check` fails on this physical Windows path with OS206. Separate checks pass for all five workspace members. That is not an aggregate formatting pass. Full workspace gates, native Linux/macOS, original font suites, Terminal rendering and addon distribution acceptance remain open.

## Reproduction

Use a private O: target, Cargo jobs2 and a current compiler/browser resource grant.

```sh
node scripts/font-color-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --color
cargo test --locked -j2 -p xterm-addons --test font_color_constructor
cargo test --locked -j2 -p xterm-addons --test font_gdef_constructor --test font_avar_constructor --test font_metadata_constructor --test font_svg_constructor --test font_fvar_constructor --test font_buffer_mutations
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_color_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_color_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

PowerShell users set environment variables through `$env:`. Node24.15.0, Rust/Cargo1.85.1 and wasm-bindgen0.2.100 are the local tools. Browser versions and exact artifact hashes are retained with the final matrix receipt.

## Ownership

Only O:/xterm-rust-swarm-16/tmp/Cao442 is owned temporary work. It includes the private target, scratch and generated browser artifacts. Reference dependencies and wasm-bindgen are read-only reused paths in the original P: checkout. Preserve source, fixture and concise receipts in Git before removing owned outputs. Never traverse reference links, modify AGENTS.md or remove another owner's files.
