---
name: webgl-cell-color-check
description: Use when changing WebGL packed cell colors, decoration/selection precedence, inverse completion or dotted/pattern variant phases.
---

# WebGL cell color checks

Read `docs/webgl-cell-colors.md` for the typed component boundary and remaining
browser integration scope. Reuse the unchanged-source oracle and fixture rather
than copying the algorithm into another test implementation.

1. Inspect current main and producer worktree status. Keep source_repo read-only.
2. Use an owned O: worktree/target/tmp and the current resource owner's focused
   compiler grant. Cargo uses `-j2`. This component needs no browser grant.
3. Set SOURCE_ROOT to the unchanged installed reference. Run
   `node scripts/webgl-cell-color-oracle.mjs`. To verify existing fixtures without
   changing them, set CELL_COLOR_ORACLE_OUTPUT to an owned O: temporary path and
   compare bytes with `crates/xterm-web/tests/fixtures/webgl-cell-colors.json`.
4. Run `cargo test --locked -p xterm-web --test webgl_cell_colors --all-features -j2`
   and the same command with `--release`. The fixture contains 2,956 cases at seed
   0x7a12c431. Two Rust functions pass; the differential runs every case twice.
5. Run native and wasm32-unknown-unknown Clippy for `--lib --test webgl_cell_colors
   --all-features -- -D warnings`. Include the existing webgl, webgl_atlas_colors,
   webgl_atlas_pixels, webgl_glyphs and webgl_rectangles native targets for shared
   component regressions.
6. Preserve commands, source/fixture/oracle hashes and actual failures. Check
   exact workspace fmt separately and disclose OS206 rather than replacing its
   result with a member check. Compilation is not a browser runtime result.
7. Obtain isolated review, preserve accepted source/evidence and clean owned
   temporary files after use. Never remove reference/dependency links recursively.

The resolver accepts typed values. Tests here do not establish raw JS getter or
module-global reentry behavior, actual model consumption, public WebglAddon or
full renderer parity. Those remain task306 obligations.
