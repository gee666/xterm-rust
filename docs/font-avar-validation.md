# Deferred avar constructor validation

Overall font equivalence is NOT PROVEN. Task434 validates the missing avar constructor phase in the Rust ligature loader. Task342 retains other complex metadata and variable-font behavior, including coordinate selection and shaping. CPAL remains unmodified. Its eager warnings need a separate loader ownership change.

## Read and callback contract

The source selects the last avar descriptor during directory traversal without loading earlier duplicates. It parses avar after fvar and before META. The fvar validator now returns its already parsed axis count, so an avar callback cannot retroactively change that count.

A missing fvar produces a synchronous warning before avar loading. Parsing then reads major and minor versions. A major version other than1 produces another warning before the axis count read. Reserved bytes are skipped. A missing fvar fails when the parser accesses its axes, after reading the avar count. A mismatched count fails with the source's exact argument-error text. Segment map counts determine successive F2Dot14 coordinate reads. The source does not validate map ordering, coordinate ranges, minor versions or reserved values.

Warning failures abort before later reads. The generic native callback error and browser thrown value remain unchanged. Successful callbacks can refresh the original font bytes. Descriptor selection and previously parsed fvar axes stay fixed. A borrowed avar table sees payload mutations after a version warning without rereading the version. A compressed table keeps its independent inflated bytes. Later META loading uses the refreshed font buffer.

Native FontError::Bounds represents source DataView and typed-array bounds errors. Its display text does not claim identical browser RangeError messages. FontError::Type preserves the missing-axes message, and FontError::Invalid preserves the mismatched-count message. FontWarning::message returns Cow to represent dynamic version text without allocating fixed diagnostics.

## Source and test evidence

The source oracle executes the unchanged installed opentype.js constructor. Its dependency SHA256 and Ubuntu Mono input SHA256 remain in the fixture and results/font-avar/oracle.json. The supplied dependency has no invented upstream revision.

The initial473 directed cases cover truncated headers, versions, axis counts, segment-map extents, SFNT/WOFF data, zero nominal directory length, duplicate descriptors, missing dependencies, earlier fvar/name/hmtx failures, META precedence, warning aborts and callback mutations. The expanded477 cases add missing-fvar warning mutations before avar reads. The source reports112 accepted,355 rejected and10 warning aborts. Object, number, null and undefined throws are classified by exact thrown-value identity.

The Rust assertion helper reuses the common independent metadata recipe decoder, including task428's backward-compatible extraTables and shared WOFF payload extensions. Source-compressed bytes avoid repeatedly recompressing each original font table inside the WASM test. Every reconstructed input matches the source's FNV32 hash. All477 original source input SHA256 values, outcomes and warning arrays stayed identical through this test-only optimization, as recorded in shared-input-control.json. No source assertion is changed. Native callbacks exercise both no-refresh and refreshed-buffer paths. The actual-addon scenario uses the Rust LigaturesAddon in the browser, not a JavaScript font parser. It checks failed-load caching, good-family recovery, missing-fvar failure, unsupported-version continuation, thrown-error retention, payload mutation and descriptor-only mutation. The source fixture independently executes copied upstream addon production code.

Initial473 native comparisons pass, as do retained META/fvar/mutable-buffer cases in seven test functions and seven ligature unit tests. Skipping the avar phase causes390 mismatched rows in the initial negative-control run. Production validation was restored and the473-case comparison passed again. Native all-target/all-feature xterm-addons Clippy and the focused WASM Clippy pass with warnings denied. The actual WASM binary builds. Final477 native outcomes, exact input hashes and warnings pass on rebased main dcde3e9c. Retained metadata, fvar, SVG and mutation suites pass in eight test functions. The final negative control skips avar and fails394 rows; source validation was restored before the passing builds. Both browser test functions pass in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4, with no failed, ignored or filtered tests. Each engine compares the477 constructor inputs and ten addon result fields. browser.json retains artifact hashes; input-pins.json binds fifteen production/test/oracle inputs.

The first matrix passed Chromium/WebKit but hit the60-second Firefox watchdog while the Rust test repeatedly compressed unchanged input font tables. browser-timeout.json preserves that failure. Reusing exact source-compressed input bytes reduced native corpus time from6.83 to1.07seconds. The final three-engine matrix passes with the same60-second watchdog. No timeout was increased, expected result changed or assertion removed. These timings explain a test-input bottleneck, not an application speedup.

Exact workspace formatting reports Windows OS206. All five workspace members pass separate cargo fmt package checks. This is not an aggregate formatting pass. Final native all-target/all-feature addon Clippy and focused WASM Clippy remain warning-free. Full workspace testing and native Linux/macOS results are not claimed.

## Independent review

Task434 was accepted at f4d812ee after isolated review against dcde3e9c. The reviewer inspected the unchanged source and Rust read/callback phases, reran both source oracles, executed eight retained constructor test functions and seven ligature unit tests, and verified all fifteen input pins plus four retained browser artifact hashes. No new Cargo or browser builds ran during review under the resource restriction. The accepted scope does not include full workspace gates, other native platforms, other task342 tables or variable-font shaping.

## Reproduction

Use a private O: target and Cargo jobs2. Request the resource coordinator's compiler/browser grant before heavy commands. Node24.15.0, Rust/Cargo1.85.1, wasm-bindgen0.2.100 and the existing reference Playwright installation were used.

```sh
node scripts/font-avar-oracle.mjs --source=PATH_TO_REFERENCE
SOURCE_PACKAGE_JSON=PATH_TO_REFERENCE/package.json node scripts/ligature-addon-oracle.mjs --avar
cargo test --locked -j2 -p xterm-addons --test font_avar_constructor
cargo test --locked -j2 -p xterm-addons --test font_fvar_constructor --test font_metadata_constructor --test font_buffer_mutations
cargo test --locked -j2 -p xterm-addons --lib ligatures
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_avar_browser -- -D warnings
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_avar_browser node scripts/browser-host-check.mjs
cargo fmt --all -- --check
```

PowerShell users set environment variables with $env: rather than POSIX syntax. Original full suites, live Terminal rendering, native Linux/macOS execution and the final workspace quality matrix remain separate requirements.

## Ownership and cleanup

All new temporary files and builds belong to O:/xterm-rust-swarm-16/tmp/Cerdic433. That includes target, tmp and generated browser distributions. Reference dependencies and wasm-bindgen are reused read-only from the P: checkout and are not owned cleanup paths. Retain source, fixtures and concise receipts in Git before removing the private worktree. Do not traverse reference links, modify AGENTS.md or remove another owner's paths.
