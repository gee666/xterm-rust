# GSUB zero subtable pointers

Task517 repairs zero subtable pointers for GSUB lookup types1-8. Full font, addon and terminal equivalence remains NOT PROVEN.

## Behavior

The unchanged OpenType parser consumes each subtable pointer and follows it before reading the next pointer. A zero pointer yields JavaScript undefined without reading a format word. The old Rust implementation read all offsets first, then treated zero as an offset to the lookup header. This rejected fonts the source accepts and changed error precedence.

The Rust lookup variants now retain optional subtable values at their original indices. Parsing still visits all nonnull subtables, including those outside selected calt groups. Null subtable errors occur only when FontImpl actually dereferences them. Selected type6 reads substFormat, selected type8 reads coverage, and reachable substitution references read coverage. Unselected groups and unreachable records do not fail. Source scalar substitution can return before a later null slot. Source range substitution deliberately continues after all-nonnull replacement values. Later constructor META errors still precede FontImpl tree errors.

The source type9 null-subtable representation is unchanged. No JavaScript terminal or font engine executes Rust implementation logic. There are no new dependencies or unsafe blocks. The small documented font.rs line-limit exception keeps its existing ordered loader and public Font operations together.

## Corpus and commands

`scripts/font-gsub-subtables-recipes.mjs` contains 1,224 directed SFNT/WOFF cases. It covers types1-8, null-only and mixed subtable arrays, selected and unselected calt groups, scalar/range references, reachable and unreachable sequence positions, missing offsets and mark-filter words, earlier malformed pointed-to data, and later META errors. The source parser accepts508/rejects716. FontImpl accepts392/rejects832. Input SHA256 plus an independent Node container writer pin every reconstructed input. Browser and Rust byte builders must reproduce them. The corpus is directed, not randomized.

The source dependency SHA256 is `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`. Original source files and assertions remain unchanged. The UbuntuMono font fixture and upstream attribution are retained.

Run commands from the producer worktree, with new temporary outputs and Cargo target inside that owned O: root. Native toolchain is rustc1.85.1/cargo1.85.1; Node24.15.0. Browser and compiler runs require the team resource owner's grant. Cargo uses-j2. The existing wasm-bindgen0.2.100 and installed original Playwright dependencies are reused.

```sh
SOURCE_PACKAGE_JSON=P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/package.json GSUB_CORPUS=subtables node scripts/font-gsub-lookup-oracle.mjs
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_gsub_subtables_browser --test font_gsub_refs_browser -- -D warnings
cargo check --locked -j2 --target wasm32-unknown-unknown --lib -p xterm-web -p xterm-wasm-support -p xterm-font-addons
SOURCE_PACKAGE_JSON=P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/package.json GSUB_CORPUS=subtables node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_subtables_browser node scripts/browser-host-check.mjs
node results/font-gsub-subtables/verify.mjs
```

Set WASM_BINDGEN and PLAYWRIGHT_MODULE to their existing installations. The check skill documents these adapters. Generate expected fixtures only from an inspected new source corpus. Ordinary replay is read-only for fixtures.

## Observed evidence

`results/font-gsub-subtables/` retains exact raw output and command scope. `native-before.log` records an actual pre-fix run against private prerequisite417b9a2b, an exact copy of accepted509. All four production files were restored in a finally block. It found944 mismatches. The new implementation passed202 tests across48 native addon summaries, with zero failed, ignored, measured or filtered tests. These include the retained lookup1384/type91038/null570/refs240 assertions before the new1224.

Native strict all-target/all-feature addon Clippy, both affected WASM test lint targets and the affected dependent WASM libraries passed. Exact `cargo fmt --all -- --check` failed with WindowsOS206. Each of the five package formatting checks passed. Member passes do not establish the aggregate gate. Production/test/script diff-check passed. The complete staged diff-check reports CRLF trailing whitespace and final blank lines in exact raw command output. Those logs and the checker output are retained unchanged, not trimmed or suppressed to manufacture a clean result.

Unchanged-source parser1224, FontImpl1224 and actual source-addon1224 passed in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Exact engine-specific missing-property TypeError messages were checked before normalization. Both actual Rust/WASM tests passed in every engine, with zero failed, ignored or filtered tests. The synchronous test retains all earlier GSUB assertions and the new1224 cases. The asynchronous test exercises actual RustLigaturesAddon loading, fallback, caching and refresh behavior for every new case.

The initial combined browser driver exceeded its300second outer deadline after Chromium passed. Its incomplete report and logs are retained. A same-artifact Firefox diagnostic exceeded180seconds. The accepted509 evidence had Firefox144.77seconds versus Chromium11.29seconds. A justified longer420second assertion deadline then observed both new Firefox tests passing in249.86seconds. The remote BrowserServer.kill shutdown stalled after those assertions had been saved, and the480second outer deadline terminated that diagnostic. A fresh process inspection confirmed no owned process remained. Chromium and WebKit replayed the same hash-pinned artifacts and exited normally after closing the client before the server. There was no Firefox test rerun after its recorded pass, no assertion reduction, source change or recompilation to make these browser results pass. These execution and shutdown outcomes are separate fields in the final report.

`verify.mjs` checks immutable producer hashes, all776 reference files and, after browser completion, exact reports and native/WASM/binding/CSS pins. Run it before cleanup so reviewers can independently inspect generated artifacts. After cleanup, `--retained` checks committed source and proof without claiming generated binaries remain present.

## Integration and limits

Base is main1106d480 plus private509417b9a2b. Only the new517 commits belong in canonical pickup, after ORIGINAL509d2f316b3 and its347d6917 receipt. Never pick the private prerequisite again. Integration belongs task309, not this producer. Original497 is already in main1106d480.

The fresh creator initially timed out60seconds midcheckout. Inspection found no live owned creator or Git process and only a stale owned index.lock. Recovery reconstructed only the new owned1106 checkout, verified13564 tracked files and clean status, and completed the physical reference copy. No original reference or other owner's worktree was removed. All517 compiler/browser/server grants are released. Generated artifacts remain only for independent review. Cleanup has not yet run for517.

This does not close deeper metadata342, font/GPU family325, null nested coverage/class structures, full shaping behavior, complete unchanged upstream suites, stable workspace quality, actual native Windows/Linux/macOS matrices, mapping, demonstrations or matched speed/size/LoC comparisons. All remain required by the full brief.
