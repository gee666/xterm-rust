# WebGL addon port

Task39 owns the actual Rust WebGL2 renderer and addon. This is not complete. The native storage helpers now present under `crates/xterm-web/src/webgl/` are not a working GPU renderer, and no WebglAddon constructor is exported on their behalf. Browser evidence must exercise WebGL2, not DOM or JavaScript engine fallback.

## Shared interfaces and ownership

Sargon owns `crates/xterm-web/src/webgl/`, its focused tests/oracles and the `webgl.mjs` acceptance adapter. Cornelia owns the DOM renderer, theme/geometry services and `renderer::service_api::Renderer` replacement contract. Xavier owns the browser terminal, root WASM exports and web-sys feature list. No agent should rewrite another's service/manifest files to force integration.

The existing wasm-bindgen/js-sys/web-sys dependencies supply the GPU boundary. The port adds only required WebGL2, observer and path features to that existing dependency, not a new graphics framework. Portable buffer/model/cache algorithms remain testable without a browser. GPU object wrappers, canvas rasterization, observers and DOM listeners are WASM-specific. Safe web-sys handles own JavaScript objects; no Rust unsafe block is needed for this approach.

The renderer must implement every required source `IRenderer` method. Optional viewport-visibility and atlas operations use capabilities, not successful no-op required methods. Activation replaces the live renderer through the shared service, and disposal restores/resizes the default renderer only while the terminal is alive. Before-open activation subscribes to real onWillOpen. A working class will not be advertised before that path exists.

Source CharAtlasCache uses one module-global pool shared across terminals. The Rust design should use an explicitly owned shared atlas pool passed by the browser bootstrap to addons, rather than a mutable global in the portable core. Any JavaScript bootstrap is limited to obtaining/passing this Rust owner and forwarding binding calls, not drawing, rasterization, terminal execution or atlas policy. The bootstrap and ownership boundary need independent review before activation is considered implemented.

## Source components

| Source under addon-webgl/src | Rust component and status |
| --- | --- |
| TypedArray.ts | `webgl/typed_array.rs`, implemented but native/WASM verification pending during disk pause |
| RenderModel.ts | `webgl/model.rs`, implemented but verification pending |
| Types.ts | Cursor/storage types present; atlas/GPU contracts still to implement |
| CharAtlasUtils.ts | Config generation/equality implemented using shared ColorSet/ColorRef/cache types; verification pending |
| CharAtlasCache.ts | Explicit shared pool/weak terminal ownership and source-ordered release implemented; verification pending |
| WebglUtils.ts | Projection/growth/version bookkeeping implemented; actual GPU resource operations still planned |
| CellColorResolver.ts | Planned packed-color resolution over existing decoration/selection services |
| GlyphRenderer.ts | Planned glyph instance model, uniforms, texture pages and shaders |
| RectangleRenderer.ts | Planned background/cursor batching and shaders |
| TextureAtlas.ts | Planned cache, rasterization, page allocation/merging/eviction and versions |
| CursorBlinkStateManager.ts | Planned clock-driven cursor work and cancellation |
| DevicePixelObserver.ts | Planned real ResizeObserver device-pixel callback boundary |
| renderLayer/BaseRenderLayer.ts and LinkRenderLayer.ts | Planned link canvas overlay, selection/style updates and cleanup |
| customGlyphs/CustomGlyphDefinitions.ts | Planned attributed generated shape data, retained with source hash |
| customGlyphs/CustomGlyphRasterizer.ts and Types.ts | Planned shape/pattern paths and main-realm/offscreen canvas selection |
| WebglRenderer.ts | Planned live renderer service implementation |
| WebglAddon.ts and typings/addon-webgl.d.ts | Planned callable addon class/options/events/lifecycle bridge |

These planned components are work, not external blockers. They are not compiled as empty placeholder implementations.

## Exact User case mapping

Criteria328-352 are User-authored contracts. Their command IDs remain unchanged.

| Case | Criterion | Required source behavior and evidence |
| --- | --- | --- |
| gpu-01 | 328 | Deferred/open activation, renderer replacement, disposal/restoration/resize and terminal-first teardown |
| gpu-02 | 329 | customGlyphs/preserveDrawingBuffer defaults and explicit values; missing WebGL2 and Safari<16 capability/error behavior |
| gpu-03 | 330 | Context loss during/without rendering, event timing, restoration/redraw and resource/listener cleanup |
| gpu-04 | 331 | Atlas getter canvas identity and ordered change/add/remove events, including clear and disposed subscribers |
| gpu-05 | 332 | Complete applicable SharedRendererTests assertions against actual Rust GPU pixels, retaining source browser skips |
| gpu-06 | 333 | Every config field and every ANSI entry independently affects equality/cache invalidation exactly as source |
| gpu-07 | 334 | Equal-config sharing, config changes, both disposal orders, reacquisition and final-owner cleanup |
| gpu-08 | 335 | Shared-atlas merging invalidates glyph page/layout references for every terminal, retaining pixels |
| gpu-09 | 336 | Shared clear invalidates other renderers' old glyph instances without manual reset |
| gpu-10 | 337 | Nonmergeable page-cap eviction, layout version/events, bounded textures and valid redraw |
| gpu-11 | 338 | Oversized joined glyph allocation evicts before adding and respects device texture limits |
| gpu-12 | 339 | Original alternate-screen/TUI atlas stress trace, primary pixels, page/resource bounds |
| gpu-13 | 340 | Pattern glyph redraw after same-origin document adoption with valid canvas ownership |
| gpu-14 | 341 | Prefer OffscreenCanvas without DOM factory; otherwise main-realm DOM factory, exact calls/assertions |
| gpu-15 | 342 | Box/block/braille/powerline/progress/git/legacy ranges, fractional DPR/spacing, joins and font fallback |
| gpu-16 | 343 | Codepoint/combined UTF-16 plus bg/fg/ext/restriction cache identity; bounds/offset/clip/empty/long glyphs |
| gpu-17 | 344 | Top/bottom decoration, selection/inverse, underline and RGB/indexed/default color precedence |
| gpu-18 | 345 | Dim/normal contrast caches, bold-bright palette, transparent/inverse and contrast-exempt glyphs |
| gpu-19 | 346 | Every underline style/color and strikethrough, wide/joined cells, DPR geometry, clipping/padding |
| gpu-20 | 347 | Resize/DPR/font/cell sizes, actual observer entries/fallback, viewport/canvas/atlas invalidation |
| gpu-21 | 348 | Model/dirty-row updates after narrow-over-wide/combined/joined writes, scrolling and buffer switches |
| gpu-22 | 349 | Equal/different backgrounds, transparency, cursor styles/focus, batch order/clipping and pixels |
| gpu-23 | 350 | Virtual-clock blink transitions, hover/leave underlines, bounds, cancellation/disposal/replacement |
| gpu-24 | 351 | All eleven original typed-array cases across nine types, defaults/negative/out-of-range/empty, float values and copied storage |
| gpu-25 | 352 | Shader/program creation/compile/link failures, repeated render/dispose/reload and GPU/event resource cleanup |

Every command is `node scripts/acceptance-oracle.mjs --case gpu-NN`. The adapter must return independent reference and Rust/WASM captures on Chromium, Firefox and WebKit with real runtime versions, WASM/source hashes, nonzero assertions, pixels/state/events and the exact case ID. Missing implementation/runtime fails. A source-defined GPU skip must retain its condition, evaluated reason and source identity. All skips or zero assertions cannot pass. See `docs/acceptance/runner.md`.

## Portable storage binding decisions

TypedArray helpers accept `Option<f64>` start/end so omitted values, fractions, infinities and NaN do not disappear through an integer API. Modern slice and the legacy fallback remain separate. The fallback deliberately retains JS remainder and Math.min behavior, including differences from modern slice below negative length or at fractional indices. Invalid source numeric indexes assign undefined, which becomes zero for integer arrays and NaN for floating arrays. The nine JS constructors map to eight Rust numeric storage types because Uint8ClampedArray already contains clamped u8 values before slicing. A returned Vec owns its values; allocation errors propagate instead of aborting on untrusted sizes.

RenderModel preserves source constants and packed arrays. Resize reallocates only when cols*rows*4 changes, even when a changed row count would otherwise suggest a new line-length array. Clear fills only packed cells and line lengths; selection and cursor remain. Checked native overflow/allocation errors represent the boundary where JS typed-array construction would fail. The model reuses Cornelia's portable selection projection instead of a duplicate implementation.

## Source details that must survive GPU integration

- GLSL/GPU code must retain JavaScript numeric representation where observable. Packed FG/ext from bitwise operations may be negative while the same model Uint32 entries are positive. Use signed-aware numeric cache keys and strict comparisons, not premature u32 normalization. The shared contrast cache now accepts numeric keys preserving -1 versus 4294967295.
- The combined character marker is a JS bitwise OR result, negative before model storage. `allowRescaling` therefore rejects it at the source's codepoint > 0xff check. The portable helper accepts i64 to retain that distinction.
- Atlas numeric rasterization uses String.fromCharCode, truncating to 16 bits. Astral text normally follows the separate combined UTF-16 cache path. restrictToCellHeight is not part of either source four-key cache.
- Glyph null-cell clear fills only the first eight of eleven floats, leaving texture-size-y and the two position fields. Do not broaden the clear and assume exact state parity.
- RenderModel compares the unmarked code before writing the combined marker, making combined cells dirty again on later frames. JoinedCellData copies fg/bg but not extended state, uses fake code0x1fffff and intentionally rejects setFromCharData. These are source contracts, not opportunities for silent correction.
- GPU geometry differs from the DOM helper: floor device character width, ceil height, source JS rounding for spacing/offsets, and derive CSS cell size from device cell size divided by DPR. Reuse shared geometry types without substituting the DOM rounding algorithm.
- The source renderer constructs its first atlas before the addon attaches event forwarders. Do not invent initial forwarded change/add events. Context-loss notification waits 3000ms; the debug logger runs before preventDefault, and restoration removes the cached owner, rebuilds GPU state and requests redraw.
- Warmup uses33 <= code <126. ClearTexture returns early when the first page's current row is at origin, even if other pages exist. Page/layout versions are monotonic shared values, not consume-once dirty flags.
- Shader and texture limits are initialized from the first context. Assignment order matters if a later parameter read fails; do not replace source partial initialization with an unexplained atomic reset.
- Image integration patches the current renderer setter. GPU installation must route the live setter facade rather than a saved base alias or a direct native setter that bypasses image127. Cornelia/Xavier own that shared route.
- Known-tag canvases and actual create-context results can originate in another document. Parent-realm instanceof checks reject valid cross-realm objects; a safe web-sys unchecked JS wrapper cast must document the native DOM operation proving the type.
- Canonical stylesheet is browser/xterm.css. No addon-specific duplicate copy is needed.

## Evidence status

No GPU case has yet passed against the Rust addon. The new portable tests have not run during the coordinated low-disk pause. Original source remains read-only. Full terminal/browser equivalence is NOT PROVEN. Future observed results must replace this status only with retained commands, counts, hashes and actual native/browser evidence, not module existence or covered votes.
