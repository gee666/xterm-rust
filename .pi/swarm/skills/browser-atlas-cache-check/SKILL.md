---
name: browser-atlas-cache-check
description: Use when changing shared browser WebGL atlas ownership, runtime limits, lazy services, disposal or creation phases.
---

# Shared browser atlas cache checks

Read `docs/webgl-browser-cache.md` for typed Rust boundaries and the remaining task306 integration scope. Reuse `scripts/browser-host-check.mjs`; do not create another browser runner.

1. Set `SOURCE_ROOT` to the untouched original source with its installed esbuild dependencies. Run `node scripts/browser-atlas-cache-oracle.mjs`. It bundles unchanged CharAtlasCache plus actual TextureAtlas in memory and pins input/wrapper/bundle hashes next to `tmp/browser-atlas-cache/source.js`.
2. Set `WASM_TEST=wasm_webgl_browser_cache` and `WASM_PRELUDE` to that bundle. Set `PLAYWRIGHT_MODULE`/`WASM_BINDGEN` if their existing tools are outside the owned worktree. Run `node scripts/browser-host-check.mjs` with a coordinator allocation and private O: target/TEMP/TMP, Cargo -j2. Require Chromium, Firefox and WebKit, no ignored/filtered tests.
3. Run focused native `webgl` assertions and warning-denied native/WASM all-feature Clippy for the changed lib/test. Retain exact failed and final commands. Try the required full workspace fmt; record Windows OS206 failure separately from package/changed-file passes.
4. Negative control: change the production pool factory to use a fresh AtlasRuntime per newly created atlas. The source version comparison must fail. Restore exact production bytes in finally, then rerun the full three-browser matrix and strict WASM Clippy. Do not mutate reference source or weaken assertions.
5. Retain concise reports, source/input hashes and final served compiler/bindings/snippet identities before deleting owned temporary outputs. Check original reference hashes after execution and cleanup. Do not delete another owner's runtime files.

These are real cache/atlas component tests, not full WebGL addon/renderer/Root or unchanged-suite acceptance. No source engine runs in the Rust implementation.
