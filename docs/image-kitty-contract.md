# Kitty graphics native contract

Task124 owns `crates/xterm-addons/src/image/kitty/` and the strict shared payload decoder `image/base64.rs`. Main image task38 wires the actual terminal and browser host. User criteria312-327 and IM4 in `docs/image-acceptance.md` apply. Component tests do not satisfy the live img-33 through img-48 acceptance commands.

## Implemented command and decoder interfaces

`command::parse_command(&[u16]) -> KittyCommand` retains every recognized control field. Strings are optional UTF-16 vectors; numbers are optional binary64 values. Absent, empty, NaN and negative zero stay distinct. Unknown keys are ignored, first equals splits each comma segment, and later recognized duplicates replace earlier values. The parser uses one validated decimal conversion rather than per-digit floating rounding.

`image::base64::PayloadDecoder` has:

- `new(keep_size, max_encoded_bytes, initial_encoded_bytes) -> Result<Self, InvalidSettings>`.
- `init(Option<usize>, Option<usize>) -> Result<(), InvalidSettings>`, with `None,None` resetting the current decoder configuration.
- `put(&[u32])` and `end()` returning `Result<DecodeStatus, TryReserveError>`.
- `DecodeStatus::{Ok, DecodeError, NotInitialized, SizeExceeded}` and source numeric status codes0,-1,-2,-3.
- `data() -> &[u8]`, `loaded_bytes()`, `free_bytes()` and `release()`.

The strict decoder intentionally differs from the forgiving clipboard/name helper. It truncates incoming codepoints to low8bits like Uint8Array.set, delays validation until131072 pending units, retains the last quartet for final padding, preserves in-place partial/error state, and rejects invalid alphabet rather than dropping whitespace. Allocation failures remain typed errors. Physical pages are not eagerly touched just to imitate WASM reservation. Settings are bounded by the original 0xffff0000 maximum. A zero-initial-size growth loop that hangs upstream is made terminating; image handlers reject nonempty zero-limit payload before reaching that third-party helper defect.

The command component has source evidence: all23 original test bodies pass, and native complete-field snapshots match1411 inputs, including1024 seeded traces. The strict decoder's three standalone native Rust tests pass270 source-generated traces with7955 operations against the actual original WASM dependency. Every byte/status/loaded/free observation is compared, including131072 threshold and repeated end/release cases. Controlled compilation and replay receipts are under results/build-slots/1793150050105-48816-099e8bfd-2a1c-458a-954f-7dcf32d83166 and1793150050463-38648-cd2e40dd-b6fd-4133-b2b3-a88d3c746234. This is native Windows component evidence, not an addon/Cargo/browser parity claim. Invalid raw JS numeric decoder settings retain a separate integration gap under task205.

## Kitty source-byte storage

`KittyImageData` owns immutable shared `image::encoded::EncodedBytes`, raw binary64 id/width/height/format, and UTF-16 compression. Its memory backend retains an owned vector without exposing mutable access, avoiding the second full-payload copy required by Rc-slice conversion. Browser hosts supply an actual Blob-backed EncodedSource outside WASM linear memory. EncodedSourceFactory.capture_owned receives copied owned bytes synchronously before decoder release, returns a retained source handle, and reports host errors without pretending success. Source reads return owned asynchronous results and MIME metadata. No decoder/map borrow may cross a host factory or await. It is not a rendered surface or a successful display result. Copying decoder bytes must precede decoder release.

`KittyImageStorage` wraps a `Rc<dyn KittyPlacementStorage>` host. The host supplies actual `ImageResource` insertion/deletion plus assignable deletion callbacks with shared identity. It uses the shared task126 `AddImageOptions`, `ImageResource` and `StorageError` types. The actual host adapter calls `ImageStorage`; a recording host is permitted only for unit verification of native order/state.

Storage maps preserve JS SameValueZero number keys, insertion order and live mutation during deletion. Images and placements expose retained live read-only views with shared identity. Their iterators observe appends/clears until exhaustion; explicitly named snapshot methods return owned entries. A private ordered map combines a numeric-key lookup with insertion ordinals because ordinary HashMap traversal cannot preserve source eviction order or callback-driven appends. No external dependency is needed. Ordinals do not reset on clear, so an active live traversal observes newly inserted entries after a callback clears the map.

`store_image` deletes an old displayed placement before replacing its source-byte record. `add_image` removes only the stale reverse mapping before actual host insertion, then commits forward/reverse mappings afterward. The source supports only one current storage mapping per Kitty id; older displayed placements may remain visible without reverse ownership. Do not silently implement a fuller Kitty specification and call it source parity.

The wrapped deletion callback invokes the previous callback first, then updates Kitty maps. Errors stop at the source's partial state. Disposal resets Kitty maps and restores the previous callback only if the host still contains the exact wrapper identity. Weak map captures avoid callback ownership cycles. No state/map borrow may span host methods or callbacks.

## Display and protocol integration

The native transmission/controller will retain pending decoder and first-chunk metadata by explicit id, most-recent pending id or zero, exactly as source. Success/abort, control512 overflow, cumulative encoded limits, i/I conflicts, quiet and response routing remain native state decisions.

Display is an explicit asynchronous host operation, not a successful no-op. A request carries retained source bytes and the complete command. Source responses for placement/transmit-display occur only after real host completion, and rejected decode/render work maps to the source's false/response policy. Response listener failures must propagate rather than becoming ordinary false bubbling.

The real host performs platform image decode and resource operations; native pixel and placement helpers preserve raw RGB/RGBA conversion, crop/offset/scale decisions and numeric limits. Shared storage owns tile/marker/cursor effects and the browser renderer owns canvas/bitmap lifetime. Native protocol logic must never call the original JavaScript image handler.
