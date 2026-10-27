# Browser links and scheduling acceptance

This is the task 35 acceptance contract. Passing portable tests does not satisfy browser integration.

## OSC 8 discovery and wrapped ranges

Reference `source_repo/src/browser/OscLinkProvider.ts` and its complete original test suite.

- Missing buffer row invokes the callback once with undefined. Existing empty row returns an empty array. Row coordinates are one-based.
- Skip contentless cells before a run. A link requires extended attributes and a nonzero URL id. A trailing null cell inside an active run still participates. Preserve the source loop's early continue when a link starts at trimmed EOL, including the single-cell-at-EOL behavior.
- A URL-id change finishes the old run and starts the next run on the same cell. A non-link cell finishes a run without including that cell. A same-id final cell is included. Missing or empty URI suppresses only that run.
- Upward expansion requires column zero, current line wrapped, an existing previous line, and the same id at its trimmed final cell. Scan backward through only contiguous same-id cells. Stop at the first different id or nonwrapped boundary.
- Downward expansion requires end equal to trimmed EOL, next line wrapped, and same id at next column zero. Include only the contiguous prefix. Cover multiple wrapped lines, scrollback start/end, empty continuation, different ids and mid-line boundaries.
- Default protocol guard uses WHATWG URL parsing and accepts only http/https. Reject malformed URLs and other protocols. allowNonHttpProtocols bypasses parsing, including malformed strings, exactly as upstream. Do not introduce a separate approximate parser.
- Custom activation receives the originating mouse event, full URI and expanded range. Hover/leave optional callbacks receive the same values. Default activation prompts with the exact upstream warning; cancel does not open a window; accept clears opener before assigning location; blocked popup warns. Test in a real browser.

## Provider registration and asynchronous priority

Reference `source_repo/src/browser/services/LinkProviderService.ts` and `Linkifier.ts`.

- Registration appends in order. Disposing a registration removes the first matching provider identity, not merely an equal provider. Duplicate registration of the same object and repeated handle disposal follow the source's indexOf/splice behavior. Service disposal empties the list. Drop must release retained providers without an Rc cycle.
- Provider reply states distinguish not-yet-replied, undefined and empty array. Empty arrays are truthy in JavaScript and delay lower priority selection until all providers finish. A defined higher provider reply without a matching link still blocks early lower provider selection.
- Undefined higher replies permit a lower matching reply early. When every provider replies and no link was selected, choose the first matching result in provider order, not callback completion order.
- Remove overlapping lower priority links using inclusive occupied cells. Preserve the source's partially occupied cells when a rejected link first overlaps partway through its range. Wrapped ranges use columns zero through cols on intermediate rows.
- Cache is per row. Reuse cached replies within a row; do not ask unresolved providers again. Starting a new row disposes prior cached links. Mouse-out replies are ignored. Preserve and test source behavior for late replies after line changes; do not silently impose a different cancellation contract.

## Link hover, activation and decorations

- Position conversion uses mouse coordinates plus ydisp. Invalid positions do nothing. Ignore hover-widget paths before reaching the terminal root. Same-cell mousemoves do not recompute links.
- Leaving the element clears hover and current link. Resize clears it and forces new provider lookup on the next cell change. Row changes and moving out of the current range clear it before lookup.
- Mousedown records current link. Mouseup activates only with a current link, valid position inside it, and a mousedown link with equal text and all four range coordinates. A replacement object with equal content qualifies; unequal URI/range does not.
- Hover defaults underline/pointer to true only when decorations are absent. Supplied partial decoration objects preserve absent fields as falsey. Dynamic pointer/underline updates affect DOM/events only while hovered.
- Show/hide underline uses x1=start.x-1, y1=start.y-ydisp-1, x2=end.x, y2=end.y-ydisp-1, current cols. Run all four original Linkifier assertions and add nonzero scroll offset and wrapped rows above the viewport.
- Rendered viewport changes clear/requery only when the entire current range is within the affected buffer rows. Start row zero also clears links above the fold. Dispose removes listeners and cached references; no callback after disposal can mutate detached DOM.

## Animation-frame render debouncing

Reference `source_repo/src/browser/RenderDebouncer.ts`.

- Requests combine minimum start and maximum end, using latest rowCount. Omitted bounds mean whole viewport. Clamp only on frame execution. A pending frame is not requested again.
- Callback-only requests schedule a frame and execute without a render range. Refresh callbacks receive zero, in registration order, after render. Document and test callbacks registered reentrantly during render and callback iteration.
- Clear the scheduled frame and row bounds before invoking render so reentrant refresh creates a new frame and does not lose new rows. Dispose cancels the pending browser frame. Test repeated disposal and subsequent calls against the source.

## Time-based debouncing and DOM integration

Reference `source_repo/src/browser/TimeBasedDebouncer.ts` and `Dom.ts`.

- First refresh before the threshold waits for the remaining interval since zero. At or beyond threshold, render immediately. Throttled requests share one trailing timeout and aggregate bounds. Latest rowCount clamps the aggregate.
- Immediate refresh cancels pending trailing timeout. Trailing execution records actual current time. Reset row bounds before callback. Test callback reentrancy, boundary equality, repeated disposal and refresh after disposal without sleeps using injected virtual time.
- Window selection prefers node.ownerDocument.defaultView, then event.view, then global window. Page coordinates add that window's scroll offsets.
- Disposable listeners preserve capture/options and remove the same callback identity exactly once. Animation queue is per-window, descending priority with stable equal-priority order, canceled items skipped, new items deferred to the next frame. Callback exceptions are reported and do not stop remaining work.
- Browser integration must exercise actual Rust/WASM links and scheduling in Chromium, Firefox and WebKit, including scrolling, hover, activation, provider disposal and animation cancellation. Native unit tests and cross-compilation alone do not satisfy this section.

## Evidence

Record source hashes, complete original assertion outcomes, Rust assertion counts, deterministic/random seeds and retained failing cases under `results/links/`. Commands and exact gaps go in `docs/links.md`. No source assertion edits, ignored tests, fake URL validation or successful DOM no-ops.
