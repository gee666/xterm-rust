# Font addons

Full font-addon equivalence is NOT PROVEN. Native shaping task119 and metadata resolver158 passed independent review. Task40 still needs real terminal joiner integration and ligature lifecycle tests; task235 covers unused-metadata constructor rejection.

## Code

- `crates/xterm-addons/src/ligatures/` has the portable UTF16 parser, fallback matcher and native font cmap/GSUB/cache engine.
- `crates/xterm-font-addons/` contains the independent browser bindings. This avoids coupling font component tests to unfinished terminal bindings. `xterm-web` reexports the previous module and class names.
- `font_set.rs` selects, deduplicates and loads actual FontFace objects.
- `font_addons.rs` implements WebFontsAddon relayout and disposal.
- `local_fonts.rs` implements legacy/modern discovery, permission handling, blob loading and shared metadata cache. Query APIs are explicitly mocked in component tests; promises and Blob run in browsers.
- `ligatures_addon.rs`, `ligature_state.rs` and the small `ligature_host` browser adapter are task40 work in progress. They compile, but their addon-specific lifecycle and actual terminal integration are not yet tested.

## Checks

Run in the worktree with Rust1.85.1 and Node24 on PATH:

```sh
cargo test -p xterm-addons --lib ligatures::font
cargo clippy -p xterm-addons --lib -- -D warnings
node scripts/local-font-oracle.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=local_fonts_browser node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=web_fonts_browser node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=web_fonts_async_browser node scripts/browser-host-check.mjs
WASM_PACKAGE=xterm-font-addons WASM_TEST=web_fonts_ready_browser node scripts/browser-host-check.mjs
cargo clippy -p xterm-font-addons --target wasm32-unknown-unknown --lib --tests -- -D warnings
```

Set WASM_BINDGEN to an installed0.2.100 CLI and PLAYWRIGHT_MODULE to an installed Playwright index.mjs. Existing shared tools can be reused; no dependency reinstall is needed per worktree.

Observed results:

- Native font backend:7 Rust tests pass, including2,540 source font/cache calls,217 directed graphs, original merge/range cases and six WOFF comparisons. Original JS suites pass449 tests and669 assertions.
- Metadata:10 Rust/WASM tests pass in each Chromium, Firefox and WebKit. Source10-case/30-call oracle, concurrent cache case and optional-permission microtask traces pass. Independent review caught and then verified the fixed missing-await branch.
- Web fonts:8 basic,6 asynchronous and4 ready/disposal tests pass in each browser. Initial relayout now waits for the automatic setter writes rather than triggering another relayout in the test.
- A WebKit test fixture was corrected: constructor quote characters are literal there, unlike Firefox's quoted CSS family attribute. The test now explicitly supplies the quoted attribute it intends to verify; implementation assertions remain unchanged.

These are component results, not proof of full terminal/rendering integration or native Linux/macOS results. Use the existing GitHub workflow for native platform runs.

## Compatibility notes

All string/range helpers retain raw UTF16. Source shaped ranges count glyph positions, while fallback ranges count UTF16 positions; the port preserves that distinction. Cached full font results and their arrays retain shared mutable identity, while joiner results copy range pairs.

Empty fallback lists are supported. A zero-length matching fallback string can hang upstream; Rust rejects non-advancing fallback traversal instead. This safety boundary is explicit, not claimed as exact behavior for a source hang.

The metadata resolver queries before rejecting non-string family input and preserves the optional permission await boundary. The source module's realm-wide catalog must remain shared when wiring the actual addon. Font binaries and derived WOFF test data retain notices in `licenses/ligature-font-fixtures.md`.
