# Browser font addon integration

Overall font equivalence remains NOT PROVEN. This checkpoint recovers the reviewed `swarm/Fergus` font crate and wires valid public character joiners into the real Rust terminal renderer. It does not replace the terminal with JavaScript.

## Implementation

`xterm-font-addons` contains the existing Rust browser font decisions, discovery, parsing, shaping and addon lifecycle. The crate uses existing workspace dependencies. Keeping it separate lets component tests run without constructing a terminal. `xterm-web` reexports the modules and addon constructors into its WASM distribution.

`crates/xterm-web/src/terminal/joiner_api.rs` adds `registerCharacterJoiner` and `deregisterCharacterJoiner`. They use `BrowserServices.joiners`, which the live DOM renderer already owns. Both methods reject calls before open with the source message. Registration refreshes all rows; deregistration refreshes only when a registration existed and returns JavaScript undefined. No proposed-API gate is added because upstream does not gate these methods.

The renderer invokes callbacks outside theme, width-cache and row borrows. A callback can remove another registration and add one during rendering. The existing service's live traversal observes that mutation.

## Observed checks on Windows

Commands ran in the isolated O: Clifford16 worktree with `CARGO_BUILD_JOBS=2`, Rust/Cargo 1.85.1, Node 24.15.0, wasm-bindgen CLI 0.2.100 and the existing Playwright install. Browser versions were Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4.

```sh
WASM_PACKAGE=xterm-font-addons WASM_TEST=ligatures_browser node scripts/browser-host-check.mjs
WASM_TEST=wasm_font_terminal node scripts/browser-host-check.mjs
cargo clippy -j2 -p xterm-font-addons --all-targets --target wasm32-unknown-unknown -- -D warnings
cargo clippy -j2 -p xterm-web --lib --test wasm_font_terminal --target wasm32-unknown-unknown -- -D warnings
node scripts/e2e-upstream.mjs --suite addon-web-fonts --timeout-ms 120000
cargo fmt -p xterm-font-addons -- --check
```

- Recovered ligature component assertions passed, 4 tests per browser.
- Actual terminal DOM checks passed, 3 tests per browser. They cover pre-open errors, valid joiner span changes, deregistration, default fallback ligatures, CSS font-feature settings, addon disposal and callback-driven registration mutation. They use real Rust terminal writes and render events.
- The unchanged original `WebFontsAddon.test.ts` suite passed all 15 expanded browser cases. Discovery, execution and pass counts are 15, failures/skips/unrun are zero. These assertions cover actual FontFace loads, CSS family quoting, hashing and automatic relayout on a terminal opened after addon activation.
- Both focused WASM Clippy commands and font-crate formatting passed.
- The debug production demo WASM distribution built and includes both addon constructors.

`results/font-terminal/` retains exact command receipts, original-suite JSON/logs, source hashes, browser summaries and focused lint logs. These are scoped Windows/browser results, not a full workspace or native three-platform pass.

Early new-test expectations incorrectly assumed unjoined ASCII cells always occupy separate spans and that browsers retain the literal CSS `on` token. The DOM renderer groups equal adjacent cells; CSS canonicalizes enabled features. The new tests now inspect the grouped row and compare feature CSS against an independently assigned CSSStyleDeclaration. No original assertion changed. The rejected expectations and initial failures remain in the retained failure receipts.

## Remaining work

Task 307 tracks raw/malformed joiner-return compatibility, unused optional font metadata rejection, the unchanged 8-case legacy Node `LigaturesAddon.test.ts` suite and actual loaded-font shaping on a terminal. The current joiner adapter accepts arrays of finite nonnegative integer UTF16 offsets within the WASM index range. It reports invalid returns as handler errors rather than reproducing every JavaScript coercion or range-array identity mutation. This is a known compatibility gap, not evidence of equivalence.

The empty fallback source hang remains the explicit Rust safety divergence documented in `font-addons.md`. Native Linux/macOS results and the final workspace all-feature/release/format matrix remain separate requirements. `cargo fmt --all -- --check` hit Windows OS206, including a trial through an unused temporary R: drive mapping. The mapping was removed in finally; the scoped formatting pass does not satisfy that full gate. `results/font-terminal/short-path-fmt.json` records the failed hypothesis.

## Canonical integration

The existing canonical helper integrated the reviewed producer through main `6644767`, after base `1c2bdbf`. All five workspace crates built and ran all-target tests. There were 354 native test summaries containing 1,962 passing tests, zero failures. Affected all-target checks, workspace checks and applicable WASM library checks passed. The helper removed the private target. Exact default-feature gate output is in `results/font-terminal/canonical-merge.log` and its machine-readable summary. This is not the full all-feature/release/native-platform matrix.

## Temporary resources

The owned worktree is `O:/xterm-rust-swarm-16/tmp/worktrees/Clifford16`. All new targets, browser outputs and scratch lived there. A temporary `source_repo/node_modules` junction reused the existing read-only dependency install. Unlink that junction before removing the worktree; never recurse through it. Browser runners closed their own servers and browsers. Final cleanup must preserve this branch and durable results before removing the owned worktree.
