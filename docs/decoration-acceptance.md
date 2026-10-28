# Decoration service acceptance

Reference is `source_repo/src/common/services/DecorationService.ts` and its complete ten-case test suite. OSC link registration is a separate Bradley task. Browser rendering requires the Rust-backed browser integration and is not established by portable tests.

## D1. Registration, lookup and lifecycle

Reject disposed markers without events. Keep options mutable by shared identity, default overview-ruler position to full when absent, and retain marker identity. Insert decorations in the shared SortedList so stable equal-key batch ordering and snapshot iteration match upstream. Register marker disposal before the service listener. Disposal fires its event before setting isDisposed, removes only the matching decoration once, updates its cache before the removal event, and unregisters its marker listener. Repeated disposal follows DisposableStore emitter semantics. Reset snapshots decorations, disposes each in order, then clears list/cache. Propagate listener failures without hiding partial state or keeping borrows across callbacks. Use weak captures to prevent ownership cycles.

Lookups use marker logical lines and each cached height span. Defaults are x zero, width one, height one and bottom layer. The left boundary is inclusive, the right boundary exclusive. Zero width/height produce no hits. Both iterator and callback variants preserve bucket order and optional layer filtering. Callback traversal must observe live bucket mutation like JavaScript array iteration, while service decorations uses a snapshot. Do not use a stale marker line when removing a disposed decoration.

## D2. Deferred line cache

Cache add retains insertion order and indexes each spanned line. Missing buckets return absence. Removal uses indexedStartLine and current options height, not the marker's disposed line. Attach replaces old trim/insert/delete subscriptions without clearing cached decorations. Clear removes pending sync actions and indexed data, canceling scheduled work.

Trim is immediate. Nonpositive trim or empty cache does nothing. Shift logical keys by amount, omit negative keys, merge bucket order and decrement indexed starts only for live markers. Insert/delete queue actions until a host microtask boundary, allowing marker listeners to update first. Queue every event in order but request only one pending microtask. Reset pending state before callbacks so reentrant requests schedule another turn. Inserts individually remove/reindex spans crossing the insertion point, shift remaining keys and sync shifted indexed starts from live markers. Deletes drop removed keys, shift lower keys, sync surviving starts, and reindex only spans crossing the entire deleted range. Preserve partial overlaps and disposed-marker branches exactly.

## D3. Colors, render contract and integration

Foreground/background colors are lazy tri-state caches. An absent/empty color caches absence. A failed CSS parse propagates and leaves the cache unset so a later access retries. A successful parse stays cached even if options change later. Browser adapters supply the canvas fallback to the existing shared CSS parser. Render notifications carry an opaque host element handle, never a browser dependency in core. A decoration retains its current host element and publishes render events. Buffer activation attaches to the new active CircularList. Actual buffer trim/splice tests must demonstrate marker/cache cooperation, and browser adapters must schedule deferred sync with a real microtask and connect registration/removal/render lifecycle.

## D4. Evidence

Execute all ten original tests unchanged, including the asynchronous insertion case. Retain source hashes, every original result and exact discovered/executed/passed/failed/skipped/unrun counts. Port every expected assertion to Rust and add deterministic layer, event ordering, lazy-color, marker disposal, reset, reentrant traversal and insert/delete/trim edge cases. Compare deterministic seeded operation traces against actual JavaScript with retained seed and full ordered lookup state. Keep failures, run focused debug/release/fmt/Clippy, and obtain independent source/code review. Overall terminal equivalence remains NOT PROVEN until separate integration and platform gates pass.

Rust geometry uses signed logical line coordinates and unsigned integer heights. Browser-facing numeric conversions must validate this boundary rather than silently truncate invalid values. Native hosts explicitly flush queued microtasks; browser hosts subscribe to sync requests and schedule the same flush through queueMicrotask. These are scheduling adapters, not permission to run the cache actions synchronously during line events.
