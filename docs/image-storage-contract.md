# Shared image storage contract

Chloe owns task126 under `crates/xterm-addons/src/image/storage/`. This is native terminal/tile/memory logic. Browser drawing stays in task127. No decoder or DOM dependency is added to the core. The source algorithm is `source_repo/addons/addon-image/src/ImageStorage.ts`, read completely; IM5 and user303-310 remain acceptance gates.

## Concrete interfaces being implemented

- `ImageResource = Rc<dyn ImageSurface>`. A surface exposes live unsigned pixel width/height, `as_any` for platform-owned access, and an optional close callback for ImageBitmap-like resources. Canvas/native pixel resources have no bitmap close operation. A provided pixel surface wraps the existing `RgbaImage`. Shared `Rc` identity distinguishes original and resized storage without duplicating pixel arrays.
- `ImageStorage::new(Rc<dyn StorageTerminal>, Rc<dyn StorageRenderer>, ImageStorageOptions)` returns a cloneable storage handle. Callbacks capture `WeakImageStorage`, not a strong handle stored in itself.
- `StorageTerminal` provides the actual `Rc<BufferService>`, fallible `mark_dirty(row)`, direct `line_feed()` using the real InputHandler behavior, `register_marker(offset)` using the public terminal's integer/buffer rules, and active normal/alternate type. It must not feed a synthetic LF string back through the parser.
- `ImageTileInfo` is an isolated mutable payload per cell, held through `ExtendedAttrs::payload` as `Rc<dyn Any>`. Old tile objects are reused on overwrite. Unrelated underline, hyperlink and packed flags remain intact.
- `ImageSpec` owns original/actual resource handles, original/current cell metrics, marker, mutable tile count, buffer type, layer and binary64 z-index. Renderer receives retained spec identity without a storage/map borrow held across platform calls.
- `StorageRenderer` provides cell metrics and exact layer/clear/rescale/draw/placeholder/extract/conversion operations, all fallible where the host can fail. Native tests record real calls; those tests do not claim browser pixel rendering. Task127 implements actual canvas/bitmap operations against this interface.

The concrete public error is `StorageError`, retaining buffer/marker event errors and opaque host errors. Storage operations return `Result` and preserve source partial mutations before an error. No errors are hidden behind an empty draw or successful no-op.

## Methods for protocol integration

`add_image(ImageResource, AddImageOptions) -> Result<u64, StorageError>` takes scrolling, layer, z-index and VT340/IIP cursor policy. `delete_image(id)`, `wipe_alternate()`, `reset()`, `set_limit(megabytes)`, `get_limit()`, `get_usage()`, `render(start,end)`, `viewport_resize(cols,rows)`, `get_image_at_buffer_cell(x,y)` and `extract_tile_at_buffer_cell(x,y)` follow source ordering and numeric rules. The original default cell size is 7x14 and invalid configured storage limits retain the source 2,500,000-pixel constructor fallback with explicit diagnostic routing.

Storage has assignable `on_image_added` and `on_image_deleted` callback slots, not an invented multicast ordering. Getter/setter access returns shared callback identity so Kitty can wrap and restore the deletion callback only while it still owns it. Added/deleted callbacks run after source state commits, without held map or terminal borrows.

Resource close, marker disposal and user callbacks may reenter storage. Live monotonic-ID iteration must observe newly appended entries during reset/eviction where the source Map iteration does. Removing an image precedes bitmap close and deleted callback. Storage clearing cannot silently reset source ID counters or full-clear flags which the source keeps.

In the supplied 6.0.0 source, public `registerMarker` is not proposed-API gated and returns a marker for either active buffer. Do not import older normal-only/proposed restrictions from stale comments. The optional host result still represents an absent marker supplied by a custom host.

The host interfaces are contracts, not completed integration evidence. Task126 must provide actual BufferService payload/marker/cursor tests and source differential traces. Task38 and127 must wire real native/browser addon and rendering entry points before image equivalence is claimed.
