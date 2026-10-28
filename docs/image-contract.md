# Image implementation contract

Acceptance sources are User criteria280-327, copied verbatim in `docs/image-user-criteria.md`. `docs/image-acceptance.md` IM0-IM7 adds internal source branches and quirks. Both apply. Source runtime is authoritative over stale typings comments. Task38 is not complete until actual Rust terminal/browser protocols, storage and drawing work together.

## Ownership

- Marie: image/mod.rs, image/types.rs, header.rs, metrics.rs, IIP handler/integration, image public addon, source helper oracle and task38 integration. Root addon lib.rs remains Lane's.
- Viesturs task124: image/kitty/{mod,command,storage,transmission,pixels}.rs and focused Kitty tests/oracle. Name choices inside this directory belong to Viesturs. No browser draw files.
- Nuwa task125: image/sixel/ decoder, source-handler policy, palette/background extraction, tests and original dependency pixel oracle. No browser draw files.
- Task126: image/storage/ with actual BufferService, cell extended payloads and Marker lifetime. Still needs owner.
- Task127: xterm-web/src/image/ with real canvas/bitmap DOM renderer and pixel tests. Still needs owner.

## Native pixel contract

Shared `image::types::RgbaImage` owns row-major tightly packed RGBA bytes with unsigned pixel dimensions. Construction validates width*height*4 with checked arithmetic and exact length before storing. It may represent zero dimensions with empty bytes; protocol policy decides whether that result means cursor-only advancement or rejection. All protocol sizing/control metadata remains binary64 where the JavaScript source admits NaN, signed zero, fractions or rounded integers. Only actual decoded raster allocation crosses into bounded integer dimensions.

No new decoder dependency is assumed. Explain license, MSRV, target support and source parity before adding one. SIXEL source dependency is installed `sixel`0.16.0, MIT by Joerg Breitbart. Nuwa is implementing a safe native decoder and comparing the original pinned dependency's output. Native decoders return actual pixels, not a fake success or a JS-engine callback.

## Protocol and host separation

Implement protocols natively with explicit effects and completion, not browser objects in core state. A handler may request decoding/display and retain pending completion, but must not report rendering success until the host completed real work. Source action/reply/error order must remain testable. Browser-native PNG/JPEG/WebP/AVIF decoding is a permitted platform image operation; the source JavaScript protocol/state engine is not.

For initial parallel implementation, Kitty owns its own source-byte image records and pending transfer metadata. It emits typed display/delete operations and response bytes, with an explicit display-completion method or host trait chosen by Viesturs and documented here. The main addon adapts that interface to shared storage and asynchronous browser decoding. Do not couple its implementation to a not-yet-written fake renderer. Native tests can supply a recording display host only to verify command/effect order, not claim end-to-end pixel rendering.

SIXEL returns width/height/RGBA plus source-handler actions for cursor-only advancement, warnings, or image insertion. Shared storage consumes a real decoded image and source placement options: scrolling, top/bottom layer, z-index, VT340/IIP cursor policy. The native host reads current theme/attribute colors for hook. Empty width/nonzero height stays representable separately from image insertion.

Shared storage owns image/tile identity, marker subscriptions and cursor effects. Browser renderer owns actual canvas/bitmap resources and source-exact draw commands, including cropping/resizing, layer insertion, placeholder pixels and close calls. Do not put DOM dependencies in xterm-core or xterm-addons. Native terminal adapter and browser host must preserve the same buffer payloads and protocol effects. Final concrete storage/renderer handles will be agreed with task126/127 owners before wiring; these open interfaces are work, not external blockers.

## Required execution entrypoint

User cases require `node scripts/acceptance-oracle.mjs --case img-01` through `img-48`, unchanged source assertions and Rust-backed WASM in applicable Chromium/Firefox/WebKit. The shared acceptance runner does not exist yet. Task38 must integrate its image cases with the shared runner owner and reject zero-test/missing-WASM cases. Helper-only scripts must not satisfy these live-image acceptance cases by themselves.
