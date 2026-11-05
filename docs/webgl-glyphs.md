# WebGL glyph drawing

This is a GlyphRenderer component, not a working WebglAddon. TextureAtlas rasterization, packing, custom glyphs, render layers and terminal renderer replacement remain task 306. The browser demo must still report WebGL unavailable until the actual addon is implemented. Overall equivalence is NOT PROVEN.

## Mapping and contracts

The unchanged `source_repo/addons/addon-webgl/src/GlyphRenderer.ts` maps to:

- `crates/xterm-web/src/webgl/glyphs.rs` for cell vertices, clipping, rescaling, clear/resize and alternating compact buffers.
- `glyph_gpu.rs` for actual WebGL2 instanced drawing, page version uploads and owned textures.
- `glyph_shaders.rs` for the original shaders with a device-dependent sampler count capped at 32.
- `glyph_atlas.rs` for the dependency contract. The caller supplies a real atlas implementation. This file is not an implementation of TextureAtlas.
- `gpu_resources.rs` for shared RAII shader/program/buffer/VAO ownership and the unit quad also used by RectangleRenderer.

Atlas page versions must increase across page object replacement. Repacking increments the atlas layout version. `begin_frame` reports whether the complete model must be rebuilt. Assigning an atlas resets that check and invalidates every texture upload. Each changed page is uploaded from its actual canvas. Page overflow is clamped to texture capacity and warns once. Errors returned by a rasterizer keep their JS value identity.

`GlyphCell.code` is signed because the upstream combined-character bit produces negative JS int32 values before vertex updates. Null code and code zero clear the first eight glyph attributes, deliberately retaining texture height as upstream does. The shared rescaling helper excludes ASCII, emoji and powerline/nerd-font characters. GlyphCell.chars borrows UTF-16 units, preserving UCS2 lone surrogates as well as ordinary strings. Lookup uses the unit count, so a supplementary character chooses combined lookup even if it is one Unicode scalar. Rasterization receives the caller's actual terminal element, allowing one shared atlas to serve multiple terminal containers.

The caller owns a rasterization atlas and its glyph cache. The renderer retains that atlas until replacement/disposal and owns its own textures, shaders, program, buffers and VAO. Explicit disposal and Drop release those resources. Construction failures release partially allocated resources. WASM uploads copy into JS typed arrays instead of exposing unsafe views of Rust memory. No unsafe code or JS terminal engine is used.

The component queries texture capacity per renderer context, rather than using TextureAtlas's module-global static limits before that class exists. Runtime-shared atlas limit selection remains part of task 306. The current API accepts validated typed geometry. It rejects out-of-grid cells, malformed line lengths, missing glyph pages, allocation overflow and invalid viewport ranges rather than relying on JS typed-array truncation or WebGL integer conversion. Failed grid allocation leaves the previous grid usable. These are Rust boundary contracts, not proof of arbitrary mutation compatibility. The renderer reports context loss as an error. Addon context-loss events and recovery remain task 306.

## Reproduce

Use the repository Rust toolchain and installed unchanged source dependencies:

```sh
SOURCE_ROOT=/path/to/original/source_repo node scripts/webgl-glyph-oracle.mjs
cargo test --locked -j2 -p xterm-web --test webgl_glyphs
cargo clippy --locked -j2 -p xterm-web --test webgl_glyphs -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --test wasm_webgl_glyphs -- -D warnings
WASM_TEST=wasm_webgl_glyphs node scripts/browser-host-check.mjs
WASM_TEST=wasm_webgl_rectangles node scripts/browser-host-check.mjs
```

The runner supports `WASM_BINDGEN` and `PLAYWRIGHT_MODULE` paths. wasm-bindgen CLI must match locked version 0.2.100. During this run targets and scratch stay in owned O: directories, with Cargo -j2, four heavy jobs and at most two browser matrices across the team. Servers and browser processes have bounded deadlines and finally cleanup.

The source generator records SHA-256 for every bundled original dependency. Its 147 cases include 128 randomized grids with twenty updates each using xorshift seed `0x61ab9423`. Native assertions compare every backing-array f32 bit after each update, full-grid clear/resize and three repeated compact uploads. Directed cases retain null clearing, background clipping, UTF-16 lookup inputs, combined signed codes, lone UCS2 surrogates, rescaling exclusions, no atlas, differing line prefixes and same-count reshape. The one-line JSON is generated fixture data, not a hand-formatted implementation file.

Browser assertions use actual WebGL2 and readPixels with a test atlas provider backed by real canvases. Another test replays all 147 source cases against actual GPU buffer readback, begin-frame epochs and atlas lookup arguments. This proves drawing and upload behavior, not glyph rasterization. They cover multiple pages, compact line lengths, page version invalidation/replacement, lookup routing, rasterizer error identity, clipping, rescaling, resize, disposal/drop, partial constructor failures, overflow warnings and context-loss errors. Missing WebGL2 is a failed/unavailable check, never a skipped pass. Original complete WebGL/addon/demo suites remain task 306/327.

## Evidence

All three native tests pass, including 147 original-source cases with 2,588 updates. Seven glyph GPU tests pass in each of Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4 on Windows. Each engine also compares all 147 source cases against actual GPU upload bits, frame epochs and lookup arguments. The four existing rectangle GPU tests pass in each engine after shared unit-quad factoring. These are 21 glyph plus 12 rectangle test-function executions, not a full addon suite.

Focused native and WASM glyph/rectangle Clippy pass with warnings denied. Full workspace formatting passes from a real shortened worktree path, without a junction or reduced file list. `results/webgl-glyphs/verification.json` records tool versions, source/fixture hashes and scoped counts. Browser receipts retain exact commands, artifact hashes and complete per-engine outputs. No complete original WebGL suite, native Linux/macOS execution or full final workspace quality pass is claimed.

The canonical helper integrated the reviewed glyph component and the confirmed prior Oki receipt at main `4e21849c04be2ce0e542671dd335744f0c6944c1`. The default-feature changed xterm-web gate passed 323 tests across 110 summaries with zero failures/ignored tests, followed by affected/native workspace and applicable WASM library checks. Full workspace formatting passed after integration. The helper removed the private target. Exact output and scope are retained in `canonical-merge.json` and `canonical-merge.log` here. Browser matrices precede the rebase; every reviewed component/fixture source hash remained unchanged, but this is not a new full integrated browser matrix.
