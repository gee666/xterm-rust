# Buffer acceptance matrix

This checklist covers `common/buffer/Buffer.ts`, `BufferSet.ts`, `BufferReflow.ts`, `BufferRange.ts`, `Marker.ts`, and `common/services/BufferService.ts`. It does not stand in for parser, renderer, or addon acceptance. Overall equivalence remains NOT PROVEN until every item has executed evidence.

## A. Construction, cells and saved state

1. New normal buffer starts empty, capacity is rows plus scrollback capped at 4,294,967,295. An alternate buffer never gets scrollback, including after grow/shrink and option changes. `hasScrollback` also tests capacity against row count.
2. Initial cursor, base, display and saved positions are zero. Scroll region is 0 through rows minus 1. Saved attributes, charset, charset array, G level, origin mode and wraparound mode match source defaults.
3. Null and whitespace cells retain the source codepoint and width. Supplying attributes shares extended attribute state as upstream assignment does; omitting attributes resets foreground, background and extended attributes without changing content/width.
4. Blank lines use live host column dimensions, requested attributes and wrapped flag. Standalone `Buffer.resize` must not silently update its host service dimensions.
5. `fillViewportRows` fills exactly the configured rows only when empty. A partially filled buffer is not padded. Fill attributes apply to every cell, including extended and protected state.
6. `clear` resets positions, lines, scroll region and tabs. It does not silently reset saved state or erase markers differently from source. Disposal clears all markers and cancels pending cleanup work.
7. Missing-row translation returns an empty UTF-16 string. Present-row slicing preserves source column rather than scalar indexing: cut-off wide cells, zero-width placeholders, BMP, astral, combined and lone-surrogate data.
8. Wrapped-range scans include the first unwrapped physical row and all following wrapped rows. Test first, middle and last positions, both directions, one row, and a range reaching either buffer end.

Evidence must compare full cell content, width, foreground, background and extended attributes, not text alone. See source suites `Buffer.test.ts` constructor/fill/wrapped-range/translation cases and CellData/BufferLine integration tests.

## B. Row/column resize and viewport invariants

1. Grow columns before reflow; preserve existing cells and fill padding with default null attributes. Shrink columns after reflow only when source reflow is enabled.
2. Increasing rows on an empty/unfilled buffer must not initialize it. Increasing a filled viewport appends blank rows unless scrollback above the cursor can be revealed.
3. Revealing history decrements `ybase`; decrement `ydisp` only when positive. Test display at top, bottom, and between; blank lines below the cursor versus cursor on final occupied line.
4. A configured Windows PTY backend or build number makes row growth append rows rather than reveal history. Test either field independently, both, neither, build 0 and absent build.
5. Shrinking rows pops blank rows below the cursor first. When the cursor reaches the bottom, advance base/display instead. Test zero scrollback and nonzero scrollback.
6. Grow capacity before row adjustments. On capacity reduction trim from the top after adjustments; shift `ybase`, `ydisp` and saved Y toward zero before setting the new maximum.
7. Clamp X and saved X to the final column; clamp Y to final row and then apply the source growth adjustment. Reset scroll top only for initialized buffers; always reset scroll bottom.
8. After all reflow, `ybase + y` cannot exceed the final logical buffer end. Test shrinking rows and columns together with a cursor near the bottom.
9. Dimensions equal to current values preserve content and do not emit spurious line insert/delete/trim events. Source internal Buffer allows one-column ASCII resize; terminal service's public minimum of two columns must not erase that internal test case.
10. Reject unrepresentable Rust size/range inputs with typed errors before unchecked arithmetic. Retain source accepted inputs through adapter conversion; document any source error representation explicitly.

Every resize result checks physical line count/length/wrapped flags, full cells, cursor, saved positions, base/display, scroll region, marker states and exact ordered line events. Cross product includes rows grow/equal/shrink, columns grow/equal/shrink, buffer initialized/empty, normal/alternate, scrollback remaining/full/zero, user display tracking/nontracking and Windows PTY gates.

## C. Reflow and wide-character boundaries

1. Identical columns skip reflow. Alternate buffer skips reflow. Without a truthy Windows PTY build, normal-buffer reflow is enabled. With a build, only ConPTY at build 21,376 or later enables reflow. Explicit boundaries are 21,375 and 21,376, missing backend, WinPTY, build 0 and absent build.
2. `reflowCursorLine=false` leaves every wrapped group containing the absolute cursor untouched. `true` processes those groups. Test cursor on first/middle/last row of a group and outside it.
3. Larger reflow merges groups in source order, copies combined strings/extended attributes, preserves source line identities, clears trailing fragments and removes only trailing empty/unused wrapped rows.
4. A width-2 cell at the new final column moves to the next row; null its former location. Check odd/even old/new widths and adjacent wide cells. Distinguish a wide-cell padding null from ordinary tab-produced null space.
5. Larger layout deletion events fire in index-adjusted source order before layout application. Marker positions must match the same event stream. Correct removal counts are required for successive groups removing three or more rows.
6. Larger viewport adjustment decreases cursor Y at base 0, appends blank rows as required, otherwise decreases base and tracking display. Saved Y decreases with saturation at zero.
7. Smaller reflow calculates each destination length from full wrapped rows and the last row's trimmed length. It copies backwards to preserve overlapping source cells and retains combined data.
8. Smaller reflow inserts rows in batches, emits insert events in corrected forward indexes, then emits a trim event for overflow. Marker shift/disposal ordering must match.
9. Smaller viewport adjustment distinguishes not-yet-filled viewport, filled viewport with history capacity, and full history. Test `ybase=0` and nonzero, `ydisp=ybase` and detached display, plus every boundary in the original reflowLarger/reflowSmaller matrices.
10. A ten-cell line shrunk to 2 then 1 columns and expanded again preserves the retained suffix when scrollback overflows. Multiple adjacent wrapped groups restore independently on expansion.
11. Empty lines never become wrapped solely due to shrink. Preserve explicit `isWrapped` flags even when adjacent text becomes empty or a group starts at a trimmed row.
12. One-column wide-character reflow must not lock up. The public terminal minimum prevents this input; an internal helper must report a typed invalid-reflow error rather than spin. One-column ASCII source tests remain valid.

Evidence includes every original `BufferReflow.test.ts` assertion and full `Buffer.test.ts` reflow section, deterministic chunk/resize traces with retained seeds, shared row-identity assertions, and exact source/Rust cell/event snapshots.

## D. Markers and tab stops

1. IDs are unique and monotonic within the shared terminal realm. A marker's line is mutable; first dispose sets disposed=true and line=-1 before onDispose handlers. Event dispatch precedes registered resource disposal. Repeated dispose does nothing.
2. Trim subtracts line count and disposes markers crossing below zero. Insert shifts markers at or after its index. Delete disposes markers inside the deleted half-open range and shifts later markers.
3. onDispose removes the marker from the buffer collection without retaining a reference cycle. Test observer disposal, buffer clear-all, clear-one-line, callback reentrancy and buffer drop.
4. Tabs reset on full setup; incremental setup starts at the requested stop or previous stop. Tab width is read from current options. Preserve existing stops on incremental setup and rebuild on tabStopWidth changes.
5. Previous/next-stop searches default to cursor X, exclude the starting column and clamp to 0 through cols minus 1. Test exact stops, gaps, before/after width, and a one-column buffer. A zero/nonfinite tab width must be rejected rather than loop forever.

## E. Normal/alternate activation and scrolling service

1. Reset creates a filled normal buffer and an initially empty alternate buffer. It activates normal and emits exactly one activation event with active/inactive identities. Old buffers dispose markers and subscriptions.
2. Repeated activation of the current buffer is a no-op. Switching to alternate fills only if empty and copies cursor X/Y. Switching to normal copies alternate cursor, clears alternate markers/content and then emits activation.
3. Both buffers resize and tab stops are refreshed afterward. Option-change subscriptions resize for scrollback and rebuild tabs for tab width; disposal removes these subscriptions.
4. Service construction enforces minimum 2 columns/1 row. Resize updates live dimensions before buffer resizing; emits dimensions and independent changed flags even for unchanged dimensions, matching source.
5. Full-screen scrolling appends, recycles or inserts after a partial bottom region as appropriate. Scroll-top-nonzero shifts only its region and does not create history.
6. Cached blank lines refresh when width/foreground/background changes and preserve wrapped state. Verify extended-attribute behavior against source cache key, not an improved cache guess.
7. Appending before full capacity advances base and tracking display. At full capacity a user-scrolled display decrements only until zero. Non-user-scrolling display always ends at base.
8. `scrollLines` handles negative/positive/zero displacement, display clamp, transition into/out of user scrolling, suppression flag, unchanged display and exact emitted positions.
9. Buffer activation forwards a scroll event; service reset clears user-scrolling mode. Test event order with resize, activation and marker disposal callbacks.

## F. Memory, automation and review

1. Shrink preserves retained backing capacity until cleanup, while logical lengths change immediately. Schedule cleanup only when more than 10% of lines need it. Cancel/reset an earlier task on another resize or disposal.
2. Cleanup processes at most the source threshold of cleaned rows per batch, rescans once after a normal pass to catch shifted lines, and terminates. The host scheduler contract must be exercised, not replaced by a sleep in Rust tests.
3. Record discovered, mapped, executed, passed, failed, skipped and unrun counts per source suite. List every omitted assertion and reason. Memory-private-field assertions need a documented equivalent observation, not removal.
4. Keep original source/tests unchanged. Retain source hashes and failing deterministic traces. Run debug/release tests, rustfmt, warning-free Clippy and at least one independent implementation review.
5. Integration must run Buffer with actual Rust parser/InputHandler, normal/alternate service and addons. Component green tests do not prove terminal integration. Record native Windows/Linux/macOS execution separately; missing execution is not a pass.
