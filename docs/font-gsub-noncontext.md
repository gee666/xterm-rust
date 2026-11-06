# GSUB noncontext null coverage

Task 538 fixes a distinct gap after the accepted null-subtable parser work. It does not replay task 530's single-substitution format change. Base main is `0442e357`. Full functionality and equivalence remain NOT PROVEN.

## Behavior

The source OpenType parser retains a zero coverage pointer as `undefined` for GSUB types 2, 3 and 4. These lookup types are ignored during direct FontImpl tree selection, so a missing coverage table alone must not reject construction. Set parsing still runs and can fail before the optional mark-filter field or later META table.

The source FontImpl also uses those noncontext tables through unchecked referenced substitutions. When that path reaches the table, `getCoverageGlyphIndex` reads `table.format` on the missing coverage value and throws a TypeError. Rust previously parsed offset zero as an empty format-1 coverage table. That hid the missing coverage and accepted affected referenced paths. The existing generic-table `Option<Coverage>` now preserves absence instead. Referenced missing coverage now returns the existing native `FontError::Type` family with the source property name, rather than `Invalid("Substitution has no coverage table")`. Extension lookup type 7 has no outer coverage property even when its inner table has valid coverage, so its referenced error family is corrected too.

This change does not implement general null coverage/classes in types 1, 5, 6 or 8, arbitrary shaping, or the complete font metadata constructor. It adds no dependency, unsafe code or JavaScript terminal delegation.

## Deterministic reference corpus

`scripts/font-gsub-noncontext-recipes.mjs` adds 1,044 SFNT/WOFF cases. It covers types 2/3/4, direct and type-7-wrapped tables, present/absent coverage, zero/nonzero sets, selected/unselected direct lookups, scalar/range contextual references, reached/unreached sequence positions, truncated headers/sets, malformed offsets and later META failure ordering. Byte-container construction and real addon lifecycle assertions reuse the existing shared helpers. Original source and assertions are unchanged.

The unchanged source parser accepts 480 and rejects 564 cases. FontImpl accepts 408 and rejects 636. The 72 additional failures are reached missing-coverage TypeErrors, not constructor parser failures. Each generated input has SHA-256 and FNV hashes; Rust independently reconstructs and checks each input's FNV hash. Node also checks the existing browser builder against its independent container writer. Immutable replay checks the exact fixture bytes.

Source `opentype.js` SHA-256 is `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`. Base font SHA-256 is `b35dd9d2131d5d83a9b87fe9ad22c6288fa3d17688d43302c14da29812417d63`. Additional pins are in `results/font-gsub-noncontext/oracle.json`.

## Reproduction

Use the unchanged supplied `source_repo` dependencies, and set `SOURCE_PACKAGE_JSON` to its absolute `package.json` if the private worktree has only a physical source copy. New scratch, `TEMP`, `TMP` and Cargo targets must be inside an owned O: worktree. Assign `TEMP` and `TMP` explicitly to the same owned O: path, or assign `TMP` in a subsequent shell command. Obtain a current allocation before compiler/browser work.

```sh
GSUB_CORPUS=noncontext node scripts/font-gsub-lookup-oracle.mjs
cargo test --locked -p xterm-addons --test font_gsub_noncontext -j2
cargo test --locked -p xterm-addons --all-targets --all-features -j2
cargo clippy --locked -p xterm-addons --all-targets --all-features -j2 -- -D warnings
cargo clippy --locked -p xterm-font-addons --test font_gsub_noncontext_browser --target wasm32-unknown-unknown -j2 -- -D warnings
cargo check --locked -p xterm-web -p xterm-font-addons --lib --target wasm32-unknown-unknown -j2
GSUB_CORPUS=noncontext node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_noncontext_browser WASM_TEST_TIMEOUT_MS=600000 node scripts/browser-host-check.mjs
```

Use `--generate` on the first command only when deliberately regenerating the source oracle, never to hide replay drift. `WASM_BINDGEN` must point to the existing CLI version 0.2.100, and `PLAYWRIGHT_MODULE` can point to the existing source installation.

## Evidence and limits

The actual pre-fix native run fails with 72 detailed mismatches in `results/font-gsub-noncontext/native-before.log`: 24 direct noncontext-reference acceptance failures and 48 type-7-reference error-family failures. `proof-counts.json` retains these counts. Both the original reference and private physical copy match all 776 original manifest entries in `reference-integrity.json`. The fixed native all-target/all-feature addon run passes 211 tests across 51 result summaries, without failures, ignored, measured or filtered tests. Strict native addon Clippy, strict new WASM-test Clippy and dependent font/web WASM library checks pass. Initial formatting and unused-test-helper findings were fixed without suppression.

The exact aggregate `cargo fmt --all -- --check` hits Windows OS error 206 on this long physical worktree path. All five whole-package formatting checks pass individually. This is not an aggregate-format waiver or a full workspace-quality pass. Final integrated native OS/MSRV/release/docs/feature matrices remain task 304.

The initial compiler commands assigned `TEMP` to the owned O: path but used `TMP="$TEMP"` in the same export command. A shell probe confirmed that this retained the inherited `/tmp` value for `TMP`. Persistent build/scratch outputs are owned O: paths, but transient compiler TMP routing from those earlier commands is not verified. No unknown system temporary paths were deleted. Final runs must use explicit O: values for both variables.

Browser/source results and final review will be recorded before this task is accepted. Component-host addon loading/fallback/cache/refresh is not complete Terminal rendering, original-suite parity or cross-platform proof. The addon boundary constructs TypeError with the native diagnostic string. That retains Chromium wording rather than Firefox/WebKit native property-access prose; this task proves the source error family/property and addon rejection lifecycle, not engine-exact browser error messages. Existing error-format representations and broader metadata/shaping gaps remain open in task 342.
