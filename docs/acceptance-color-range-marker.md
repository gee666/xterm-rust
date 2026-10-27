# X11 colors, ranges and markers acceptance

These checks replace vague utility-level claims with explicit source paths and failure cases. They do not cover CSS Color.ts, terminal OSC dispatch, buffer reflow, browser bindings or whole-terminal equivalence. Those paths need their own criteria and integration evidence.

## X11 color parsing

Reference `src/common/input/XParseColor.ts`, function `parseColor`.

1. Accept `rgb:` in every ASCII letter case. Require exactly three slash-separated channels, all the same width, with one, two, three or four ASCII hexadecimal digits. Accept uppercase/lowercase hex and leading zeroes. Reject empty components and extra separators.
2. Scale `rgb:` channel values using nearest-integer `value / max * 255`, with maxima 15, 255, 4095 and 65535. Check zero, maximum, channel-specific values, values adjacent to rounding transitions, and every possible channel value at each width against JavaScript. Channels must not bleed into each other.
3. Accept hash syntax only at 3, 6, 9 or 12 digits. One-digit channels shift left four bits, two-digit channels stay unchanged, three/four-digit channels truncate the low four/eight bits. Explicitly require `#fff -> [240,240,240]` and `rgb:f/f/f -> [255,255,255]`.
4. Reject absent prefixes, named colors, `rgbi:`, signs, decimal points, underscores, embedded or surrounding whitespace, trailing line terminators, full-width/non-ASCII digits, supplementary characters and channel widths above four. Reject leading junk, unlike the separate unanchored CSS parser.
5. Empty input produces `None`, the mapping of JavaScript `undefined`. Parsing any UTF-8 input must not panic. Accepted syntax is ASCII, so Rust UTF-8 versus JS UTF-16 indexing cannot change an accepted result.

Evidence must include all four original parse cases and their assertions. Deterministic differential coverage must identify its seed, counts and source/workload/output hashes. Retain failures with the exact input and both results. Exhaustive single-channel coverage is not an exhaustive cross-product of triples and must not be described that way.

## X11 color formatting

Reference `toRgbString` and private `pad` in the same source.

1. `None` depth corresponds to omitted JavaScript argument and selects 16 bits. Depths 4, 8 and 12 select their exact output width. Every other representable depth selects 16 bits, including zero and `u32::MAX`.
2. Eight-bit formatting uses two lowercase digits, including low zero. Twelve-bit formatting repeats the two-digit byte and takes three characters. Sixteen-bit formatting repeats it twice.
3. Four-bit formatting uses the first digit of the *unpadded* hexadecimal byte. In particular, bytes 0, 1 and 15 produce `0`, `1` and `f`. It is not simply a shift-right-four operation.
4. Preserve the three original formatter cases and all assertions, including default depth, round trips and reduction of 12/16-bit input to 8-bit channel resolution. Compare all 256 byte values in mixed triples at each chosen depth.
5. Rust channels are `u8`, not arbitrary JavaScript numbers. Document this boundary. Future bindings must map negative/fractional/non-finite depths to the same 16-bit fallback and explicitly define behavior for out-of-range channel calls. The current core module must not claim those bindings work.

## Inclusive buffer ranges

Reference `src/common/buffer/BufferRange.ts` and all four original tests.

1. Return `cols * (end.y - start.y) + (end.x - start.x + 1)` with JavaScript binary64 operation order. Test equal-row singleton and multi-column ranges, next-row ranges, multiple rows, zero columns and out-of-width columns.
2. Reject only `start.y > end.y`, with the exact source error text and both coordinates. Do not add hidden x-order, positivity or bounds checks.
3. Preserve zero and negative results for equal-row reversed columns. Preserve negative coordinates and negative column counts within the typed i32 domain. Test i32 extrema without intermediate signed overflow and compare their binary64 results to JavaScript.
4. Document that fractional, non-finite and beyond-i32 JavaScript coordinates are not represented by the typed core API. Browser/API bindings need an explicit conversion policy, not an unchecked cast.
5. Port all four upstream cases unchanged in expected values. Add deterministic differential valid/reversed rows, negative coordinates and boundaries. Save the seed and discovered/executed/pass/fail/unrun counts.

## Markers

Reference `src/common/buffer/Marker.ts`. There is no standalone Marker.test.ts in the supplied reference; locate assertions in Buffer.test.ts and service consumers before claiming faithful source-test coverage.

1. Each newly created marker receives the next distinct positive id, independent of its line. Clones retain object identity, id, disposed state and mutable line. Separate markers at the same line remain distinct. Explain the allocator's scope, numeric limit and concurrency policy.
2. The line can move while live. Buffer trim/delete/reflow consumers must preserve identity while changing line or disposing the marker. A disposed marker's initial invalidation sets line to -1 before listeners run.
3. First disposal sets disposed state before callbacks, fires exactly one event, then disposes registered resources in insertion order and clears the resource vector. Listeners observe disposed=true and line=-1. Recursive and repeated disposal do not fire or dispose twice.
4. Listener/resource registration during disposal must match source traversal. Event listeners are traversed under Event.ts semantics. Resources added during the event participate in later resource cleanup. The original array disposal uses a live `for..of` traversal. Resources appended by resource callbacks must also be visited in that same disposal. Test this independently from event-listener snapshot semantics.
5. Register returns the same resource identity, retains duplicates, and does not eagerly dispose resources added after disposal. Preserve source behavior rather than replacing its array with DisposableStore's deduplicated closed-store policy.
6. If an event callback fails, propagate the failure and do not execute later listeners or registered resources. Disposed state and line invalidation remain visible. A second dispose is still idempotent and must not replay cleanup. Never swallow dispatch errors to get a successful result.
7. If a resource callback panics, do not hold a RefCell borrow across the callback. Preserve already-executed callbacks and state; document Rust unwind versus JavaScript throw behavior. Test nested operations without borrow panics or reference cycles.
8. Cleanup on Rust drop must be an explicit documented ownership choice. Do not silently call user callbacks from the last clone's drop without tests for reentrancy and error handling. Clone handles must not form permanent cycles through subscriptions or registered closures.
9. Integration evidence must attach markers to the real Rust Buffer implementation and exercise line deletion, trimming, scrollback, resizing and alternate-buffer lifetime. Unit-only marker results cannot mark that integration complete.

## Review and evidence gates

For each implemented utility, independently inspect source mapping and run focused Rust tests, differential/original baseline commands, rustfmt and Clippy with warnings denied. Record versions, exact commands, seeds, counts, failures and reviewer findings under results/. A green whole-workspace run cannot replace the named branch cases above. Linux/macOS execution and browser binding checks must be marked unrun until observed.
