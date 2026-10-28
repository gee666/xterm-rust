# Image storage progress and remaining proof

Task 126 is in progress, not complete. Chloe owns `crates/xterm-addons/src/image/storage/` and the native storage cases/support. All source methods now have native bodies, actual BufferService/Marker/cell payload access, a real headless-terminal adapter, opaque platform resource ownership and fallible drawing/resource interfaces. Source `ImageStorage.ts` was read completely.

Implemented but not yet accepted:

- Source limits, constructor fallback diagnostics, original/resized pixel accounting and numeric-ID FIFO eviction.
- Ordered SameValueZero Map shared with Kitty, preserving insertion order under nested linefeed placement and live append/clear iteration without a duplicate container.
- Direct real InputHandler linefeed/dirty hooks, clipped tile writes, VT340/IIP/non-scrolling cursor policies, isolated payload reuse and old-image tile counts.
- Marker-disposal deletion, alternate retally/wipe, reset, single bitmap close and assignable callback identity. Native callback/resource drops release storage borrows first.
- Layer insertion/removal, early return, full-clear flags, inclusive row scan, consecutive tiles, placeholders, stable z-order and resource/tile retrieval.
- Source width-growth content-loop quirk and a signed/cyclic number getter boundary.

Nineteen directed native cases have been authored in `tests/image_storage.rs`, `tests/image_storage_cases/` and `tests/image_storage_support/`. They use the actual Rust headless terminal, BufferService, BufferLine and Marker, while recording platform drawing calls. They have not yet executed. A recording drawing host is component evidence only, not browser pixels or a complete image addon.

The first bounded addon check stopped on unrelated core mode-callback type errors. A later queued check was rejected by the 1 GiB guard before a compiler started. Receipts remain in `results/image-storage/` and `results/build-slots/1793150280656-36248-5a0b7755-d212-4044-a432-ec7aafdb3437/`. The reviewed single-target relocation is now complete, allowing new controlled checks through `scripts/build-slot.mjs`.

Remaining work:

1. Compile the current cohesive storage/package and fix real type/runtime findings. Run all 19 native cases, source-comparison traces, rustfmt and warnings-denied scoped Clippy. Resource getters, metrics and close classification must propagate original host errors.
2. Build an unchanged-reference ImageStorage oracle using real original headless buffers with a recording platform host. Compare cursor, cells/attributes/payloads, resource state, insertion order, budgets, marker/deletion/callback order, render commands and post-error state. Retain source hashes, seeds, exact counts and failing prefixes.
3. Integrate the actual protocol hosts from tasks 38/124/125 and actual browser renderer from 127. Preserve asynchronous completion after reset/dispose and temporary terminal ownership in the parent host.
4. Execute the applicable native and three-browser acceptance cases. No renderer, image protocol or platform is considered verified from helper tests alone.

Overall image equivalence remains NOT PROVEN. No resource limitation is being used to defer missing implementation as an external blocker.
