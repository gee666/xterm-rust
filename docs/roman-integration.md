# Raw font integration on current main

Task280 integrates accepted tasks170,230 and275. Full equivalence remains NOT PROVEN. This task does not complete renderer32, arbitrary theme objects255, sparse palettes262 or public browser API31.

## Preserved changes

The retained producers are Aristides/Octavia656a9c4 and Max230. Only commits14e854e, c07d54e,4162d4b,93efcfb,c659374,ca44881 and540fd7d were carried, rather than replaying the old Root stack. Current main already supplied the two callback aliases from275, so their duplicate cherry-picked declarations were removed. Other current-main changes were preserved. The original dirty primary tree and all producer branches remain untouched.

The actual DomRenderer now reads raw font options after CSS injection, initializes its shared width cache after listener and CSS setup, and measures rows without holding an outer mutable cache borrow across host callbacks. CharSize preserves raw font values and per-expression conversion order. Portable native cache tests and real canvas tests exercise the shared implementation. The 353-line dom-font.json fixture retains four unchanged-source traces as one cohesive data file.

The width-cache source runner now accepts the same SOURCE_ROOT override used by dom-font-oracle.mjs. It resolves TypeScript from that reference's package.json, avoiding a dependency installation or junction in this worktree. No reference inputs were changed.

## Environment and commands

Observed native Windows with rustc1.85.1, Node24.15.0 and Playwright Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Browser runs here are not native Linux/macOS evidence. The worktree is swarm/Roman under C:/Users/user/.xterm-rust-target/7144d30a1fc3-worktrees/Integration/tmp/worktrees/Roman. Builds use its private target and unchanged repository profiles.

Commands run from this worktree:

```sh
cargo test --locked --offline -p xterm-web --lib --test theme_css --test theme_css_raw
cargo clippy --locked --offline -p xterm-web --lib --test theme_css --test theme_css_raw -- -D warnings
cargo clippy --locked --offline -p xterm-web --lib --test dom_font_browser --test wasm_raw_font_terminal --test row_factory_browser --target wasm32-unknown-unknown -- -D warnings
cargo test --workspace --all-targets --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
cargo fmt --all -- --check
node scripts/width-cache-oracle.mjs
node scripts/dom-font-oracle.mjs
node scripts/check-theme-css.mjs
node scripts/check-raw-theme-css.mjs
WASM_TEST=dom_font_browser node scripts/browser-host-check.mjs
WASM_TEST=row_factory_browser node scripts/browser-host-check.mjs
WASM_TEST=wasm_raw_font_terminal node scripts/browser-host-check.mjs
WASM_TEST=lib node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-wasm-support WASM_TEST=font_cache node scripts/browser-host-check.mjs
```

Source checks set SOURCE_ROOT to the preserved primary source_repo and NODE_PATH to its node_modules. Browser checks set PLAYWRIGHT_MODULE there and WASM_BINDGEN to the matching0.2.100 CLI in primary tmp/wasm-tools. Source files and assertions were not edited. Browser servers and browsers are owned and closed by the existing runner.

## Results before final integration

Logs are retained under results/integration/roman. Browser summary copies there include artifact hashes, engine versions, test names, immutable run paths and command results. Generated browser binaries remain untracked in results/browser-host and results/acceptance-artifacts.

- Scoped native checks passed59 tests. Source checks passed10 original WidthCache tests with819 assertions and6 reentrant cases, four DomRenderer font traces,73 canonical CSS cases and30 complete raw styles plus78 fast-color cases.
- Scoped native/WASM Clippy and direct rustfmt checks for every changed Rust file passed.
- The full native all-target/all-feature workspace run passed1956 tests in339 targets, zero failed or ignored.
- Full native Clippy first found the existing scrollbar_visibility.rs type-complexity lint. Rebase onto main5ae58c8 included its independently landed8c34869 repair. The same full Clippy command then passed.
- Three real-browser integration suites passed6 tests per engine, covering four caller traces,565 retained row calls from38 source cases, and an opened exported Rust Terminal with raw font updates and disposal. Four standalone actual-canvas cache tests also passed in each engine.
- The first full WASM library run failed in all engines at viewport_browser_tests.rs166 and285. A clean detached main e720ce8 baseline reproduced both failures in Chromium, excluding these font changes. Joyce confirmed missing production dependencies from the preceding viewport integration and reopened251. The original failures, error-formatting failure and timeout receipts are retained. No tests were skipped or weakened.
- Full workspace cargo fmt still fails Windows error206 from command-line length. A first direct formatting attempt also included non-Rust files and failed; the corrected Rust-only check passed. Both logs remain. This does not count as a full workspace formatting pass.

Final post-rebase browser and canonical merge results will be recorded below. Missing complete renderer/theme parity, global formatting, supported feature matrix and observed native Linux/macOS runs remain open project work.
