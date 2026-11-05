# Overview ruler canvas

The Rust browser Terminal now creates `OverviewRulerRenderer` when `scrollbar.showScrollbar` is not false-like and `scrollbar.width` is truthy. A later scrollbar update can create it lazily. Hiding the scrollbar keeps the existing canvas and sets its bitmap width to zero, matching `CoreBrowserTerminal.ts`. Terminal disposal removes the canvas, unregisters listeners and cancels a pending animation frame.

`renderer/overview_ruler.rs` owns subscriptions and scheduling. `overview_ruler_draw.rs` draws the outline and pooled decoration zones. `terminal/overview_services.rs` connects these to the same buffer, decorations, theme, dimensions and browser services used by the main renderer. No JavaScript terminal engine runs on the Rust path.

Recovered source and fixtures come from retained producer commit `49e6c92` on `swarm/Cornelia`. The source's module-global drawing constants, lane order and unusual bottom-border rectangle height are intentional compatibility choices. The old unused anchor flag is omitted because upstream never reads it to change drawing. Existing `ColorZoneStore` supplies pooling and merging rather than a duplicate implementation.

## Verification

Run from the repository root with the supplied reference built as documented for other source oracles:

```sh
SOURCE_ROOT=/path/to/source_repo node tools/overview_ruler_oracle.mjs
WASM_TEST=overview_ruler_browser node scripts/browser-host-check.mjs
WASM_TEST=overview_terminal_browser node scripts/browser-host-check.mjs
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test overview_ruler_browser --test overview_terminal_browser -j2 -- -D warnings
cargo test -p xterm-web --lib -j2
```

`SOURCE_ROOT` defaults to `source_repo`. The browser runner also accepts `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` paths for reusing installed tools. It closes its own browsers and HTTP server in `finally` blocks.

Observed on Windows with Node v24 and locked wasm-bindgen 0.2.100:

- Four unchanged-source drawing cases matched the retained fixture.
- `overview_ruler_browser`: one Rust test per Chromium, Firefox and WebKit passed. Each replays all four cases and asserts real canvas calls, pixels, dimensions, stylesheet and pending-job cleanup.
- `overview_terminal_browser`: two Rust tests per engine passed. They check lazy creation, same-canvas retention, hide/show, resize, theme color pixels, shared decoration registration/removal, alternate-buffer visibility and no post-disposal drawing.
- Warning-denied focused WASM Clippy passed. Native xterm-web library tests passed 54/54. Owned-file rustfmt checks passed.

Raw scoped results and unchanged-source hashes are in `results/renderer/overview-ruler-2026-11-05.json`. Full workspace, other platform and original complete-suite claims belong to their separate gates. Overall equivalence remains NOT PROVEN. This does not establish every raw JavaScript zone/coercion behavior or every renderer/addon interaction.
