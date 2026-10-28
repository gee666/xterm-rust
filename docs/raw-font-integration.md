# Raw font and CSS caller integration

Task170 continues the retained Octavia and Max branches. This is a scoped font/CSS result, not complete renderer or terminal equivalence. Overall equivalence remains NOT PROVEN.

## Implementation

Branch `swarm/Octavia` stacks Xavier's `b956eeb` browser producer and accepted Max230 cache changes. Commit `540fd7d` wires the raw cache through the actual `DomRenderer` and shared row planner. Commit `c659374`, accepted task275, names two existing frame-callback types so WASM Clippy can pass without lint suppression.

- Renderer options no longer coerce font objects into a lossy native `Font` snapshot. DOM measurement and canvas setters retain original UTF-16 and per-expression String hints.
- Construction installs source listeners and injects CSS before creating the four width-cache canvases. A private `OnceCell` represents that source construction phase. Reentry before initialization reports a TypeError rather than a Rust borrow panic.
- Options changes update dimensions, inject live CSS, then read all four raw font arguments and configure canvas variants. A CSS setter may therefore mutate the font used by the following cache update.
- WidthCache owns callback-safe shared state. The renderer and row planner use shared references without an outer `RefCell` guard across canvas callbacks. Character-size changes clear that same cache.
- CharSizeService reuses `raw_font::text_metrics_font` and `font_size`, preserving size-before-family reads and raw family values passed to DOM setters.
- The portable row planner is available on native targets as intended. Its previous WASM-only declaration left row-color code unused in native builds. The row fixture helper now reads URL IDs as u64, matching `ExtendedAttrs` without truncation.

## Executed checks

Observed environment: native Windows, rustc1.85.1, cargo1.85.1, Node24.15.0. Local Playwright engines were Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. These are browser runs on Windows, not native Linux/macOS evidence.

Commands run from the isolated worktree:

```sh
cargo test --locked --offline -p xterm-web --lib --test theme_css --test theme_css_raw
cargo clippy --locked --offline -p xterm-web --lib --test theme_css --test theme_css_raw -- -D warnings
cargo clippy --locked --offline -p xterm-web --lib --test dom_font_browser --test wasm_raw_font_terminal --test row_factory_browser --target wasm32-unknown-unknown -- -D warnings
node scripts/dom-font-oracle.mjs
node scripts/check-theme-css.mjs
node scripts/check-raw-theme-css.mjs
WASM_TEST=lib node scripts/browser-host-check.mjs
WASM_TEST=dom_font_browser node scripts/browser-host-check.mjs
WASM_TEST=row_factory_browser node scripts/browser-host-check.mjs
WASM_TEST=wasm_raw_font_terminal node scripts/browser-host-check.mjs
```

For shared reference dependencies, the source trace used SOURCE_ROOT pointing to the preserved primary `source_repo`. The CSS bundler used NODE_PATH pointing to that reference's `node_modules`. Browser runs used PLAYWRIGHT_MODULE there and the existing matching wasm-bindgen0.2.100 CLI through WASM_BINDGEN. No dependency installation or source edits were required.

Results:

| Check | Observed result |
| --- | --- |
| Native library tests | 54 passed, no warnings after portable-row fix |
| Native CSS tests | 2 passed, including all73 canonical source styles |
| Native raw CSS tests | 3 passed, covering30 complete styles, live mutation/hint trace and78 fast-color cases |
| Native and WASM scoped Clippy | Passed with `-D warnings` |
| Source DomRenderer font callers | Four directed unchanged-source cases passed, imported source hashes retained |
| Source canonical CSS oracle | 73 cases passed, seed0xc55c0101 |
| Source raw CSS oracle | 30 styles, live callback trace and78 fast-color cases passed |
| Browser library suite | 13 tests passed in each engine, including CharSize and four actual cache/canvas cases |
| Browser DomRenderer font trace replay | Four tests passed in each engine |
| Browser RowFactory replay | One test passed in each engine, replaying565 captured calls from38 original cases |
| Opened public Rust Terminal raw fonts | One test passed in each engine, live option/CSS/canvas code units and DOM teardown |

`dom-font-oracle.mjs` transpiles unchanged reference TypeScript in memory. Only its measurement canvases, clock and unused row-factory constructor are controlled doubles. The Rust browser replay uses real DOM elements and real canvases. Neither is a whole-terminal pixel comparison. All four font-caller source traces are compared to the retained fixture rather than generated from Rust expectations.

Raw native, source and quality logs are under `results/dom-font/`. Immutable browser runs and latest summaries are under `results/browser-host/{lib,dom_font_browser,row_factory_browser,wasm_raw_font_terminal}`. Failed compile/test attempts are retained there. The first browser test failure came from a test helper clearing an Array.from copy instead of the live event array; the helper was corrected, not the production assertions.

## Remaining gates

`cargo fmt --all -- --check` hits Windows error206 because of command-line length. `cargo fmt -p xterm-web -- --check` runs but reports pre-existing differences in unrelated link/timer/browser-test files. Every Rust file changed by170 was formatted and checked directly with rustfmt and `skip_children=true`. Full workspace formatting is not claimed.

No full workspace, release, feature matrix, native Linux/macOS or remote CI pass is claimed here. Main integration remains subject to the canonical merge gates and current bootstrap fixes. Task32 still owns complete renderer, theme, decorations, overview ruler and browser pixel coverage. Tasks255 and262 retain boxed/stringlike theme color inputs and sparse ANSI reentry. In particular, typed theme values do not yet establish equivalence for arbitrary JavaScript objects. Task31 still owns the remaining full public browser API. None of those gaps is waived by these passing font tests.
