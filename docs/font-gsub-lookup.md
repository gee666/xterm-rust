# GSUB lookup header reads

Task473 continues task342 after accepted460. Worktree `swarm/MasamuneGsub` starts at main `3a4f5e8f`, which already contains the accepted442/447/453/457 font union. Accepted460 producer `57bd8d3b` is a local prerequisite as `dd53c6c7`. Its only conflict was the font skill append, resolved by retaining both paragraphs. Do not replay the entire private history as a new producer. Canonical integration remains owned by task309.

## Behavior

The supplied opentype.js shared GPOS/GSUB lookup parser reads the type and rejects values outside1..9 before reading flags, subtable count or offsets. Rust previously rejected those values only after reading the rest of the header. A truncated header therefore incorrectly returned a bounds failure instead of the source lookup-type failure. GSUB now uses the existing `FontError::GposLookupType` variant, whose diagnostic is already the common `GPOS/GSUB lookup type N unknown.`. The variant's historical name is retained rather than changing an unrelated public error contract.

When the mark-filter flag is set, source reads the trailing markFilteringSet word after parsing all subtables. An earlier subtable format failure wins even when that trailing word is absent. Rust previously read it first and replaced the format failure with a bounds error. The read now follows the subtable loop. Its value is still not used for shaping, matching the existing constructor-only contract here.

Only `crates/xterm-addons/src/ligatures/font_gsub.rs` changes product behavior. No dependency, unsafe code, source assertion or AGENTS.md change. The existing source-browser runner accepts `GSUB_CORPUS=lookup` to reuse its server, byte builder and actual-addon scenarios. Default `header` behavior remains available. Independent Node Buffer reconstruction verifies byte identity against the existing browser DataView builder; native/WASM reconstruction verifies input fingerprints.

## Verification

Set `SOURCE_PACKAGE_JSON` to the unchanged installed reference package.json. Set `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to the existing Playwright module and wasm-bindgen0.2.100 executable. Use the owned O: target and separate TEMP/TMP directory, Cargo-j2 and current resource grants.

```sh
node scripts/font-gsub-lookup-oracle.mjs
cargo test --locked -j2 -p xterm-addons --test font_gsub_lookup
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gsub_lookup_browser -- -D warnings
cargo check --locked -j2 --target wasm32-unknown-unknown --lib -p xterm-web -p xterm-font-addons
GSUB_CORPUS=lookup node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_lookup_browser node scripts/browser-host-check.mjs
```

Use `--generate` only after inspecting intentional recipe changes. Default oracle execution requires byte-identical fixtures and records source hashes. The corpus has1,384 SFNT/WOFF recipes,96 source accepts and1,288 rejects. Directed truncations test invalid lookup types, flag combinations, empty lookups, malformed subtables before missing/present mark-filter words and later META ordering. All1,384 actual source-addon load/cache records match the constructor outcome. There are no source-warning callbacks in this scoped corpus.

Native uses the existing typed Rust representation for bounds and subtable-format errors. The fixture retains exact source class/message/offsets; unknown messages fail rather than normalize away a mismatch. Browser source comparisons require exact protocol diagnostics and normalize only engine-specific DataView RangeError prose. Actual-addon checks compare fallback/native ranges, refreshes, reads and cache hits.

`results/font-gsub-lookup/commands.json` retains exact scoped commands, including initial failure. Old unchanged product gives664 mismatches. A `finally` block restores exact fixed bytes before the passing native run. Initial strict Clippy found an unused shared test helper; using its existing bounds/META error mapping fixed that without suppression. The final all-target/all-feature addon suite has186 passes across41 summaries,0 failures/ignored. Strict scoped native/WASM Clippy and dependent WASM libraries pass. Aggregate fmt fails with Windows OS206; each of the five package fmt checks passes. An initial local source replay omitted SOURCE_PACKAGE_JSON and failed module resolution before running cases; the correctly configured replay passes without fixture regeneration.

Unchanged source passes all1,384 constructors and1,384 actual-addon records in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Actual Rust/WASM passes2 test functions in each engine, covering the same corpus and load/cache records with0 failures, ignored or filtered tests. `source-browser.json` and `rust-browser-results.json` retain source/fixture/compiler/WASM/bindings/CSS pins. Independent review is recorded separately. These are constructor and addon component tests, not full terminal or workspace acceptance.

## Remaining scope

Overall equivalence remains NOT PROVEN. Type9 GSUB lookup behavior, null feature/lookup/subtable records, complete subtable decoding and diagnostic formats, class-definition warnings and callback mutations, eagerly snapshotted subtable offsets, variation application and full shaping remain task342/325. Native OS matrices, full original suites, full terminal rendering and workspace quality gates remain separate requirements. Existing accepted font callbacks and constructor-order hooks are preserved, not reimplemented here.
