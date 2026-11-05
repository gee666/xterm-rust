# GSUB null feature and lookup records

Task497 fixes one remaining part of342 after accepted483. It does not establish full font or terminal equivalence.

## Behavior

The supplied opentype.js `parseFeatureList` and `parseLookupList` use `parsePointer`. A zero feature offset yields an undefined feature. A zero lookup offset yields an undefined lookup at that list index. Neither reads bytes at offset zero.

`font_gsub.rs` now preserves these values. `Gsub.selected` is absent when any calt feature is undefined. Non-calt undefined features do not affect selection. Every feature is still parsed, and all later constructor tables still run. Only `Font::from_gsub` reports the canonical `lookupListIndexes` TypeError, matching source `FontImpl` in `source_repo/addons/addon-ligatures/src/fontLigatures/index.ts`. Thus a malformed later lookup, feature variation or META table must win over that selection error.

`Gsub.lookups` stores optional entries without renumbering. An unselected undefined lookup is harmless. A selected one reports `lookupType`. A type6 rule referencing an undefined lookup reports `subtables` in both scalar and range substitution paths. Out-of-range lookup handling and general null subtables remain outside this change.

Lookup pointer reads now interleave with pointed-to table parsing. An invalid first lookup is rejected before a missing second pointer. This preserves the shared source parser's order rather than eagerly copying all pointer words.

No new dependency, unsafe code, source assertion changes or JavaScript engine delegation was introduced. All five product files remain at most350 lines.

## Regressions and commands

`scripts/font-gsub-null-recipes.mjs` defines570 deterministic directed SFNT/WOFF cases. There is no randomized sample or seed in this scoped corpus. Cases cover null calt/non-calt features, multiple features, duplicate/out-of-range selected indices, null lookup slots, malformed later lookups, truncated pointer lists, later META failures and referenced-null scalar/range paths. Existing independent Node, browser and Rust byte builders must agree on every input fingerprint.

The shared oracle defaults are unchanged. Use an installed source dependency tree with `SOURCE_PACKAGE_JSON`. All temporary paths in these runs were inside owned `O:/xterm-rust-swarm-16/tmp/worktrees/WillieGsubNull`; TEMP/TMP pointed to its `tmp/system` directory for compiler/browser runs.

```
GSUB_CORPUS=null node scripts/font-gsub-lookup-oracle.mjs
GSUB_CORPUS=null node scripts/font-gsub-header-browser-source.mjs
cargo test --locked -j2 -p xterm-addons --test font_gsub_null
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 --target wasm32-unknown-unknown -p xterm-font-addons --test font_gsub_null_browser --test font_gsub_type9_browser --test font_gsub_lookup_browser -- -D warnings
cargo check --locked -j2 --target wasm32-unknown-unknown --lib -p xterm-web -p xterm-font-addons
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_null_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs
```

Only the first intentional new-fixture creation used `--generate`. Later null, type9 and lookup source replays compared immutable expected bytes. Both retained browser targets were also rerun with the same command and timeout. The360-second bound preserves the prior483 setting for the combined retained and new assertions, not a reduced workload.

The Node baseline observes168 parser accepts and402 rejects. FontImpl observes98 accepts and472 rejects. The70 additional TypeErrors distinguish deferred null feature, selected null lookup and referenced null lookup effects. The unchanged source parser, FontImpl and actual LigaturesAddon each passed all570 cases in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Exact engine-specific TypeError messages are asserted separately from canonical Node expectations.

The new Rust/WASM target passed both functions in each engine. Its synchronous function also replays all1384 retained lookup and1038 retained type9 cases. The retained type9 and lookup targets each passed both functions in all three engines after the shared product changes. Every browser result has zero failures, ignored or filtered tests.

Native all-target/all-feature addons testing passed192 tests across44 summaries, zero failures or ignored tests. Strict native Clippy, all three affected WASM test targets and dependent WASM libraries passed. `cargo fmt --all -- --check` failed with Windows OS206. All five separate package formatting checks passed; those do not replace the failed aggregate command.

`results/font-gsub-null/negative-control.mjs` temporarily restores the exact five pre497 product files from private prerequisite58921f10. The new native test then fails with272 mismatches. Its finally restores all five original source hashes, and the same test passes. Both the initial old-product failure and this explicit restoring control are retained. Use this control only in an owned worktree with a compiler allocation.

## Evidence and integration

`results/font-gsub-null` retains raw commands/logs, source report, all three Rust/browser reports and per-engine logs, native executable pin,20 producer source pins, exact negative-control restore hashes and both776-file reference integrity checks. Rust1.85.1, Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 were used on native Windows.

The producer starts from main735e114a with ONLY original483 commits privately copied as8525bed5 through00e5635d. Private accepted495 helper58921f10 fixes owned HTTP cleanup. Neither private chain is a new producer. Current main5088f8fd already integrates original483; canonical integration belongs309. Apply only the new497 commits after original483 and original495, preserving the other main fixes. Never replay private473copy86996eb4.

Broader342/325 null-subtable decoding, class warnings/callbacks, full shaping, unchanged original suites, stable full-workspace quality, native Linux/macOS, final mapping and comparison requirements remain open. Overall equivalence is NOT PROVEN.
