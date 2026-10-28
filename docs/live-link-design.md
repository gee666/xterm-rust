# Proposed live-link representation

Task210 owns this migration. The current code is still the snapshot implementation. This document is a proposal for Cameron, Lane and Xavier to agree before changing callers.

## Keep browser objects live

A browser link handle should own the original JavaScript ILink object, not a Rust copy of its text/range. This avoids lossy back-copy schemes, preserves callback range identity, and allows range/text/callback getters to be read at the source's actual phases. Rust-created OSC links can create equivalent plain JS data objects at the ABI boundary, while Rust continues to implement OSC discovery, provider ordering, caching and hover/activation decisions.

The intended interface is an opaque cloneable `BrowserLink`, constructed from an original JS object, with fallible operations for current text/range reads, contains/equality tests, callback dispatch and disposal. A native Rust convenience constructor may build a plain data object with Rust callbacks. It must not freeze a user-supplied JS object or create a strong Rust/JS self-reference cycle.

`BrowserLinkifier.current_link` must become fallible or expose a fallible live-handle method used by all production consumers. Selection and mousedown matching must read live data. Do not retain an old Option-returning method that silently hides getter failures.

## Share the algorithm, not snapshots

Make the existing provider-reply priority/cache implementation generic over retained payload handles and fallible match/filter operations. Native differential tests can continue to use typed static `Link` values. The browser path supplies live-object reads. Do not copy the provider-priority algorithm into a second browser-only implementation.

Map and reply-array identities need shared ownership with short interior borrows. A callback can append/replace provider replies or replace the active cache. Disposal and overlap traversal must retain the old Map/Array identity where source local variables do, while later accesses to the active cache read the new field. No mutable RefCell borrow may span a user getter/callback. Reply insertion order remains distinct from provider priority.

Browser hover state must retain the source's raw decoration values and descriptor timing. JavaScript undefined is not interchangeable with false when the generated decorations getter is read. Install the source's dynamic decoration descriptors only after its hover callback phase, and use weak controller references in Rust callback closures.

## Binding boundaries

Reading property chains, calling JS functions with their original receiver, and creating plain descriptor/data objects are ABI glue. All provider priority, cache traversal, hover/activation state and terminal parsing remain Rust logic. Primitive JavaScript number coercion/operator helpers may be needed to preserve getter ordering and JS errors; do not delegate the terminal engine or the Linkifier algorithm to the original xterm.js implementation.

Native `ProvidedLink` from the web-links addon should carry enough per-link binding context to expose its already-created JS range object unchanged. Lane's current stable range cache is not by itself live hit-testing. The same object must be the one used by later Linkifier range reads.

Exact acceptance and traces are in `docs/live-link-acceptance.md`. The initial ordinary-range implementations/tests remain intact until migrated, and all observed passes remain tied to their original snapshots. No current equivalence claim is made.
