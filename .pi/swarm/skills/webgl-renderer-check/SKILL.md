---
name: webgl-renderer-check
description: Use when changing WebGL model updates, joined-cell state, signed color cache equality or frame/atlas retry ordering.
---

# WebGL renderer checks

Read docs/webgl-renderer-update.md. This checks the typed incremental engine, not the public browser addon or arbitrary JS getter/reentry semantics. Reuse the atlas/GPU checks for changes to those consumers.

From an owned O: worktree, set SOURCE_ROOT to the unchanged supplied reference with existing esbuild dependencies. Run:

```sh
node scripts/webgl-renderer-oracle.mjs
node scripts/webgl-cell-color-oracle.mjs
cargo test -j2 -p xterm-web --test webgl_renderer --test webgl_cell_colors --test webgl_glyphs --test webgl_rectangles --test webgl --test webgl_atlas_colors --test webgl_atlas_pixels
cargo test -j2 -p xterm-web --test webgl_renderer --test webgl_cell_colors --test webgl_glyphs --test webgl_rectangles --test webgl --test webgl_atlas_colors --test webgl_atlas_pixels --all-features
cargo test -j2 -p xterm-web --test webgl_renderer --test webgl_cell_colors --release
cargo clippy -j2 -p xterm-web --lib --test webgl_renderer --test webgl_cell_colors --all-features -- -D warnings
cargo clippy -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test webgl_renderer --test webgl_cell_colors --all-features -- -D warnings
cargo fmt --all -- --check
```

Require byte-identical fixture regeneration, original source hashes, retained seeds and complete state/trace assertions. Numeric fg/bg/ext arguments must remain signed where source bitwise expressions make them signed. Do not normalize them to unsigned before cache or previous-background comparisons. Keep the unchanged-joined-cell carry and final merge probe regressions.

The same `webgl_renderer` target exports two wasm-bindgen tests on wasm32. For actual 32-bit runtime checks, use `WASM_TEST=webgl_renderer CARGO_BUILD_JOBS=2 CARGO_FEATURES=acceptance-hooks node scripts/browser-host-check.mjs`, with `WASM_BINDGEN` pointing to CLI 0.2.100 and `PLAYWRIGHT_MODULE` to the existing Playwright module when needed. This replays the full source fixture in all three browsers, including large-positive row arguments that native 64-bit tests cannot distinguish. It does not construct a public addon or GPU renderer.

Use bounded commands under the current resource owner's allocation. Keep logs/source/artifact pins in results/webgl-renderer. Keep the actual 32-bit browser run when changing row-number conversion; WASM compilation alone is not runtime evidence. Record full-workspace formatting failures rather than substituting member checks. Remove only owned disposable target/tmp/worktree contents after review and preserving commits/evidence, checking processes/reparse points first.
