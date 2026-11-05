# Out-of-range GSUB lookup references

Task509 fixes one error contract after task497. Full font and terminal equivalence remain NOT PROVEN.

The source type6 tree builders reach substitution records only when their input position has glyphs. Both scalar and range paths then evaluate `lookups[substitution.lookupListIndex].subtables`. An array index outside the lookup list yields undefined, just as a null lookup slot does. Rust previously rejected an absent index with `FontError::Invalid("Missing GSUB substitution lookup")`, while explicit null slots already produced the source TypeError. Both paths now share that existing TypeError. Record filtering and traversal order are unchanged.

This is a deferred FontImpl error, not an OpenType parser rejection. Unselected groups, absent glyphs and unreachable input positions must not fail. A later META constructor error must still win over a tree-building error. Scalar substitution can short-circuit a later invalid record after a successful replacement; the source range routine uses a different condition and may reach that invalid record. The tests preserve that distinction.

## Regressions

`scripts/font-gsub-refs-recipes.mjs` defines240 directed cases across SFNT and WOFF. There is no randomized sample or seed. Cases cover scalar/range input, first/second/unreachable positions, valid/index-count/65535 references, selected/unselected groups, malformed later META, empty coverage and ordered duplicate-position records. The existing Node, browser and Rust container builders must agree on the complete input fingerprint. The shared binary writer exported by the null corpus avoids another copy of list-offset construction.

The unchanged source parser accepts144 and rejects96 cases, all96 for later META. Source FontImpl accepts116 and rejects124, including28 out-of-range `subtables` TypeErrors. Before the product edit, native execution reports exactly28 mismatches. It includes all1384 lookup,1038 type9 and570 null-record cases before the new240 cases. Expected fixtures were generated once from source and then replayed read-only.

Commands, from the owned worktree with installed source dependencies via `SOURCE_PACKAGE_JSON`:

```
GSUB_CORPUS=refs node scripts/font-gsub-lookup-oracle.mjs
cargo test --locked -j2 -p xterm-addons --all-targets --all-features
cargo clippy --locked -j2 -p xterm-addons --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 --target wasm32-unknown-unknown -p xterm-font-addons --test font_gsub_refs_browser --test font_gsub_null_browser -- -D warnings
cargo check --locked -j2 --target wasm32-unknown-unknown --lib -p xterm-web -p xterm-font-addons
GSUB_CORPUS=refs node scripts/font-gsub-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_gsub_refs_browser WASM_TEST_TIMEOUT_MS=360000 node scripts/browser-host-check.mjs
```

Native all-target/all-feature testing passed201 tests across47 summaries, with zero failed/ignored/measured/filtered. Strict native Clippy, both affected WASM test targets and dependent WASM libraries passed. Exact `cargo fmt --all -- --check` failed with Windows OS206. All five package checks passed. Separate package checks do not replace the aggregate failure.

The unchanged source parser, FontImpl and actual LigaturesAddon each passed all240 cases in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Exact engine-specific TypeError diagnostics are checked before canonical comparison. Rust/WASM passed both functions in each engine, zero failed/ignored/filtered. The synchronous function retains all2992 earlier corpus cases and checks the new240. The async function runs all240 cases through the actual Rust LigaturesAddon loading, fallback/cache and refresh lifecycle. These are component differential probes, not a full terminal original-suite pass.

`results/font-gsub-refs` retains exact raw commands/logs, source/browser reports, compiler/WASM/bindings/CSS hashes,25 producer input pins, native executable pin and original/physical-copy776-file reference checks. Native Windows used Rust1.85.1, Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100. Product/test/script diff checks pass. The full staged diff check reports seven raw-output whitespace findings, including PowerShell/Node CRLF and test-log final blank lines. Those logs remain byte-for-byte unchanged rather than being trimmed or hidden. The preliminary isolated review independently replayed240 source cases, the pinned native executable and25pins/776referencefiles with no product/test findings. Final browser evidence review is pending.

No new dependency, unsafe code or JavaScript terminal-engine delegation. Only one production Rust file changed and it remains below350 lines. Broader342/325 null-subtable parsing, class definitions, warnings, full shaping, unchanged original suites, native OS matrix, workspace quality and final comparisons remain open.

## Pickup and temporary ownership

Base is current main e64f4870 plus private461a19f5, an exact copy of accepted497's adc36f47. Private461a19f5 is NOT a new producer. Canonical pickup must follow original497 and take ONLY new509 commits. Main already includes original483/495. No prior canonical gate, CI or cleanup was replayed.

Only temporary root owned here is `O:/xterm-rust-swarm-16/tmp/worktrees/AntigoneGsubRefs`, including private target, source copy, browser outputs and tmp/system. Creation timed out during checkout, leaving an absent index and one partial log. With no live Git worker, recovery loaded HEAD's index, checked out missing files without overwriting existing files, restored only the proven partial own log, verified tracked-clean state and resumed the existing creator. This did not modify main or any reference. Cleanup is pending acceptance and must use a no-link/process preflight before removing only this root.
