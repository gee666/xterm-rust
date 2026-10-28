# Image activation wiring notes

Task38 owns the real native/WASM addon coordinator. These are source inspection notes from `ImageAddon.ts`, not executed integration evidence. Native policy224, IIP156, storage126 and renderer127 are separate components until this wiring is tested.

## Constructor and options

The source clones runtime options and constructor defaults separately with Object.assign. The twelve declared fields are copied; explicit undefined is not the same as a missing property. Raw JS option coercion and decoder constructor-versus-init failures remain task205. Rust native ImageOptions currently models valid bool/f64 configuration, not arbitrary JS objects.

Reset changes only sixelScrolling and sixelPaletteLimit to constructor defaults. It clears current storage before resetting current protocol handlers in insertion order, and returns false. Storage limit and placeholder choices survive. The palette set-default query reports before resetting handlers. A report callback may reenter and change options or handler membership. Execute ordered policy effects with no policy/map borrow held across callbacks; re-read live options/membership when processing ResetHandlers.

## Activation order

1. Assign current terminal.
2. Create/assign renderer and shared storage. Storage's added callback forwards to the existing addon emitter.
3. If enabled, mutate existing windowOptions and enable exactly the three source size report flags. Do not restore them on disposal.
4. Register the always-present CSI DECSET/DECRST/DA1/graphics handlers, render listener, DECSTR/RIS/reset hooks, buffer switch and resize callbacks in source order.
5. Register enabled SIXEL DCS q, IIP OSC1337 and Kitty APC G, with their protocol-specific storage wrappers. Kitty storage/handler disposal joins the same ordered list.

There is no source repeated-activation guard or automatic disposal of a previous activation. Do not add an already-active success/no-op or replacement cleanup without testing the source behavior. Disposables accumulate across activations; the protocol handler map replaces existing entries by key while retaining key insertion order.

Parent callbacks capture the addon, not an immutable activation snapshot. Consequently the parent CSI report path and render/resize/buffer callbacks read its current terminal/storage fields after repeated activation. Protocol handlers themselves capture the terminal, renderer and storage passed to their individual constructors. Old registrations can therefore still reach old protocol contexts while parent callbacks reach new current fields. Preserve the distinction in any lifecycle binding rather than closing every callback over the first terminal.

JavaScript argument evaluation also matters. The large _disposeLater call evaluates registrations before appending any of its arguments. If a host registration throws midway, earlier registrations may already exist while their disposal tokens have not entered the source list. Host failure tests must observe partial state; do not assert a stronger transactional cleanup contract merely because Rust RAII makes it convenient.

## Async lifetime and resources

IIP bitmap promises and Kitty display promises can finish after reset or addon disposal. Source has no generation cancellation guard. Retain the required terminal/storage/renderer context until completion, without introducing a permanent Rust/JS ownership cycle. ImageStorage has no disposed-state successful no-op. Its dispose resets existing entries but does not prohibit a later pending insertion.

Synchronous QOI decoding and ImageData construction precede the IIP fast path/promise. Those failures propagate. Bitmap promise rejection is caught and warned. Kitty display always crosses Blob.arrayBuffer asynchronously before its final response, even when dimensions are invalid. Pending completion and reply timing must be tested, not inferred from a consumed parser return.

Renderer placeholder promises also assign their bitmap after showPlaceholder(false) or disposal in the source. There is no source rejection catch or generation guard. Test controlled resolution/rejection and explicit close calls before choosing a native lifetime strategy. Native renderer127 uses a JS-owned lifecycle record with a weak native handle, keeping actual rendering math/routes in Rust and avoiding hidden Rc/JS cycles.

## Required evidence

User cases img01..48 require the actual Rust-backed WASM terminal, independent original execution and all applicable Chromium/Firefox/WebKit outcomes. Capture actual pixels, buffers, cursor, replies, events and properties. Losslessly compressed pixel artifacts may retain raw bytes while the receipt carries width/height/format and the uncompressed hash. Compare the full actual bytes before compression. No metadata request, fake bitmap or headless-only component can pass a live graphics case.

Real source failures must remain visible. Original image baseline is639 case-projects,627 passed,8 failed and4 skipped; Alaric linked exact identities and logs from post71. Do not weaken original pixel assertions to make either source or Rust green. Source fallback/font/renderer failures need diagnosis and retained evidence, not a completion claim.
