# WebGL TextureAtlas

Task 423 implements the actual atlas over the accepted rectangle, glyph, palette, page, pixel, signed-width and custom-character components. It does not export a public WebglAddon. Task 306 still requires WebglRenderer orchestration, Root integration, shared runtime installation, context lifecycle and unchanged full addon/demo suites. Overall equivalence remains NOT PROVEN.

## Implementation

`webgl/texture_atlas.rs` owns glyph caches and implements the existing GlyphAtlas contract. Returned GlyphRefs retain shared coordinates through page merges. Numeric keys use String.fromCharCode semantics, including signed combined-cell markers. Combined keys preserve UTF-16 units, including lone surrogates. Height restriction and parent element are intentionally absent from cache keys. Empty and invisible glyphs share one runtime-owned null object.

`atlas_rasterizer.rs` draws real Canvas2D text and accepted Rust custom glyphs. It attaches its hidden temporary canvas to inherit font-feature-settings. Platform baseline selection matches the source module snapshot. Background and foreground resolve in their separate source drawing phases. Underline, overline and strike paths are in `atlas_styles.rs`. Invisible glyphs leave the source save frame unbalanced. Strike y-offset reads the previous line width before changing it.

`atlas_pack.rs` ports current/fixed row selection, threshold handling, exact edge comparisons and oversized-glyph pages. `atlas_store.rs` performs quad merges, coordinate alias updates, version shifts, eviction and page events. Page/list/cache borrows end before events, Unicode callbacks, loggers and canvas calls. The browser binding must pass one AtlasRuntime to every atlas in its existing AtlasCache. That owner supplies shared limits, ConfigFactory, PageClock and the null glyph. No new terminal or Unicode authority exists.

Warmup uses the existing source-selected BrowserTaskQueue and existing terminal LogService. It queues ASCII 33 through 125 once, checks the live cache per task and resets its warmed flag on clear/eviction. Rust queue ownership uses weak atlas callbacks so dropping the owner does not create a reference cycle. Explicit atlas disposal removes canvases and disposes events, but does not cancel already queued warmup tasks while the atlas remains owned.

The existing `prepare_image` component API remains copy-producing. The actual `prepare_canvas` path processes getImageData's JS-owned pixels in place, without a replacement ImageData constructor. A real constructor Proxy demonstrated that the previous helper added an observable call absent from TextureAtlas.ts. Rust still performs filtering and bounds arithmetic. No renderer or terminal engine is delegated to JS. No dependencies or unsafe code were added.

## Verification

Use `.pi/swarm/skills/webgl-texture-atlas-check/SKILL.md`. The oracle bundles unchanged source in memory and records imported input hashes. The original reference files and assertions are not edited. Each engine compares its own source and Rust Canvas output rather than comparing platform fonts across engines.

The directed scenarios check text/custom characters, all underline styles, strike/overline, transparency, dim/contrast, high DPI/line height, disabled custom glyphs, cache identity, empty/invisible objects, lone surrogates, combined strings, oversized ligatures, row packing, page merge/eviction, version changes, clear metadata and event reentry. One seeded scenario uses 192 mixed glyph/style steps with seed 0x31e0a4. Final pages include full RGBA FNV-1a hashes and nonzero-alpha counts, alongside exact glyph coordinates, retained aliases, packing rows, page versions, used area and event order.

Host probes use genuine contexts and temporary prototype interceptors. They compare original thrown Symbol identity, background/font/foreground phases, fillText lookup failures, failed-upload row advancement and retry behavior, invisible save behavior and the absence of an extra ImageData constructor. They restore every prototype/global in finally before Rust assertions. These are selected boundary probes, not a claim that every possible host-object mutation has been exhaustively tested.

The GPU test installs the actual TextureAtlas in the existing GlyphRenderer, draws a real custom block through WebGL2 and verifies pixels, layout invalidation, clear/rebuild and GPU error state. This is component integration, not a substitute for the public addon or full renderer.

A warmup test first compared real-time callback counts and failed only in WebKit, 5 Rust callbacks versus 2 JS callbacks. It already matched pixels and metadata. The retained failure is a nondeterministic probe, not an atlas compatibility fix. The corrected probe supplies the same virtual performance clock and scheduler to both implementations and retains the exact callback-count assertion. A separate genuine constructor negative control failed 11/12 functions before the in-place canvas fix, then passed.

At the pre-review checkpoint, all 14 atlas test functions passed in Chromium, Firefox and WebKit, 42 executions with no ignored or filtered tests. The unchanged 379 regression suite passed three functions per engine, including all 656 source fixtures. Native xterm-web all-target/all-feature tests and warning-denied Clippy passed. Scoped WASM library/atlas/pixel-test Clippy passed. Receipts distinguish each checkpoint and retained failure. A later display assignment correction uses the source's strict CSS property setter rather than setProperty; review must refresh the final runtime receipt against that frozen source.

Exact `cargo fmt --all -- --check` still fails Windows OS206. All five workspace members pass their individual format checks. This is not reported as an exact-command pass. Full workspace all-feature quality, native Windows/Linux/macOS, upstream/full-browser/demo acceptance and release comparisons remain the project-level obligations.
