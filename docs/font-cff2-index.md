# CFF2 FDArray INDEX boundaries

Task533 follows accepted519. Scope is the eager FDArray INDEX after global subroutine INDEX, bounded top dictionary lexical/duplicate decisions and the missing-FDArray guard. Later per-font/private dictionaries, FDSelect, charstrings and existing browser Bounds/Type wording remain342. Full342 and overall equivalence are NOT PROVEN.

## Implementation

The existing INDEX reader now preserves numeric fractional offsets through arithmetic until each DataView read applies ToIndex. Signed offset accumulation, unusual offsetSize values, empty/reversed ranges and width0 remain source-compatible. DEFLATE range replay reuses existing bounded history. No new dependency, public API, unsafe code or browser engine delegation was added.

The dictionary returns the actual FDArray operand shape after all lexical/duplicate errors. A single operand is numeric. Empty/multi-operand arrays retain JS string concatenation during start+offset and later INDEX additions. A multi-element array has commas, so every address becomes NaN and reads index0; equal offsets require no object-range reads. Rust avoids source allocation of one empty array per object. This resource optimization is not an equivalence claim for source inputs that fail a resource bound. A nonnumeric advancing string range is rejected explicitly instead of reproducing the source's nonterminating concatenation loop. This safety divergence is not approved or called equivalent. Existing negative-blend divergence remains unchanged.

## Reference and assertions

The unchanged original source is `source_repo/node_modules/opentype.js/dist/opentype.js`, SHA256 `737212d8aae53d1996ef10b315e4369a1542ebb9b94c13c5f99793914745d484`. Attribution is in `licenses/opentype-js-MIT.txt`.

The oracle retains complete unchanged constructors for464 recipes with input SHA256/FNV32, seed0x533cff02, source/font/fixture hashes and warning lists. It runs one child at a time with128MiB heap and a3000ms deadline. A failed resource bound is an observation with exact input and stderr, never a successful constructor or equivalence result. The initial unbounded monolithic attempt exited134 at the Node heap limit. It did not log the current recipe, so its exact triggering case is UNKNOWN and not reconstructed by guesswork.

For phase-specific successful INDEX traversal, a separate in-memory copy exports existing source parseCFFIndex and parseCFFTopDict functions. Their bodies and the original file remain unchanged. The unique export anchor is checked and the original source hash is pinned. Full constructors still call the uninstrumented installed source. The worker emits whole-font INDEX phase, complete constructor and table-only INDEX phase separately before returning. Resource failures may leave later phases unrun, which remains explicit in the fixture.

Native private tests compare every reported table-only source phase, including valid INDEX traversal. Complete native/WASM constructor comparisons cover inputs rejected during the whole-font INDEX phase plus original good-font controls. Later-phase and resource observations stay separate, with retained original outcomes, not passing full constructor claims. Browser source runs only safe complete constructors rejected at INDEX, plus four actual addon loads. The actual Rust addon checks async rejection, error caching, fallback ranges, recovery, reads, refreshes and disposal using the existing shared host. Source browser error bytes remain in its report. Rust's existing RangeError wording differs and is not normalized away or called equivalent.

## Reproduction

Require a team allocation. Use private O:target/TEMP/TMP and Cargo-j2. Reuse installed source dependencies, Playwright and wasm-bindgen0.2.100, with no package install. Commands are:

```sh
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-index-oracle.mjs
cargo test -j2 -p xterm-addons --all-targets --all-features
node scripts/font-cff2-index-negative.mjs
cargo clippy -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy -j2 -p xterm-font-addons --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
SOURCE_PACKAGE_JSON=/absolute/reference/package.json node scripts/font-cff2-index-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff2_index_browser WASM_TEST_TIMEOUT_MS=600000 node scripts/browser-host-check.mjs
```

## Verification status

Source464 replay passes with456 full constructor outcomes and8 full constructors unrun because their first INDEX phase hit the heap bound. Ten child heap failures include two additional table-only phase failures after a complete constructor result. Native450 table-only phase results match;10 resource observations are not equivalence claims. Complete native/WASM constructor assertions cover199 cases, with265 explicitly limited later-phase/resource observations. Source constructor outcomes include4 successes,371 Bounds errors,60 earlier missing-name errors and21 dictionary errors. Initial test selection wrongly assumed an isolated INDEX error necessarily precedes missing-name rejection in the full constructor. It was corrected to compare the unchanged complete source outcome, including that earlier error, with no source assertion edits.

Native Windows addon all-target/all-feature214 functions across50 harnesses passed with0 failed/ignored before a Clippy-only equivalent range-expression rewrite. Strict native-addon, WASM-font and native/WASM dependent checks passed after that rewrite. The new WASM target compiled. Additional directed array-shape/string-range private tests were added afterward and await their execution. Five member format checks passed; exact workspace format still fails WindowsOS206 and is not green. No full-workspace tests or native Linux/macOS/original-suite claim is made.

The FDArray-call negative executed147 Bounds mismatches:119 accepted malformed inputs and28 later hmtx errors. Exact source SHA restoration and positive replay passed. Its initial counter missed quoted JSON case names; the failure and exact restoration were retained, then the counter was fixed to require exact147/119/28, not weakened. The initial native manual-range lint failure was fixed without suppression. Browser runtime remains unrun pending allocation. Original776 and owned physical-copy776 reference hashes were checked before gates. The initial unbounded OOM is not a passing run.

## Ownership and cleanup

Only O:/xterm-rust-swarm-16/tmp/worktrees/LukeniCffIndex and its regular target/tmp/reference copy are owned temporary work. Main11ad71a8 is unchanged. Marilyn309 owns integration and resources. Accepted519 own7 were copied privately only as prerequisites, with exact exclusions in docs/intent-lukeni533.md. No505/492/495 replay. Preserve reviewed source/evidence/runtime pins before unlinking outputs, inspect links/processes and verify original hashes after normal removal. This is a plan, not a cleanup receipt.
