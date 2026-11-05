# Selected CFF constructor headers

Task457 implements the deferred CFF header phase omitted by the earlier loader. Full font parity remains NOT PROVEN. CFF indices, dictionaries, charstrings, CID metadata, CFF2 variation stores and glyph rendering remain342 work. A valid header is not a validated full CFF program. Other optional tables and the source inflater's cross-parse global tree cache remain unfinished.

## Contracts

The final CFF descriptor takes precedence over the final CFF2 descriptor regardless of directory order. A complete glyf/loca pair bypasses both. Name parsing precedes this phase; hmtx follows it. Directory descriptors retain original values across callbacks, while deferred payload reads use the current aliased buffer. The header reads major then minor, rejects major greater than two with the exact source message, then reads size and either one CFF1 offset-size byte or two CFF2 top-dictionary-length bytes. Major zero follows the source CFF1 path. CFF1 does not validate or use hdrSize to find its first index.

Compressed selected headers share task420's source-derived bounded syntax walker. It now retains at most five output bytes and validates the entire stream before header inspection. Short output stays zero-filled, excess output is discarded, missing stored payload is zero, and backward copies beyond earlier output produce zero. No font-sized output allocation is needed for this partial metadata phase. Hint validation still discards output and uses the same walker. The existing safe guard rejects exhausted-input streams that the source would execute forever; this is a documented safety divergence, not equivalence. No new dependency, unsafe code or JS parser delegation is present.

Native `FontError::CffVersion { major, minor }` preserves the source text and maps to browser Error. Checked native bounds and compression use the established FontError variants. Browser built-in bounds messages vary by engine and are not claimed text-identical. Custom CFF version messages are compared exactly.

## Evidence and limits

The source-hashed corpus has520 constructor cases,66 accepted,446 ordinary errors and8 warning throws, seed0x457cff01. Inputs include both tags, SFNT and WOFF, duplicates, outline selection, header read precedence, recorded lengths, compressed short/overflow/stored/fixed data, and early CPAL warning mutation or exact thrown values. A separate actual-addon trace covers four loads, cached failures, successful recovery and the previous error object passed to debug. The source oracle executes unchanged opentype.js and copied addon TypeScript with only transpilation/import resolution adapted.

Native520 plus unchanged hint792, outline612 and color676 corpora pass four test functions. The old strict miniz output-size guard fails the retained short/overflow/missing-stored-payload cases; `negative.mjs` restores exact producer bytes in finally and the full520 corpus passes again. Strict scoped native and WASM Clippy pass. Both actual WASM test targets compile. Full workspace fmt hits Windows OS206; member checks are recorded separately. This is not a workspace, native Linux/macOS or full upstream suite pass.

Final source520 constructor cases and four actual-addon loads pass in Chromium148, Firefox150 and WebKit26.4. Current Rust passes two test functions per engine, including all520 constructors, independent JS/Rust byte equality and the same four actual-addon loads. The shared-inflater hint regression passes two functions per engine covering792 constructors, actual-addon loads and the separate safe termination guard. No test was skipped or filtered. `source-browser.json`, `rust-browser.json` and `hint-browser.json` pin their executed artifacts and retain per-engine logs. The source preflight found no defects; final acceptance review checks these new receipts. These scoped passes do not close342 or prove overall equivalence.

## Reproduction

From this worktree, set SOURCE_PACKAGE_JSON to the installed original source package.json for Node dependency reuse. New scratch and targets stay inside this owned O: worktree. Every shell command has an external timeout and Cargo uses-j2.

```text
node scripts/font-cff-header-oracle.mjs --source=P:/workshop/Windows_Worshop/oira666/xterm-rust/source_repo
node scripts/font-cff-header-addon-oracle.mjs
cargo test --locked -j2 -p xterm-addons --test font_cff_header --test font_hint_tables --test font_outline --test font_color_constructor -- --nocapture
cargo clippy --locked -j2 -p xterm-addons --lib --test font_cff_header --test font_hint_tables --test font_outline --test font_color_constructor --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-font-addons --target wasm32-unknown-unknown --test font_cff_header_browser --test font_hint_tables_browser --all-features -- -D warnings
node results/font-cff-header/negative.mjs
node scripts/font-cff-header-browser-source.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_cff_header_browser node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=font_hint_tables_browser node scripts/browser-host-check.mjs
```

Browser commands require an explicit team grant. Reuse original Playwright and wasm-bindgen0.2.100 through their documented environment overrides. No source assertions, AGENTS rules, credentials or main refs were edited. This branch privately merges accepted442 and453 plus current main32bf9544. Integration must preserve those prerequisites once, preserve447 GPOS separately, and apply the minimal457 source/fixture/evidence delta. Owned temporary root is O:/xterm-rust-swarm-16/tmp/worktrees/Abigail457 only; preserve commits, scan links and stop owned jobs before cleanup.
