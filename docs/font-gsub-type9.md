# GSUB type9 constructor behavior

Task483 follows accepted473. The base is main26c7c42d, which already contains accepted460 as e09faa66. Task473 is copied privately as86996eb4. Canonical integration belongs309. Pick only the new483 source/receipt commits after original473, never the private prerequisite.

The shared supplied opentype.js lookup parser accepts type9. GSUB has no parser at that array index. A zero subtable pointer bypasses parseStruct, so empty lookups and any number of null pointers are valid. A nonzero pointer calls Object.keys(undefined) and throws TypeError without reading its target. The parser processes pointers sequentially, so that TypeError precedes a later missing pointer, mark-filter word or invalid META table.

Rust represents these surviving absent subtables as Lookup::NullSubtables with their count. It does not discard them. The source ligature constructor ignores a selected type9 lookup, but a type6 rule referencing a null subtable fails when it reads coverage. Both individual-glyph and range substitution paths preserve that TypeError. No new dependency, unsafe code, reference assertion or AGENTS.md change.

## Reproduction

Use the unchanged installed source dependencies through SOURCE_PACKAGE_JSON. Set PLAYWRIGHT_MODULE and WASM_BINDGEN to the existing Playwright index.mjs and wasm-bindgen0.2.100 executable. Set TEMP/TMP and CARGO_TARGET_DIR inside the owned O: worktree. Obtain current resource grants, use Cargo-j2 and run heavy commands sequentially.

```sh
GSUB_CORPUS=type9 node scripts/font-gsub-lookup-oracle.mjs
cargo test --locked -j2 -p xterm-addons --test font_gsub_type9
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gsub_type9_browser -- -D warnings
cargo check --locked -j2 --target wasm32-unknown-unknown --lib -p xterm-web -p xterm-font-addons
GSUB_CORPUS=type9 node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_type9_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs
```

The default oracle remains lookup and the default source-browser runner remains header. Use --generate only after inspecting intentional recipe changes. The independent Node Buffer writer checks exact bytes against the existing browser builder. Native/WASM builders verify each input fingerprint. Tests retain the unchanged1384-case473 corpus as a regression check.

The new corpus has1038SFNT/WOFF recipes. OpenType accepts72 and rejects966. FontImpl accepts64 and rejects974, because eight referenced-null cases fail only when constructing the source lookup tree. Actual addon loading/cache records distinguish these two failure phases. Corpus cases cover flags, truncation at every header/pointer byte, nonzero out-of-input pointers, later offsets, mark-filter reads, META precedence, selected type9 lookups and scalar/range references.

Native old product fails with358mismatches. The retained negative-control.mjs program uses private473baseline86996eb4 and requires an isolated worktree with a compiler grant. Its finally block restored all four exact product-file hashes before the passing corpus rerun. The all-target/all-feature native addon run passed188 tests across42 summaries, with zero failed or ignored. Native strict Clippy, focused WASM strict Clippy and affected WASM libraries passed. Aggregate formatting still fails WindowsOS206, all five package formatting checks pass. Exact logs and source/artifact pins are under results/font-gsub-type9.

Unchanged source passed all1038 parser,1038 FontImpl and1038 actual-addon records in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Actual Rust/WASM passed both new test functions in each engine, with zero failed, ignored or filtered tests. The retained lookup suite also passed both functions in every engine after the shared helper extraction. Final strict WASM Clippy covers both browser targets and that helper. Reports retain compiler/WASM/bindings/CSS hashes and exact raw engine diagnostics.

The bounded360s browser deadline accounts for the retained473 Firefox measurement of112.09s and the additional1038 constructor cases. The final new suite took144.43s in Firefox. It changes no assertions or workload. Initial review found that the diagnostic normalizer incorrectly checked engine-specific messages on canonical Node expectations too. Actual and expected messages are now checked separately before canonical comparison; all three engines passed those exact checks. Native errors deliberately represent DataView bounds with FontError::Bounds and source TypeErrors with FontError::Type. Browser comparisons retain exact raw engine diagnostics and assert explicit engine-specific messages before normalization.

## Remaining work

Full equivalence is NOT PROVEN. General null feature/lookup/subtable records, class warnings and callback mutation, other subtable decoders and diagnostic formats, eager offset snapshots, variation application and full terminal shaping remain342/325. Full unchanged suites, workspace quality and native Windows/Linux/macOS matrices remain required.
