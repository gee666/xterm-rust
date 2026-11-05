# Custom glyph pattern host-call phases

Task392 repairs the browser boundary of task387. It does not complete TextureAtlas, WebglRenderer/WebglAddon, original WebGL suites or the complete port. Overall equivalence remains NOT PROVEN.

## Confirmed differences

The unchanged source `CustomGlyphRasterizer.ts` creates a temporary canvas and context, then calls `new ImageData(width, height)`, then parses the fill color. It rereads `imageData.data` for each channel assignment, calls `ctx.createPattern(canvas, null)` and resets the cached pattern transform. Variant zero constructs an identity DOMMatrix without calling `translateSelf`.

The initial Rust component had identical rendered pixels for the retained normal fixtures, but its host operations differed. It parsed color first, built a separate clamped array and used the three-argument ImageData overload, passed `"repeat"`, and always called `translateSelf`, including zero offsets. It also selected OffscreenCanvas availability at draw time rather than module initialization. Genuine browser constructor/method proxies expose these differences without substituting a canvas or delegating Rust drawing to JavaScript.

The initial three-function Chromium negative control reports zero passed, three failed, zero ignored. After adding factory-availability coverage, the same original task387 `patterns.rs` produces zero passed, four failed. The original file was restored only for that negative control and the fixed file restored in `finally`. Both raw failed runs are retained under `results/custom-glyph-pattern-host392/`. The tests restore every instrumented browser property before asserting. Independent review then found a further callee-lookup ordering error: the initial fix built DOMMatrix before reading `pattern.setTransform`. A genuine pattern's throwing getter must win over a separately throwing matrix constructor. The additional fifth-function negative control reports four passed and one failed in Chromium. The final direct branch expressions preserve method lookup before argument construction and keep both thrown values distinct.

## Implementation

Pixel generation and color interpretation remain Rust. A small `patterns.js` module performs only the Web API operations that the locked typed web-sys overloads cannot express with the same argument shape and source timing. Every fallible call is a catching wasm-bindgen boundary. Offscreen factory availability is captured once when this host module initializes; its constructor is still read when called. DOM fallback uses the main realm document, not an adopted atlas canvas's ownerDocument.

Rust now creates canvas/context/ImageData before color interpretation, fills the ImageData's own JavaScript-owned data buffer, rereads the data property for every channel, calls the real createPattern method with null repetition, and omits translation for zero offsets. It preserves thrown values and does not insert a pattern into the Rust cache after failed construction or data access. No cache borrow crosses a host call. The previous no-borrowed-WASM-memory ownership property is retained through the two-argument ImageData constructor.

The JavaScript file is DOM/property interop, not a glyph renderer or terminal engine. The native glyph definitions, paths, geometry, color calculations, loops and selection remain Rust. There is no new dependency, unsafe code, lint allowance or reference-source modification.

## Assertions and evidence

The new test binary uses real Canvas2D objects and proxies of actual browser constructors. It records constructor arity, numeric arguments, context lookup, ImageData data getter reads, createPattern receiver/null argument, DOMMatrix construction and translations. It compares complete traces, exact pixels and thrown Symbol identity with the unchanged source. Cases cover valid drawing, canvas and image constructor throws, unsupported color order, data getter throws and subsequent cache reuse, pattern callback throws, all four variants, and availability changes after module initialization.

Five new Rust/WASM test functions exercise Chromium, Firefox and WebKit with both normal OffscreenCanvas and forced DOM fallback, including the independent getter-order regression. All five existing task387 browser functions remain unchanged. The final two-mode matrix has 60 test-function executions, not 60 distinct source cases; exact passing receipts and hashes are recorded in the verification file. No functions were skipped or filtered. Every existing 2,355-case fixture byte remains unchanged; the two native glyph test functions and the two unchanged original factory assertions pass. The two test binaries share context/parameter/logger setup; no original assertion was removed.

The initial base passed exact formatting, full native workspace all-target/all-feature warning-denied Clippy, and scoped WASM library plus both test-binary Clippy on Rust1.85.1. After the run2 rebase and added workspace targets, the exact `cargo fmt --all -- --check` command fails with Windows OS206 even at the short physical O:I path; Rust1.97.1 and a temporary SUBST alias reproduce it. The owned alias was removed in finally. All five complete workspace-member `cargo fmt -p PACKAGE -- --check` commands pass. These are full member checks, not a claimed pass of the failed exact command; final quality task304 remains open for this tooling limitation.

The initial full native workspace all-target/all-feature run passes 2,064 functions across 415 summaries. The rebased run2 refresh passes 2,069 functions across 420 summaries, zero failed or ignored, with a bounded 95-second command. Both raw runs are retained. Fresh scoped WASM warning-denied Clippy passes after the getter-order repair. Independent review is recorded separately with the final task receipt. Source, fixture, prelude, implementation, binding and WASM hashes identify each run. Native SVG transcendental tolerance remains the separately documented task387 contract; browser pixels have no tolerance.

## Reproduction

Use the existing source dependencies and wasm-bindgen0.2.100 with the current heavy/browser limits. Set TEMP/TMP/TMPDIR separately to owned O: scratch and Cargo build jobs to2. No dependency installation is needed.

```sh
node scripts/custom-glyph-oracle.mjs
cargo fmt --all -- --check
WASM_TEST=wasm_custom_glyph_pattern_host WASM_PRELUDE=tmp/custom-glyphs/source.js node scripts/browser-host-check.mjs
WASM_TEST=wasm_custom_glyph_pattern_host WASM_PRELUDE=tmp/custom-glyphs/source-dom.js node scripts/browser-host-check.mjs
WASM_TEST=wasm_custom_glyphs WASM_PRELUDE=tmp/custom-glyphs/source.js node scripts/browser-host-check.mjs
WASM_TEST=wasm_custom_glyphs WASM_PRELUDE=tmp/custom-glyphs/source-dom.js node scripts/browser-host-check.mjs
cargo test --locked -p xterm-web --test custom_glyphs -j2
cargo clippy --locked --workspace --all-targets --all-features -j2 -- -D warnings
cargo clippy --locked -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_custom_glyphs --test wasm_custom_glyph_pattern_host -j2 -- -D warnings
cargo test --locked --workspace --all-targets --all-features -j2
```

Set SOURCE_ROOT and REFERENCE_ROOT to the unchanged supplied source when it is outside the checkout. WASM_BINDGEN and PLAYWRIGHT_MODULE may point to the existing read-only tools. These scoped runs do not establish full live renderer, global prototype mutation or arbitrary public addon parity.
