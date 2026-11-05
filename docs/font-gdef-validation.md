# Deferred GDEF constructor validation

Task428 closes the GDEF constructor path of task342. Full font and full port equivalence remain NOT PROVEN. GPOS, CFF/CFF2, complete outline reads, color and other variation paths are not established by this work.

## Contract

The Rust reader selects the final GDEF directory descriptor without loading earlier duplicates. It validates that descriptor after kern and before GSUB, retaining the source constructor order. Accepted fvar and META validation retain their later phases. Deferred reads use the original copied directory descriptor and current font bytes, including writes made during a warning callback.

GDEF parsing reads the version, class definitions, attachment lists and coverage, ligature caret lists and mark glyph sets. Zero pointers skip children. The source accepts versions computed as `major + minor / 10` equal to1,1.2 or1.3, including unusual major0 encodings. It ignores the version1.3 variation-store offset and format3 caret device offset. Its mark-set parser consumes16-bit pointers, not the specification's32-bit records. Rust follows those actual reads rather than adding stricter OpenType validation.

Invalid ClassDef formats warn synchronously and continue without reading more of that ClassDef. `FontWarning::InvalidClassDefinition` records the source DataView offset and format. The message method now returns `Cow<str>` so the existing constant kern message stays borrowed while dynamic messages retain their exact offsets. The browser addon uses its existing live console receiver/getter/error path. Callback failure aborts immediately and preserves the thrown JS value. Subsequent uncompressed reads refresh from the original buffer alias. Already inflated GDEF bytes remain independent, but later tables see the refreshed original font.

Malformed data uses typed native errors. GDEF and CaretValue version errors retain the source messages. `FontError::CoverageFormat` retains the source absolute DataView address and message. Bounds and compression errors use the existing native representations. No dependency or unsafe code was added. The implementation does not expose unused GDEF metadata as a new public API and does not delegate parsing to JavaScript.

## Inputs and scope

`scripts/font-gdef-oracle.mjs` executes the installed unchanged opentype.js parser. Its SHA256 is `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`. The deterministic corpus has1329 cases:245 accepted,1076 malformed-data rejections and8 warning-callback failures. Seed `0x6DEF428` supplies256 malformed cases. Directed cases cover truncation, pointer/count/format boundaries, duplicate descriptors, ignored lengths/fields, SFNT/WOFF compression, phase precedence and warning mutations.

The source and Rust rebuild descriptors independently. Compressed WOFF payloads are shared fixture inputs so compressor choices cannot shift absolute warning addresses. Each reconstructed input must match its retained FNV32 byte check; source generation also retains SHA256 for every complete input. These input checks do not replace outcome and warning comparisons. The common metadata builder's optional extra tables and encoded payload support leave all existing corpus descriptors unchanged.

The second source runner reuses the existing reviewed addon transpilation adapter and runs its baseline guards before nine GDEF loading scenarios. `font_gdef_browser` runs the real Rust font reader and addon through a boundary terminal host. It covers two warnings, live alias mutation skipping the second pointer, inflated table independence, rejected versions, warnings invalidating later fvar and META, console getter and call Symbol throw identity retained for later debug logging, reentry during loading, rejection caching, fallback ranges, later successful loading and refresh. This is component evidence, not a complete Terminal renderer, original browser suite or demo pass.

## Reproduction

Use a private O: worktree and target, Cargo jobs2, locked dependencies and the matching wasm-bindgen0.2.100 CLI. Set TEMP, TMP and TMPDIR separately to the owned worktree's tmp directory. Reuse installed source dependencies without editing source or assertions. Obtain current compiler/browser resource grants before heavy commands.

```sh
node scripts/font-gdef-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/font-gdef-addon-oracle.mjs --source=PATH_TO_REFERENCE
cargo test --locked -j2 -p xterm-addons --test font_gdef_constructor
cargo test --locked -j2 -p xterm-addons --test font_metadata_constructor --test font_buffer_mutations --test font_svg_constructor
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gdef_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gdef_browser WASM_TEST_TIMEOUT_MS=300000 node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

The source addon runner writes only owned `tmp/ligature-addon-oracle` scratch and durable results. Browser runners close their servers and child browsers in finally. New disposable paths are owned under `O:/xterm-rust-swarm-16/tmp/M427`; cleanup must preserve source, fixtures and concise results first and inspect reparse points before removal.

## Evidence

Source corpus and nine source addon scenarios pass. The first source reentry probe resolved its completion signal inside the warning's nested join call, before the source load continuation ran. The retained failure is a test completion bug, not a Rust result. The helper now guards nested warning reentry exactly like the existing kern warning probe, without changing the source or the asserted behavior. Native execution passes all1329 exact-input/outcome/warning cases in one Rust test function. The final eight cases cover GDEF/fvar precedence and live warning mutations after integrating419. All1321 earlier GDEF case descriptors and outputs remain unchanged. Retained META1830, fvar287, SVG183 and buffer-mutation39 cases pass in seven focused test functions; seven existing ligature unit tests also pass. Native xterm-addons all-target/all-feature Clippy and focused xterm-font-addons WASM test Clippy pass with warnings denied. The final expanded WASM test binary builds successfully.

A temporary negative control bypassing only GDEF validation produces1144 mismatches on the final1329 corpus, then the original source is restored in finally and passes again. The earlier1321-case control produced1138 mismatches. These controls are not shipped source changes. The shared WOFF payload/input checks pass. All776 reference-manifest hashes match; retained metadata/fvar/SVG fixtures are byte-identical to main785a7781. Existing buffer-mutation controls still pass.

The exact `cargo fmt --all -- --check` command fails with Windows OS206 in `fmt-final.log`. All five complete member checks pass separately. They are not relabeled as the exact-command pass. `results/font-gdef/` retains raw logs and input pins. Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 each pass both Rust/WASM functions on the final1329-case/nine-scenario artifact, with zero failures, ignored or filtered tests. Their test durations are2.13,34.44 and2.88 seconds. `browser.json` retains versions, exact outputs, runner commands and WASM/binding/source hashes. The runner's explicit300-second deadline does not change its default. All child browsers and the artifact server closed normally. Canonical task438 integrated GDEF together with the accepted avar loader and other reviewed fixes into main3aa04692. The three shared font files preserve both producers' reviewed phases and warning variants; the common decoder and browser host call are identical. The unchanged canonical helper passes2094 default-feature native test functions across438 summaries plus affected/workspace/WASM checks, then removes the private target. Exact source/receipt composition is recorded in `results/integration438/`. These are not complete all-feature, Linux/macOS or original/browser/demo matrices, which remain unproven.
