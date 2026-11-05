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

Use bounded commands under the current resource owner's allocation. Keep logs/source/artifact pins in results/webgl-renderer. No browser matrix is necessary for this typed algorithm alone; WASM compilation is not browser runtime evidence. Record full-workspace formatting failures rather than substituting member checks. Remove only owned disposable target/tmp/worktree contents after review and preserving commits/evidence, checking processes/reparse points first.
