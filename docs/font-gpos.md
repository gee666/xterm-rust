# Deferred GPOS constructor validation

Task447 closes a scoped gap under task342. Overall equivalence remains NOT PROVEN.

The source's OpenType constructor parses the final GPOS descriptor after kern/GDEF and before GSUB/fvar. Rust previously ignored it. `font_gpos.rs` now validates version, script/feature/lookup records, single and pair adjustment values, coverage/class definitions, filtering-set fields and the source's shallow feature-variation records. `font_gpos_layout.rs` retains the parsed values needed for `Position.init`, including the source binary search and failures for absent referenced feature/lookup records.

These tables do not drive the source addon ligature algorithm. Source lookup types3 through9 return diagnostic objects without reading their payload. The Rust validator deliberately does not reject those payloads. Device offsets and variation-record child offsets are likewise not followed by this source version. Zero pointers skip children. Zero-length lists read no payload. Logical offsets use u64 so WASM32 can retain an out-of-range ignored pointer without prematurely dereferencing it.

Class warnings reuse the existing `FontWarning::InvalidClassDefinition`. A callback may throw or mutate the original font buffer. Later uncompressed reads observe the refreshed snapshot while already parsed headers/counts/flags/value formats and directory descriptors remain unchanged. Compressed tables retain their independent inflated bytes. No callback holds a RefCell borrow. No unsafe code, new dependency, source engine delegation or reference assertion change was added.

## Current evidence

Base is actual main `3aa04692`, not the legacy P: checkout. Accepted task420 remains on `swarm/G420`, unintegrated in this producer. Task442's eager color loader and task453's TrueType outline work are separate owned changes. Their future union must preserve the GPOS descriptor and after-GDEF/before-GSUB hook plus current refreshed bytes.

- `node scripts/font-gpos-oracle.mjs` executes the unchanged installed opentype.js constructor. Current fixture has 1,571 inputs, 422 accepted, 1,141 ordinary failures and eight thrown-warning failures. Seed `0x4476a05`, 128 seeded cases. Source, base font, fixture and each constructed input are hash-bound.
- `cargo test -p xterm-addons --test font_gpos_constructor -j2` passes one Rust test function containing all 1,571 comparisons. The native API's existing checked bounds and missing-table variants are mapped explicitly to source failure classes. GPOS format/version/lookup messages, warning strings and phase ordering compare directly.
- `node scripts/font-gpos-addon-oracle.mjs` passes all 1,571 actual unchanged-source addon load/cache cases. Independent Node and browser input builders compare byte-for-byte. Loading fallback, success/failure ranges, refresh count, blob-read count, cached-repeat behavior and warnings are retained in `font-gpos-addon.json`.
- Scoped warning-denied native Clippy passed. `results/font-gpos/clippy.log` records the command output.
- Three-engine unchanged-source baseline and Rust/WASM execution are PENDING. `font_gpos_browser.rs` contains two test functions covering the constructor corpus, independent Rust/browser input equality and all actual-addon records. No browser pass is claimed yet.

Run oracle scripts with `SOURCE_PACKAGE_JSON` pointing to the installed read-only reference package. The copied source in this O: worktree supplies unchanged addon TypeScript. Browser execution also needs the existing matching wasm-bindgen0.2.100 CLI and Playwright installation. All new temporary work lives under `O:/xterm-rust-swarm-16/tmp/worktrees/Delilah447`, including its regular private target and tmp directories. There are currently no owned live processes.

## Limits

This is not full task342 closure. CFF/CFF2, other variation tables, generic malformed compressed table compatibility, the full original ligature/terminal suite and full native OS/browser quality gates remain separate work. The common WOFF inflater still uses the preexisting checked-zlib policy, unlike the source's permissive tiny-inflate behavior. Valid source-generated compressed inputs in this corpus do not prove arbitrary malformed-deflate parity. The corpus avoids unbounded allocation/nonterminating source cases; no source hang or process OOM was reproduced or presented as passing equivalence.

`results/font-gpos/` retains current native/oracle logs and scoped receipts. Earlier 1,003/1,033-input logs are historical and not relabeled as 1,571-input evidence. No main ref has been changed by this task. Final integration must use the existing gated coordinator and must not late-add to a frozen batch.
