# Search acceptance checks

These checks cover every branch in the supplied search addon source. A checked implementation is not evidence of an executed assertion. Record mapped/executed counts separately in `docs/search.md` and raw results. No search criterion can be closed by testing a JavaScript terminal instead of Rust.

## A. Forward and reverse engine paths

Source: `SearchEngine.ts` and `SearchEngine.test.ts`.

- Empty term returns no result; addon clears selection. An absent match returns no result. Default options, absent options and individually undefined booleans use upstream truthiness.
- Explicit `find` throws for start column equal to or greater than terminal width. Negative row/column starts, missing lines, starts beyond the last row and an empty buffer do not panic.
- Forward search tests first row at the supplied column, later rows from column zero, and stops at `baseY + rows`. It does not wrap in plain `find`.
- `findNextWithSelection` starts at zero without selection. It starts at selection end for an unchanged cached term and selection start for a changed term. It scans later rows, wraps earlier rows and finally retries the previous selection row from zero. A single match is returned again.
- `findPreviousWithSelection` starts at the last row and terminal width without selection. With a changed term it attempts rightward expansion from selection start before reverse search from selection end. With an unchanged term it searches before selection start. It scans earlier rows and then wraps from the bottom.
- Forward search starting on a wrapped row backs up to the first row and accumulates columns. Reverse search skips wrapped continuation rows while accumulating columns. Long runs must not use recursive stack growth.
- Literal search covers case-sensitive/insensitive matching, match before/at/after the offset, overlapping candidates in reverse, original requested term in result and no accidental Unicode case-fold matching.
- Whole-word boundaries use the exact punctuation/space constant from upstream, not Rust word boundaries. Test beginning/end of line, every delimiter, underscore, tab, non-ASCII neighbors and rejected first candidate even if a later candidate would satisfy whole-word rules.
- Regex search covers valid/invalid patterns, `g` versus `gi`, captures/backreferences/lookahead/lookbehind, anchors relative to the sliced search text, greedy/non-greedy matching, overlapping reverse candidates, empty forward match rejection, empty reverse candidates, astral pairs and lone surrogate matching without the Unicode flag. Invalid patterns propagate an error, not an empty result.
- Match output has exact UTF-16 term units, buffer row/column and cell span. Test narrow cells, combining strings, astral characters, CJK width-two cells and zero-width followers, null cells as spaces, multi-row matches and a match ending exactly at a row boundary.

## B. Wrapped line translation and cache lifecycle

Source: `SearchLineCache.ts` and `SearchLineCache.test.ts`.

- Cache begins absent. A set before initialization has no effect. Initialization creates an empty cache; repeated initialization preserves entries and refreshes access time. Get of an unset/beyond-buffer row yields absence. Set replaces an existing entry.
- Cache entries retain arbitrary UTF-16 text and line-start offsets. Out-of-range terminal lines translate to empty text with offsets `[0]`.
- A nonwrapped line honors trim-right. Only the final row of a wrapped line is trimmed. Full-width rows, content with trailing spaces, null cells, CJK, astral and combined cells translate exactly like the source buffer API.
- For a null width-one last cell immediately before a wrapped width-two first cell, remove exactly the artificial blank and adjust later UTF-16 line offsets. Do not remove an actual final character or a null before a narrow cell.
- TTL is exactly 15,000 milliseconds since the most recent init call. Check 14,999, 15,000, refreshed deadline and repeated expiry. Test clocks explicitly, without sleeps.
- Line feed, cursor movement and resize each invalidate all entries; disposal clears timers/subscriptions and cache and is idempotent. Portable cache invalidation tests alone do not cover terminal event wiring.

## C. Search state and result tracking

Sources: `SearchState.ts`, `SearchResultTracker.ts`.

- Cached term and last options distinguish missing from empty/present values. Valid terms are nonempty, including whitespace-only and surrogate-only terms.
- Missing old options means changed. Missing new options with existing old options means unchanged. Compare caseSensitive, regex and wholeWord by strict optional boolean equality, distinguishing absent from false. Incremental and decorations do not affect that comparison.
- Highlighting updates only with decorations present and missing/changed cached term or changed search options. Clearing cached term leaves options intact. Reset clears both.
- Result tracking truncates to highlight limit in insertion order. Index equality uses row/column/size and ignores term. Clearing selected decoration disposes it once. Result events fire only when decorations are requested, with index `-1` when absent/unlisted and exact result count. Reset clears results and selected resources.

## D. Addon, decorations and browser integration

Sources: `SearchAddon.ts`, `DecorationManager.ts`, `DecorationManager.test.ts`, addon typings and all addon integration suites.

- Constructor default highlight limit is 1000; custom limits bound results and decorations. Search before activation errors. Activation installs write/resize listeners and independent cache invalidation listeners. Disposal cancels delayed work, listeners, active/inactive decorations and markers.
- Before/after search events and result-change events preserve source ordering. Search stores the latest options and cached term in the source order, including the source behavior when options change on an unchanged term.
- Highlight iteration moves by result cell size, handles row carry, avoids repeated results and respects limit. Empty terms clear selection/decorations/results. `clearDecorations(true)` retains cached term; the default does not. `clearActiveDecoration` leaves inactive matches intact.
- Successful search selects the exact cell span, scrolls only if outside the viewport and centers using floor(rows/2). `noScroll` preserves viewport. Missing result clears selection and selected decoration.
- Write/resize debounces refresh by 200 milliseconds, replaces a pending refresh, performs incremental previous search with no scrolling and cannot run after disposal.
- Decorations split a wrapped match into per-row ranges, compute marker offset from baseY/cursorY, use active top/inactive bottom layer and matching background/border/ruler options. Ruler highlights are deduplicated by row. Marker and decoration disposal clear each other without double disposal. Partial creation failures preserve source cleanup semantics.
- Render callbacks apply the two source CSS classes and optional outline exactly. Confirm actual Rust-backed browser output and selection on Chromium, Firefox and WebKit, not a recorded decoration command list alone.

## E. Evidence

- Run all original engine and cache assertions unchanged against the original source baseline. Record exact counts and source hashes. Port each assertion or mark it unrun, not silently omitted.
- Compare Rust and original engine/cache/state outputs over deterministic cases plus seeded randomized actual-cell rows, malformed UTF-16, wrapped rows, null/wide/combined cells, regex/literal options, selections and starts. Save seeds, iteration counts and fixtures. Retain failing inputs.
- Run targeted Rust tests and Clippy on the declared MSRV, then workspace fmt/Clippy/tests. Browser/terminal integration failures remain open under D even if A/B/C pass.
- Obtain independent review of code and executed evidence. Keep `equivalence NOT PROVEN` until the full addon and applicable platform/browser suites pass.
