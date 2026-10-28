# User image acceptance cases

Verbatim criterion bodies and verification from User criteria280-327. The live swarm criterion is authoritative if edited later. Source module paths abbreviated there resolve within source_repo/addons/addon-image/.

## 280: Image option defaults follow runtime, not stale typings

Construct with no options, then override each flag/limit independently. Expect runtime defaults pixelLimit=16777216, palette=4096 and sequence limits=33554432; overrides leave other defaults unchanged.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-01. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts DEFAULT_OPTIONS; test/ImageAddon.test.ts ctor options

## 281: Image activation preserves size-report option policy

Activate with enableSizeReports true/false. True enables windowOptions 14/16/18 reports; false preserves caller settings. Disposal must not restore the old settings.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-02. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts activate; typings/addon-image.d.ts enableSizeReports

## 282: Image support flags gate protocol registration

Disable sixelSupport, iipSupport and kittySupport separately. DCS q, OSC1337 and APC G must dispatch only to enabled handlers; unrelated fallbacks stay callable.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-03. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts activate

## 283: Image reset restores protocol settings but not storageLimit

Change palette, DEC80 and storageLimit, add images, then reset via API, RIS and DECSTR. Clear storage/pending protocols, restore constructor palette/scroll policy, preserve runtime storageLimit; return false to continue reset dispatch.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-04. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts reset

## 284: SIXEL DEC80 switches scrolling with source polarity

Send CSI ?80h then CSI ?80l around palette.sixel. Set disables SIXEL scrolling, reset enables it. Cursor row and buffer scroll must equal the upstream VT240/340 fixture results.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-05. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts _decset/_decrst; test/ImageAddon.test.ts scrolling & cursor modes

## 285: SIXEL advertises the exact primary device attributes

With sixelSupport=true, CSI c/0c replies ESC[?62;4;9;22c. Nonzero DA parameter is consumed without that reply; disabled support preserves the next core handler.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-06. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts _da1

## 286: Graphics palette queries preserve status and limits

Exercise CSI ?1;1S, ?1;2S, ?1;3;4096S, 4097 and colon subparams, ?1;4S and invalid actions. Expect source success/action-error replies, constructor-default reset and no accepted oversized palette.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-07. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts _xtermGraphicsAttributes

## 287: Graphics geometry queries honor pixel-limit boundaries

Query CSI ?2;1S and ?2;4S with measured/unmeasured canvas and area below/equal/above pixelLimit. Expect rounded CSS dimensions below limit, floor(sqrt(pixelLimit)) square otherwise; ReGIS/unknown item returns item error.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-08. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts _xtermGraphicsAttributes

## 288: SIXEL split streams decode the same palette pixels

Replay palette.sixel as one write, bytewise and at every DCS boundary. Expect identical RGBA tiles, palette selection, repeat/raster handling and terminal cursor/scroll trace, not merely a successful parser return.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-09. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/SixelHandler.ts; fixture/palette.sixel

## 289: SIXEL transparency uses the active inverse background

Decode identical SIXEL with P2=1/0 under default, indexed, RGB and inverse SGR backgrounds, including before open. Expect transparent fill only for P2=1 and source-equivalent selected background/endian RGBA otherwise.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-10. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/SixelHandler.ts hook/extractActiveBg

## 290: SIXEL size overflow aborts without corrupting next image

Feed exactly sixelSizeLimit and one extra unit, decode errors, CAN/SUB and unhook(false), then a valid image. Expect source abort/release behavior, no rejected tiles, and successful recovery with correct cursor.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-11. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/SixelHandler.ts put/unhook

## 291: SIXEL empty raster preserves source cursor movement

Feed width=0 with nonzero raster height, empty raster and ordinary image at the bottom margin with scrolling on/off. Match advanceCursor and final cursor/buffer results, including no stored canvas for empty width.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-12. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/SixelHandler.ts unhook; src/SixelImageStorage.ts

## 292: IIP headers parse identically across chunk boundaries

Parse File=, FilePart=, MultipartFile=, FileEnd and ReportCellSize at once and bytewise. Match field types, data-start offsets and terminal parser state; preserve the original HeaderParser assertions.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-13. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHeaderParser.ts; src/IIPHeaderParser.test.ts

## 293: IIP malformed header fields abort at source boundaries

Exercise missing starter, empty key, empty/nonnumeric size, empty name, illegal characters, unknown fields and 1024/1025-character fields. Match accepted fields, abort state and reset recovery exactly, including JS numeric parsing.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-14. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHeaderParser.ts; src/IIPHeaderParser.test.ts

## 294: Image metrics identify every supplied format fixture

Run IIPMetrics testimages across PNG/JPEG/GIF/QOI, VP8/VP8L/VP8X WebP and AVIF. MIME and dimensions must equal each preserved fixture assertion, including EXIF JPEG and odd AVIF dimensions.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-15. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPMetrics.ts; src/IIPMetrics.test.ts; fixture/testimages

## 295: Image metrics reject truncated or invalid headers

Test 0/31/32-byte inputs, bad PNG IHDR, JPEG missing SOF/truncation, bad VP8 signatures and AVIF ispe outside scan limit. Expect source unsupported or zero-dimension result without panic or out-of-bounds read.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-16. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPMetrics.ts imageType/jpgSize

## 296: IIP base64 failures do not mutate terminal cells

Feed bad alphabet/padding/separators, exact/over iipSizeLimit, inline=0, unknown image format and success=false, then valid File=. Match warnings/abort and ensure no rejected image, cursor advance or placeholder.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-17. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts put/end

## 297: IIP size metadata remains optional and informative

Transmit the same PNG with absent, exact and inaccurate size metadata. Expected displayed pixels and cursor equal upstream, rather than enforcing the obsolete declaration comment that size is required.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-18. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts; test/ImageAddon.test.ts size params is optional/informative

## 298: IIP multipart assembly preserves cancellation rules

Replay MultipartFile/FilePart/FileEnd with chunks, empty parts, malformed parts, stray FileEnd and intervening File=. Only complete valid payload adds one image; intervening other sequence cancels the old transfer exactly as upstream.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-19. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts _isMultipart/_abortMulti; test/ImageAddon.test.ts FilePart support

## 299: IIP cell-size report uses CSS metrics and three decimals

Request OSC1337;ReportCellSize before/after measurement and at fractional DPR. Expect ESC]1337;ReportCellSize=h;w;scale ST with three decimals and source default-cell fallback, emitted as non-user input.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-20. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts end REPORTCELLSIZE

## 300: IIP resizing converts cell pixel and percent dimensions

Render each source image with width=20,height=5; 320px/160px; 50%/30%; auto and one omitted dimension. Expected floored dimensions and aspect-ratio policy equal source, including viewport-fit downscale without upscale.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-21. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts _resize/_dim; test/ImageAddon.test.ts IIP - resizing

## 301: IIP pixel limits reject both decode and resized overflow

Exercise zero dimensions and original/resized area just below, equal to and above pixelLimit. Only source-accepted images allocate stored canvases; rejected images leave buffer and cursor unchanged.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-22. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts end

## 302: IIP image decoding retains actual format pixels

Render palette PNG/QOI and supplied spinfox/GIF/JPEG/PNG fixtures. QOI and PNG palette bytes must match exactly; other outputs match upstream assertions, including first-frame/static decode and decode rejection.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-23. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/IIPHandler.ts end; test/ImageAddon.test.ts IIP support - testimages/IIP - QOI support

## 303: Image storage limit evicts FIFO synchronously

Set storageLimit below 0.5, at 0.5, at 1000 and above 1000 MB; grow then lower it under pressure. Match RangeError boundaries, FIFO victims and immediate storageUsage before setter returns, counting RGBA pixels*4.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-24. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts setLimit/_evictOldest; test/ImageAddon.test.ts storageLimit

## 304: Image added events fire once after accepted storage

Write two valid images, then an invalid/aborted image. Expect one onImageAdded per accepted image, source-visible storage state in callbacks, stable subscription disposal and no event for rejected data.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-25. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts addImage; src/ImageAddon.ts onImageAdded; test/ImageAddon.test.ts onImageAdded

## 305: Image scrollback trim frees the last referenced image

Render image spanning several lines, trim first/some/last markers, and scroll entirely off. Storage survives while source retains references and is released at the same trim transition; usage and placeholders match.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-26. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts; test/ImageAddon.test.ts delete image once scrolled off

## 306: Alternate-buffer image cleanup preserves normal images

Store normal and alternate images, switch buffers repeatedly and manually evict alternate entries. Only source-selected alternate images disappear; no skipped entry during map mutation and no loss of normal buffer tiles.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-27. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts wipeAlternate/_evictOnAlternate; test/ImageAddon.test.ts alternate images/manual eviction

## 307: Text overwrite removes image tile references correctly

Overwrite first/last/all tiles of SIXEL and IIP with text and erase operations. Expect source cell extended attributes, tile counts, remaining pixels and final storage deletion, with no stale full-image overlay.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-28. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts _writeToCell/render; test/ImageAddon.test.ts text overwrite removes tiles

## 308: Image viewport resize extends and clips the same tiles

Resize narrower/wider and change cell metrics/DPR with partial images at the right/bottom edge. Match source tile IDs, empty extensions, clipping and rendered positions, preserving normal/alternate ownership.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-29. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts viewportResize; src/ImageRenderer.ts

## 309: Image cell accessors return original and cropped canvases

Call getImageAtBufferCell/extractTileAtBufferCell before activation, on empty/out-of-range cells, and first/edge image tiles. Match undefined versus canvas, original identity, tile dimensions and RGBA cropping.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-30. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageStorage.ts getImageAtBufferCell/extractTileAtBufferCell; typings/addon-image.d.ts

## 310: Image placeholders toggle without reviving evicted data

Evict an image, toggle showPlaceholder true/false repeatedly and render. Match source placeholder pixels/refresh behavior, preserve missing image state and return -1 storageUsage/storageLimit before activation.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-31. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts getters/showPlaceholder; src/ImageRenderer.ts

## 311: Image disposal detaches parser render and resize hooks

Load/dispose addon before/after images and while decode is pending, then send protocols/resize/render. Match upstream post-dispose callbacks and retained state, remove owned resources/listeners, and do not dispose the terminal.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-32. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/ImageAddon.ts dispose; src/ImageRenderer.ts dispose

## 312: Kitty command parsing preserves absent and empty fields

Parse every control key with missing '=', empty values, repeated/unknown keys, negative z, C=0, p and I, and parseInt suffixes. Match undefined, empty string and numeric results from parseKittyCommand, not Rust defaults.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-33. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsTypes.ts; src/kitty/KittyGraphicsTypes.test.ts

## 313: Kitty transmit stores pixels without accidental display

Send 1x1 black/3x1 RGB PNG with a=t, a=T, omitted a and a=. Expect stored-only for t/default, display for T and ignored empty action; match explicit/auto IDs, RGBA and cursor.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-34. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handleTransmit/_handleTransmitDisplay; test/KittyGraphics.test.ts

## 314: Kitty chunk continuations retain first-chunk identity

Split payload over two/three APCs with m=1 then m=0 and omit i on later chunks or all chunks. Expect byte-identical assembly, source ID selection, exactly one final response when appropriate, and no early display.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-35. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts put/end; test/KittyGraphics.test.ts

## 315: Kitty chunk size limits count the entire transmission

Send multiple individually-small chunks crossing kittySizeLimit and interleave IDs. Match source abort, release and pending-map state; later valid transfers must not reuse rejected bytes or corrupt other IDs.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-36. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _streamPayload/end; test/KittyGraphics.test.ts

## 316: Kitty query validates without storing or displaying

Use a=q with empty payload, valid PNG, invalid base64 and raw RGB lacking dimensions. Expect exact OK/EINVAL response framing and no image storage, tile/cursor change or image-added event.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-37. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handleQuery; test/KittyGraphics.test.ts

## 317: Kitty quiet flags suppress only the specified replies

Exercise q=0/1/2 for successful/failed transmit, display, query and placement, with and without i. q=1 suppresses OK only; q=2 suppresses errors too; no-ID rules and exact APC reply bytes match source.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-38. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _sendResponse; test/KittyGraphics.test.ts

## 318: Kitty rejects conflicting image id and image number

Send both i and I with/without payload and quiet=0/2. Expect source EINVAL text and no storage/display mutation, including no-payload conflict path.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-39. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts put/_handleNoPayloadCommand; test/KittyGraphics.test.ts

## 319: Kitty rejects file and shared-memory transmission

Exercise t=f/s/t/d and omitted t in query, transmit and transmit+display with/without i. Only direct transport follows the source success path; others yield correct EINVAL/silence and perform no filesystem access.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-40. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handleTransmit/_handleQuery; test/KittyGraphics.test.ts

## 320: Kitty delete selectors remove targeted stored images

Send a=d with d=i/I/a/A, absent id, unsupported selector and default selector. Match selected handler/storage removals and canvas pixel clearing; unrelated IDs and placements survive targeted deletion.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-41. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handleDelete; test/KittyGraphics.test.ts

## 321: Kitty deletion aborts only the matching pending uploads

Delete one ID or all during m=1 transfers, then send continuation chunks. Deleted bytes must never reassemble; targeted deletion preserves other pending IDs and matches source map/decoder cleanup.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-42. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handleDelete/_cleanupAllPending; test/KittyGraphics.test.ts

## 322: Kitty placement reuses image data and reports missing ids

Transmit-only, place twice with a=p, then place missing/absent IDs. Expect independent displayed placements with reusable stored data, exact OK/ENOENT and source no-ID silence.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-43. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handlePlacement; test/KittyGraphics.test.ts

## 323: Kitty placement geometry preserves crop and offsets

Place multicolor-200x100.png with c/r, x/y source crop, w/h source extent and X/Y subcell offsets including invalid/empty crop. Match source pixel rectangle, padding, scaling and error/cursor effects.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-44. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _decodeAndDisplay; test/KittyGraphics.test.ts

## 324: Kitty placement cursor movement honors C=1

Display at bottom/right boundaries with C omitted, C=0 and C=1. Default advances and scrolls exactly as source; C=1 preserves cursor while displaying the same image tiles.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-45. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _decodeAndDisplay; source_repo/addons/addon-image/src/kitty/KittyImageStorage.ts; test/KittyGraphics.test.ts

## 325: Kitty raw RGB RGBA and compression decode faithfully

Transmit f=24/32/100 with exact/short pixel bytes, missing dimensions, valid deflate/deflate-raw and invalid compression. Match decoded RGBA, alpha and precise source error/silence, with no rejected partial placement.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-46. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _createBitmap/_decompressZlib; test/KittyGraphics.test.ts

## 326: Kitty placement layers and ids preserve response routing

Place same image with positive/zero/negative z and distinct p. Match foreground/background draw order and response p field for successful/missing placement; clearing one image removes all source-linked displays.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-47. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts _handlePlacement/_sendResponse; test/KittyGraphics.test.ts

## 327: Kitty control overflow reset and abort are recoverable

Feed control data at 512/513 units, aborted APC, reset/dispose during pending uploads, then valid command. Match source partial-state and release behavior, with no old payload emitted as text or joined into a new image.

Verification: Required: node scripts/acceptance-oracle.mjs --case img-48. Replay cited JS assertions and Rust-backed WASM; compare pixels, buffer, cursor and replies in applicable Chromium/Firefox/WebKit; fail on mismatch or zero tests.

Source: source_repo/addons/addon-image/src/kitty/KittyGraphicsHandler.ts put/reset/dispose; test/KittyGraphics.test.ts
