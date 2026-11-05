# GSUB header constructor parity

Task 460 starts from main `32bf9544` on `swarm/PelayoGsub`. It fixes constructor reads in `crates/xterm-addons/src/ligatures/font_gsub.rs`, not the entire GSUB parser. Overall equivalence remains NOT PROVEN.

## Behavior

The supplied opentype.js dependency uses `parseVersion(1)`, which computes `major + minor / 10`. GSUB versions `0.10` and `0.11` therefore follow the same paths as `1.0` and `1.1`. Other values reject with `Unsupported GSUB table version.`

A zero root ScriptList, FeatureList or LookupList pointer means an empty list. A zero script or language-system record pointer means absent content. The parser must not read the parent table as that missing child. Default language-system pointers follow the same rule.

For version 1.1, a nonzero FeatureVariations pointer requires the version 1.0 variation header, a 32-bit record count and both offsets in every record. The source retains those offsets but never follows their condition-set or substitution targets. Rust previously inspected those targets and rejected valid source inputs. Invalid variation versions now reject with `GPOS/GSUB feature variations table unknown.` before reading the count.

The change keeps the existing deferred GSUB phase after GDEF/GPOS and before fvar/avar/META. No container descriptors, callbacks, refresh hooks, dependency, unsafe code, source files or upstream assertions changed. The new static errors use the existing `FontError::Invalid` representation. Native DataView bounds errors remain `FontError::Bounds`, and missing required tables remain typed `MissingTable` errors.

## Tests and commands

Run from the owned worktree with Node 24.15.0, Rust 1.85.1 and the installed reference dependencies. Set `SOURCE_PACKAGE_JSON` to the read-only source package.json. Temporary scripts, source transpilation, compiler outputs and browser bundles stay inside this O: worktree.

```sh
node scripts/font-gsub-header-addon.mjs --generate
node scripts/font-gsub-header-addon.mjs
cargo test --locked -j2 -p xterm-addons --test font_gsub_header
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gsub_header_browser -- -D warnings
cargo check --locked -j2 --target wasm32-unknown-unknown --lib -p xterm-web -p xterm-font-addons
node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_header_browser node scripts/browser-host-check.mjs
```

Use `--generate` only after inspecting intentional new recipes. The default commands require byte-identical fixtures. Both Node source scripts use real unchanged source code. Its module-resolution adapter is the existing `ligature-addon-oracle.mjs`. Browser source bundling is isolated from Rust and never ships with the product.

The corpus has 1,496 deterministic cases, with 228 source accepts and 1,268 source rejects. It includes SFNT and WOFF, truncations, version aliases and invalid versions, null/non-null root/script/default-language/language pointers, variation versions/counts/record truncations/ignored invalid targets, earlier required-table failures and later META ordering. The independently reconstructed Rust bytes are checked against source input fingerprints, and browser-source input reconstruction is checked byte-for-byte against the independent Node builder.

`results/font-gsub-header/` retains exact source hashes and outcomes. The unchanged main parser produces 962 mismatches in `native-before.log`; restoring the fixed parser passes in `native-after.log`. The scoped native addon suite has 180 passes across 36 summaries, no failures or ignored tests. Strict native/WASM Clippy and dependent WASM library checks pass. All five package fmt checks pass. The aggregate `cargo fmt --all -- --check` still reports Windows OS206; this is a failed aggregate gate, not a pass.

The unchanged source passes all 1,496 constructors and 1,496 real addon load/cache records in Chromium, Firefox and WebKit. Actual Rust/WASM passes both test functions in every engine, covering the same corpus and addon records. `source-browser.json` and `rust-browser-results.json` retain exact engine versions, source/fixture/binary hashes and observed counts. The source runner compares protocol errors exactly; it normalizes engine-specific DataView RangeError prose and the four missing-descriptor TypeErrors only. Those TypeErrors must still name `compression` on the expected missing table. The initial unnormalized Firefox message failure is retained, and no Rust source or upstream assertion changed to correct that harness assumption.

These are constructor/addon component tests, not terminal rendering acceptance. Independent review is recorded separately before handoff.

## Remaining scope

This is not full task 342/325 closure. GSUB subtable decoding, class-definition warnings and callback mutations, null feature/lookup/subtable records, mark-filter read ordering, selected lookup/tree errors, variation application and complete shaping remain outside this scoped header change. Full original suites, all-feature workspace gates, native OS matrices and full terminal rendering are also separate requirements.

The other accepted font producers must be preserved during integration. This task touches neither `font.rs` nor `font_bytes.rs`, so 442/447/453/457 payload refresh and constructor-order hooks remain owned by their existing union. The source test recipes contain no GPOS/color/CFF/outline mutations and do not claim to verify that union.
