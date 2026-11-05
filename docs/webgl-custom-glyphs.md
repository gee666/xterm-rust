# WebGL custom glyph drawing

Task387 ports the supplied `addon-webgl/src/customGlyphs` definitions and drawing
algorithm. It does not complete TextureAtlas packing, the live WebglRenderer,
WebglAddon binding or original terminal-level WebGL suites. Those remain task306
and full-suite327 work. Overall equivalence is NOT PROVEN.

## Rust contracts

`webgl::custom_glyphs::try_draw_custom_glyph` takes the real Canvas2D context,
UTF-16 units, `CustomGlyphDrawParams` and the existing portable `LogService`.
Parameters retain cell and character dimensions separately, offsets, font size,
DPR, optional background and the pattern variant. It returns whether the source
lookup handles the value and propagates recoverable canvas/logger failures.
The caller owns any outer canvas save/restore.

The shared `draw_custom_glyph` algorithm emits typed operations immediately to a
`GlyphCanvas`. Native tests record them; the browser executor calls Canvas2D.
This avoids allocating a command list per glyph and keeps geometry independent
of browser types. Rust implements definition selection, scaling, clipping, path
state, reflected quadratic controls, SVG arc conversion, snapping, octant blocks,
sextants, seven-segment digits, braille and vector padding. No JavaScript terminal
or glyph renderer runs in the Rust implementation.

`definitions.rs` contains 778 generated definitions, including multipart glyphs.
Its size exemption is for cohesive Unicode/path data, not unstructured application
logic. The generator reads the unchanged reference through esbuild, expands
constant path functions and translates the remaining xp/yp expressions into Rust
closures. It rejects unsupported expressions. The source SHA-256 is in the file.
The definitions retain their MIT copyright and license notice.

Patterns use the source definition's generated identity and fill style as cache
keys. Identical pixel rows do not accidentally merge distinct source arrays.
Each use resets the cached pattern's transform, including variant zero. Pattern
pixels use JavaScript-owned Uint8ClampedArray storage, not borrowed WASM memory.
The source's `#rrggbb00` alpha fallback, non-string fillStyle failure and different
negative/vector/path clip and save/restore sequences are intentional. Temporary
pattern canvases use OffscreenCanvas when available or the main realm document,
never the atlas canvas's mutable ownerDocument. The only JavaScript production
shim bridges CanvasPattern.setTransform's modern DOMMatrix argument, which the
locked web-sys version incorrectly types as SVGMatrix.

The supplied ordinary definition object also reports the twelve standard
Object.prototype property names as handled without drawing. The portable lookup
preserves those results, nonmatching UTF-16, lone surrogates and combined strings.
Arbitrary runtime mutation of Object.prototype or Function.prototype is not
established by this scoped component proof. Full live renderer/joiner/raw-host
acceptance remains open, not silently inferred from these tests.

## Verification

Evidence is in `results/custom-glyphs/checks.json`, with source and artifact hashes,
raw native gate logs, the unchanged original factory baseline and both browser
reports. Base is `aa3f7921`, before the later independent ImageAddon integration.

- The two unchanged original `CustomGlyphRasterizer.test.ts` factory assertions
  pass in Node using their original source and assertions.
- Native `custom_glyphs` has two passing test functions. One compares 2,355 source
  traces: every one of 778 glyphs at three geometries, six nonmatches, twelve
  inherited names and three large-number snapping regressions. Operations and
  ordinary numeric arguments compare exactly. Only SVG ellipse arguments allow
  relative error `1e-12` for native libm versus JavaScript transcendental results.
  Browser pixel comparisons have no tolerance.
- Five Rust/WASM test functions pass in each Chromium148.0.7778.96,
  Firefox150.0.2 and WebKit26.4, in both OffscreenCanvas and explicitly unavailable
  OffscreenCanvas runs. That is thirty test-function executions, zero failed,
  ignored or filtered. Each mode compares all 2,355 cases' exact pixels and context
  state, fifteen pattern glyphs across four variants and three alpha styles,
  definition/color cache identity, gradient errors, and uncached patterns after
  both canvases move into an auxiliary document that rejects createElement.
- Full workspace formatting, native all-target/all-feature Clippy with warnings
  denied, and scoped WASM Clippy pass. Native Windows workspace all-target/all-
  feature tests pass 2,062 test functions across 411 summaries. This is not native
  Linux/macOS execution or a full all-target WASM gate.

The first native comparison exposed source path tokens such as `.27Z`; Rust's
whole-string float parser was wrong. The retained U+1FBB2 case now verifies the
same prefix parsing as parseFloat. Tightening numeric checks also exposed the
existing serde_json default decoder rounding a captured source float to its
neighbor. The test-only `float_roundtrip` feature makes captured f64 values exact.
A `2^53` dimension case prevents replacing source Math.round arithmetic with a
similar-looking expression that changes large-number behavior. These fixes did
not change source assertions or the pixel equality requirement.

## Reproduction

Use the existing reference Node dependencies, wasm-bindgen CLI0.2.100 and the
repository browser runner. Set `REFERENCE_ROOT` and `PLAYWRIGHT_MODULE` if the
installed reference dependencies are outside the checkout. `SOURCE_ROOT` may
select the read-only supplied source copy. Set TEMP and TMP separately to owned
scratch, and CARGO_BUILD_JOBS=2. Observe the team's compiler/browser cap.

```sh
node scripts/custom-glyph-definitions.mjs
cargo fmt --all
node scripts/custom-glyph-oracle.mjs
cargo test -p xterm-web --test custom_glyphs -j2
cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_custom_glyphs -j2 -- -D warnings
WASM_TEST=wasm_custom_glyphs WASM_PRELUDE=tmp/custom-glyphs/source.js node scripts/browser-host-check.mjs
WASM_TEST=wasm_custom_glyphs WASM_PRELUDE=tmp/custom-glyphs/source-dom.js node scripts/browser-host-check.mjs
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -j2 -- -D warnings
cargo test --workspace --all-targets --all-features -j2
```

On Windows use environment assignment syntax for the current shell. A sufficiently
short owned worktree path is required for exact full cargo fmt because Windows
limits its rustfmt argument list. The evidence used `O:/xterm-rust-swarm-16/tmp/G`.
Generated browser bundles, private Cargo output and test preludes are disposable
after review. Retained hashes identify tested bytes even after those temporary
paths are removed. No new runtime package dependency or unsafe code was added.
