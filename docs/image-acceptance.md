# Image addon acceptance

Task 38 owns complete image integration. Subtasks own modules, not a substitute for the live addon. Implement in `crates/xterm-addons/src/image/` with browser-specific drawing under `crates/xterm-web/src/image/`. Source is read-only. No JavaScript terminal or protocol handler may execute the Rust implementation's image logic.

## IM0 public addon and graphics negotiation

Sources: `source_repo/addons/addon-image/src/ImageAddon.ts`, `Types.ts`, `typings/addon-image.d.ts`.

- Source defaults are enableSizeReports/sixelSupport/sixelScrolling/showPlaceholder/iipSupport/kittySupport true, pixelLimit 16777216, sixelPaletteLimit 4096, storageLimit 128 MB, and each protocol sizeLimit 33554432. Source values override stale declaration comments. Constructor options and reset defaults are distinct copies.
- Activation creates renderer/storage, forwards onImageAdded and registers CSI private h/l, DA1, XTSMGRAPHICS, render, DECSTR, RIS, request-reset, buffer-change and resize callbacks in source order. Register only enabled DCS q, OSC1337 and APC G protocol handlers. Test before open, after open, repeat activation, cancellation and disposal with real parser registration identity.
- Size reports mutate existing windowOptions object to enable exactly getWinSizePixels/getCellSizePixels/getWinSizeChars. Disabled enableSizeReports leaves it untouched. Disposal does not revert these flags.
- Reset restores only sixelScrolling and sixelPaletteLimit to constructor defaults, clears storage and resets handlers in insertion order; returns false so built-in RIS/DECSTR processing continues. Storage limit remains changed. Disposal traverses subscriptions in order, clears handlers and emitter, preserves source error/partial-disposal behavior and closes resources.
- Before activation storageLimit/storageUsage are -1. Storage limit setter updates source option only after storage succeeds. Placeholder getter/setter updates option and renderer immediately. Image/tile getters return absence when missing and actual browser canvases when present. Event count/order must be observable through loaded native/browser addons.
- DECSET80 disables sixel scrolling; DECRST80 enables it; arrays do not match numeric80. Both return false. DA1 consumes nonzero/array first parameter; zero/absent with sixel enabled reports exact ESC[?62;4;9;22c and consumes; disabled returns false.
- XTSMGRAPHICS with fewer than2 parameters consumes without reply. Items colors1/geometry2/unsupported include read1/default2/set3/max4 and unsupported action. Palette set accepts a scalar <=4096, including source numeric edge behavior, rejects missing/array/>4096, resets handlers on set-default. Compare exact success/item/action status bytes. Geometry uses live css.canvas or 7x14 cell fallback and terminal80x24 fallback; strict area<pixelLimit else floor(sqrt(limit)); test equality, fractional values, NaN/infinity and missing dimensions.

## IM1 streaming IIP header

Source: `source_repo/addons/addon-image/src/IIPHeaderParser.ts` and its unchanged test file.

- START/ABORT/KEY/VALUE/END states, parse -2 incomplete/-1 rejected/absolute payload offset, and end0/-1 outcomes match for full input, every single split, bytewise input, offset slices, empty chunks and repeated parse/end/reset.
- Recognize File, FilePart, MultipartFile, FileEnd and ReportCellSize. End accepts only standalone terminal markers or MultipartFile's final value. Preserve prefix-length quirks, the START pos>14 early return without abort, stale backing-buffer bytes and positions after early return, and colon/semicolon/equal handling in every state. Do not invent stricter validation.
- Fixed field capacity1024 accepts exactly-full fields and rejects the next codepoint; marker guard and field guard differ. Reset clears all1024 slots, current key, position, fields and state. Overflow/invalid input retains exact partial fields then sets type INVALID and state ABORT where source does.
- Keys use String.fromCharCode truncation of u32 to UTF16, may be empty, duplicate, numeric, non-BMP units or prototype-sensitive. Unknown fields retain independent Uint32Array copies; empty values, repeated semicolons, extra equals and duplicate keys preserve source semantics. Header fields can overwrite type. Trace property order where observable.
- Decimal fields accept only ASCII digits, empty=0, binary64 rounding/overflow retained. width/height accept only auto or digits with optional px/percent. Name uses base64 UTF8. Node Buffer's lenient base64 and browser atob's rejection differ; expose a deliberate adapter policy and test both, including malformed UTF8, padding and invalid alphabet.
- DECODERS inherits Object.prototype. Explicitly compare __proto__/constructor/toString and fields prototype assignment behavior instead of replacing these cases with a generic safe-map claim.

## IM2 image metrics and IIP payload lifecycle

Sources: `source_repo/addons/addon-image/src/IIPMetrics.ts`, `IIPMetrics.test.ts`, `IIPHandler.ts`, `IIPImageStorage.ts`.

- <32bytes is unsupported. Match PNG magic+IHDR and signed32 BE dimensions; JPEG marker walking with SOF0/SOF2, corrupt markers/truncated blocks and zero sizes; GIF87a/89a LE sizes; QOI signed BE sizes; WebP VP8X/VP8L/VP8 signatures, dimension packing, bad variant/signature; AVIF avif/avis ftyp and first ispe strictly before min(len-16,1024), positive signed dimensions only. Unaligned JS byteOffset exceptions need explicit binding tests. No unchecked indexing or endian-dependent Rust reads.
- IIP constructor computes ceil(sizeLimit*4/3), initial min(1MiB,max), retained4MiB decoder policy. Strict RFC4648 payload with optional padding, streaming arbitrary chunks, exact limit and over-limit, invalid characters and decoder release paths must match source decoder dependency, not a permissive name decoder.
- File inline false aborts; new File cancels pending multipart; MultipartFile captures defaults+fields and starts decoder at end; FilePart keeps transfer pending; FileEnd without pending or aborted transfer does nothing; success false releases without insertion. Reset's exact interaction with multipart flags must be compared, not silently fixed.
- ReportCellSize sends OSC1337 exact height;width;DPR toFixed3 using live dimensions or7x14 and DPR1, independent of image data. Test terminal before/after open and fractional geometry.
- Image decode finalizes base64 then metrics, rejects unsupported/zero/source pixel bound, computes dimensions with strict output area bound and floor. Dimension requests auto/cells/percent/px, width precedence, preserveAspectRatio, viewport-minus-one-row autoscale, float metrics and zero values need exact differential cases.
- QOI native decode and no-rescale fast path, browser createImageBitmap resize path, PNG/JPEG/GIF/WebP/AVIF images, decode rejection/warnings, release-before/after-copy ordering and async insertion must all work. Protocol accepted/aborted return true matches upstream without fabricating successful rendering.

## IM3 SIXEL

Sources: `source_repo/addons/addon-image/src/SixelHandler.ts`, `SixelImageStorage.ts`, source dependency sixel0.16.0.

- Async decoder readiness before/after hook, default VT34016+ANSI256 palette plus zero remainder to4096, transparent background parameter2=1, inverse/default/RGB/palette active colors and endian conversion.
- hook resets size/abort and initializes live palette limit. put counts codepoints only when decoder ready, strict >sixelSizeLimit abort, decoder errors log and release, later chunks ignored. Native Rust must actually decode sixel pixels, including repeat, color selection/definition, raster attributes, carriage/newline and malformed sequences against dependency fixtures.
- Reset releases/nullifies palette then reapplies defaults; successful unhook zero width/height versus height-only cursor advance, image upload,4MiB permanent-memory release threshold and actual RGBA output. Scrolling false captures/restores origin; scrolling true VT340 cursor positioning, linefeed and bottom-right boundaries.

## IM4 Kitty graphics

Sources: `source_repo/addons/addon-image/src/kitty/KittyGraphicsTypes.ts`, `KittyGraphicsTypes.test.ts`, `KittyGraphicsHandler.ts`, `KittyImageStorage.ts`.

- Parse all4 string keys and18 numeric keys, first equals splitting, comma segments without equals ignored, duplicate last write, unknown keys ignored, parseInt base10 prefix/sign/whitespace/NaN/-0/binary64 boundaries and lossless UTF16 strings. Do not reject malformed numeric values more aggressively than source.
- Control length512 exactly/overflow, split semicolon, streaming cumulative encoded limits and pending keys by explicit id/last pending/zero. i+I conflict responds early. Delete skips payload. Empty/success-false/aborted sequences, decode-error recovery, more1 accumulation, metadata from first chunk, last-pending clearing and memory release before/after Blob copy follow source order.
- Actions t/T/q/p/d with/without payload; unsupported action/medium, missing image, invalid base64, missing raw dimensions, insufficient RGB/RGBA bytes, PNG queries and exact response messages. quiet1 suppresses OK only, quiet2 suppresses all, placementId inclusion uses truthiness. Compare data event route, not just returned response text.
- Raw RGB→RGBA pixels for remainder0..3 and multiple4, PNG fallback object URL revoke/error, zlib then deflate-raw fallback, decompression errors and pixel bounds. Native protocol logic must not delegate to original JS.
- Placement source crop/offset, columns/rows/aspect scaling, cell fallback7x14, offset clamp to cell-1, explicit area clipping versus padding, z<0 bottom layer, rescale close order, cursor default versus C1 restoration after scroll. Async render failure returnsfalse and sends source response.
- Kitty storage auto IDs versus explicit0, overwrite displayed image, previous-delete callback order, wrapped callback restoration only if still owned, reverse-map staleness, shared eviction, max256 eviction down to128 undisplayed only. delete a/A clears all, i/I removes pending+image; unsupported selectors remain source no-op. Source supports one placement per image and unsupported animation/file/shm actions; port those exact limitations, do not claim full Kitty specification.

## IM5 shared image storage and terminal buffer

Source: `source_repo/addons/addon-image/src/ImageStorage.ts`.

- Limit0.5..1000MB, NaN/infinity/signed edge behavior, >>>0 pixel conversion, constructor fallback2,500,000pixels, synchronous FIFO eviction including resized copy accounting, strict room overflow, sparse removed IDs and marker-disposal/delete callback reentry.
- Add image preserves source ordering of eviction, ceil cell tiling/fallback, origin saved, dirty rows, clip-right, scrolling linefeed, non-scrolling origin placement, VT340 versus IIP final cursor, zero-tile cleanup, marker registration, alternate recount, insertion and onImageAdded. Compare actual extended payloads, tile counts and original attributes; replacing a tile reuses isolated payload and decrements old count without losing attributes.
- Normal/alternate marker disposal, wipeAlternate full-clear flags, explicit delete, reset, bitmap.close ownership and deletion callback exactly once. Missing marker behavior must match. Buffer edit/scroll/reflow/resize must transport payloads and evict images at real marker changes.
- Render lazily adds needed top/bottom layers, early abort if top insertion fails, rescale, clears once when empty, removes unused layers and obeys pending full clear. Inclusive viewport rows merge only same-image consecutive tiles, mark fullyCleared correctly and draw placeholders before stable z-index-sorted image calls.
- Width growth expands image tiles at old edge using source bounds and content-check behavior, including its loop-condition quirk; shrinking/height-only/no-images only update metrics. Get/extract checks absent rows/cells, imageId0/-1, tileId-1, original bitmap→canvas conversion and renderer tile return.

## IM6 browser renderer and resource lifetime

Source: `source_repo/addons/addon-image/src/ImageRenderer.ts`.

- Canvas creation uses terminal document or global fallback, ToInt32 dimensions; ImageData constructor/fallback typed-copy and createImageBitmap missing-function result. Opening before/after addon, patched open/setRenderer, fontSize events, replacement renderer removing layers, disposal restoring originals and closing placeholder bitmap.
- DOM output uses terminal document and screenElement; missing host warns, duplicate layer no-op, alpha context failure removes canvas, isolation=isolate without overwriting integrator z-index; bottom first child z=-1 and top append z=0 below selection overlay. Test iframe ownerDocument and disposal.
- clearLines inclusive start/end with fractional cell metrics; clearAll optional layer. draw requires valid cell metrics, rescale original/actual identity decisions, tile source clipping for Safari, floor offsets and ceil spans. extractTile floors dimensions and returns absence without context. Verify exact pixel assertions at fractional DPR and cell sizes, not startup screenshots only.
- Placeholder checkerboard32px blueprint, black/white RGBA, row parity offset, height24/min(cell+1), screen width32 alignment or4096 fallback; bitmap replacement close, disabled placeholder refresh and async lifecycle. Browser resource failures preserve source-visible outcomes and retain cleanup evidence.

## IM7 evidence, APIs and integration

- Inventory every original image unit and Playwright suite, preserve test bodies/assertions, exact discovered/executed/passed/failed/pending/unrun counts and fixture hashes. Header/metrics/types oracles are utility evidence, not live image support.
- Shared deterministic and seeded Rust/JS traces compare parser state/fields, bytes, pixel matrices, buffer cells/attributes/payloads, cursor/scroll/modes, memory accounting, replies/events, ordering, bitmaps and disposal. Retain seeds, randomized iteration counts and minimal/replay failing prefixes. Negative controls must show source behavior mutations fail the oracle.
- Load actual Rust-backed addon in Chromium/Firefox/WebKit and native Rust terminal. Exercise SIXEL/IIP/multipart/QOI/Kitty transmission/display/delete, resize/reflow/alt switch/reset, low memory eviction/placeholders, layering, image retrieval/tile extraction, async errors, disabled protocols and disposal. Every public option/event/property maps to callable Rust/WASM APIs.
- Run scoped tests, formatting and warning-denied Clippy plus workspace/feature/WASM gates; independently review protocol allocation bounds and bitmap ownership. Do not turn malformed-input failures into panics. Native Windows evidence is separate from Linux/macOS and browser results. Overall image equivalence remains NOT PROVEN while any protocol, rendering or live API path is missing.
