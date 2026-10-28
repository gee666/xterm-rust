# Ligature browser component verification

Task274 extends the accepted font components, not the complete Terminal. Full equivalence remains NOT PROVEN. Work is on swarm/Fergus, continuing66d157b.

## Tests and source comparison

`crates/xterm-font-addons/tests/ligatures_browser.rs` runs4 Rust/WASM tests in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 on Windows. All4 pass per browser with0 failed/ignored. Shared operation inputs live in `tests/support/ligature_scenarios.mjs`. Its terminal is a mock boundary; shaping/matching and lifecycle under test execute Rust/WASM. The separate native oracle executes copied upstream TypeScript transpiled without assertion changes. Only module/type-import resolution is adapted. No source engine runs on Rust test pages.

The source oracle compares38 operation fields, not38 tests. It covers registration/open errors, default/custom/disabled fallback lists, stable in-place sorting and later caller array edits, UTF16 ranges, style errors, deregistration failure/retry, repeated activation/disposal, synchronous getter reentry, loading/failed fallback, current and stale loads, glyph-position ranges after supplementary text, cache clone isolation, no-refresh missing/generic/malformed families, and refresh after disposal. The source deliberately does not cancel an in-flight font load on addon disposal. The test retains that behavior.

The eight cases in `source_repo/addons/addon-ligatures/test/LigaturesAddon.test.ts` are represented by registration assertions and empty/loading/failed/missing/generic/empty/malformed-family trace checks. Its old `font-finder`/out-esbuild harness is not run here. Do not count these component comparisons as execution of that unchanged original suite. Source font parser and GSUB original assertion evidence remains separate in docs/font-addons.md.

The source oracle initially imported OpenType through incompatible CommonJS-to-ESM namespace interop. The adapter now uses createRequire and explicit dependency exports; guards require actual native ranges[[1,3]] and refresh[0,4] before saving fixtures. Expected values were regenerated only after correcting that adapter. Separately, a real recursion mismatch was reproduced: source returned nested/outer ranges, Rust threw `closure invoked recursively or after being dropped`. Changing the joiner from FnMut to Fn permits source reentry. No state borrow spans the getter or fallback coercion.

## Commands and retained evidence

Run in the Fergus worktree with Rust/Cargo1.85.1 and Node24.15.0 on PATH. Set SOURCE_PACKAGE_JSON when reusing an existing installation instead of installing dependencies into the reference copy.

```sh
SOURCE_PACKAGE_JSON=P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo/package.json node scripts/ligature-addon-oracle.mjs
cargo fmt -p xterm-font-addons -- --check
cargo clippy -p xterm-font-addons --target wasm32-unknown-unknown --all-targets -- -D warnings
cargo test -p xterm-font-addons --all-targets --all-features
WASM_PACKAGE=xterm-font-addons WASM_TEST=ligatures_browser node scripts/browser-host-check.mjs
```

The browser runner also needs WASM_BINDGEN0.2.100 and PLAYWRIGHT_MODULE, as documented in docs/font-addons.md. It closes browsers and its ephemeral server in finally blocks. No sleeps or retry loops are used by the scenarios; controlled promises observe the source family-read completion checkpoint.

- `results/font-addons/ligature-addon-oracle.json`: source/font/fixture hashes and field count.
- `tests/fixtures/ligature-addon.json`: source expected operation outputs.
- `results/browser-host/ligatures_browser-results.json`: current4-test-per-engine result, commands and tool versions.
- `results/browser-host/ligatures_browser/`: immutable raw runs, including early harness failures and the recursion counterexample. These are retained locally; the passing run is tracked with the summary.

Native tests in the browser-only crate execute0 tests and are only a build gate. This is not native Linux/macOS evidence. Real Rust Terminal joins/rendered ligatures and WebFontsAddon renderer relayout remain task40. Malformed optional font-table constructor rejection is task235. Empty matching fallback strings retain the documented safety deviation in docs/font-addons.md.
