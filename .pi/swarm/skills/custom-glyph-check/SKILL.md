---
name: custom-glyph-check
description: Use when changing WebGL custom glyph definitions, path arithmetic, patterns or Canvas2D rasterization.
---

# Custom glyph checks

Read docs/webgl-custom-glyphs.md. This component does not establish complete
TextureAtlas/WebglRenderer/WebglAddon or terminal-level WebGL acceptance.

Use the unchanged reference and its existing esbuild/Mocha/Chai dependencies.
Set REFERENCE_ROOT for dependency resolution and SOURCE_ROOT for another read-only
source copy. Run scripts/custom-glyph-definitions.mjs, cargo fmt --all, then
scripts/custom-glyph-oracle.mjs. The latter retains2 unchanged factory assertions,
2355 source traces and test-only browser preludes under tmp/custom-glyphs.
Do not remove cases or weaken exact numbers/pixels to obtain a pass. The native
decoder requires serde_json's test-only float_roundtrip feature; ellipse arguments
alone allow1e-12 native transcendental error. Other numeric arguments are exact.

Run native custom_glyphs plus warning-denied native/WASM Clippy. Run the existing
browser-host-check.mjs with WASM_TEST=wasm_custom_glyphs and each WASM_PRELUDE
value tmp/custom-glyphs/source.js and tmp/custom-glyphs/source-dom.js. Both modes
must pass all5 functions in Chromium, Firefox and WebKit, including pattern cache
identity and adoption into a document that rejects createElement. Pin exact
reference/fixture/source/WASM/binding/prelude hashes in durable evidence. The
JavaScript renderer is an oracle only, never a Rust implementation dependency.

For pattern DOM/constructor boundaries, also run WASM_TEST=wasm_custom_glyph_pattern_host
with both preludes. Its real-context probes compare exact null repetition, two-argument
ImageData construction and repeated data reads, factory/color/throw order, captured
availability and zero-variant transforms. Preserve the old-pattern negative-control
receipts; equal pixels alone do not prove host-call parity.

Keep the shared compiler/browser limits and Cargo-j2. Set TEMP and TMP separately
to owned O: scratch. Run full native workspace fmt/Clippy/tests after changes;
feature unification from dev dependencies can affect other fixture decoders.
Use a short physical owned path for exact Windows full cargo fmt. Restore only
known test-generated unrelated result paths after retaining the scoped receipt.
After independent review, preserve source/regressions/concise evidence and remove
owned temporary targets/bundles/worktree only after link and process preflight.
