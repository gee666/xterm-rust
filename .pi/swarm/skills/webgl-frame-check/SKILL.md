---
name: webgl-frame-check
description: Use when changing typed WebGL model-to-GPU frame composition, shared atlas acquisition, frame resize, attachment, restoration or disposal.
---

# Actual frame checks

Read docs/webgl-browser-frame.md. This component uses real Rust canvas/model/atlas/GPU code. It is not the complete raw browser renderer or WebglAddon.

Reuse scripts/browser-host-check.mjs with WASM_TEST=wasm_webgl_frame. First build scripts/webgl-frame-oracle.mjs with SOURCE_ROOT pointing to the immutable original source and its installed esbuild. Supply WASM_PRELUDE=tmp/webgl-frame/source.js, wasm-bindgen CLI0.2.100 and that source's Playwright module. Run all Chromium/Firefox/WebKit cases without filters. Missing WebGL2 is failure.

The oracle constructs the unchanged full WebglRenderer with original Terminal and real GPU components. Its only injected service is fixed character measurement. Eleven Rust functions cover nine source scenarios with sixteen full framebuffer/model comparisons, plus two typed lifecycle groups. Original source and Rust pixels are compared within the same engine. Exact resize cache churn is intentional and source-asserted.

Run cargo test --locked -p xterm-web --test webgl --test webgl_renderer, native and WASM warning-denied library Clippy, new WASM test Clippy and formatting. Follow current team grants, owned O: targets/TEMP/TMP, Cargo -j2 and resource guards. No retired global queue.

scripts/webgl-frame-negative.mjs clears actual production glyph geometry before draw. It requires ordinary assertion failures in all three engines and restores original source bytes in finally. Rerun the full positive matrix afterward. Never mutate the reference or weaken assertions.

SOURCE_ROOT=/path/to/reference node scripts/webgl-frame-audit.mjs --record --runtime records producer pins. An independent reviewer runs without --record. Default verification checks original/source/durable pins after owned runtime cleanup; --runtime requires all generated artifacts still present. Do not claim a producer snapshot is independent review.

Preserve exact failures, concise logs and source/build hashes before guarded removal of owned generated artifacts. Parent306 retains rawhost/reentry, complete constructor/subscriptions, Addon/Root and full original suites. Context540's initial burst/GC counterexample is still open.
