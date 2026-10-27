# Selection, scrolling and accessibility acceptance

These checks define task 34's scope. A native model test does not establish browser behavior. Each row needs an implementation path, executable assertion, exact result and independent review before acceptance. Missing wiring or unrun browsers remain gaps. The upstream source and assertions stay unchanged.

## Selection coordinates, criterion 28

Reference `src/browser/selection/SelectionModel.ts` and all 18 cases in `SelectionModel.test.ts` under `source_repo`.

- Clear removes both endpoints, word length and select-all state.
- Reversal compares rows before columns; absent endpoints and equal endpoints are not reversed. Final start picks the earlier endpoint except select-all, which starts at buffer origin.
- Select-all end uses current columns and `ybase + rows - 1`, including after resize and scrollback changes.
- Missing start yields no end, even if an end exists. Missing or reversed end uses start plus minimum selection length.
- A minimum word/line length wins over a shorter end on the start row; a longer end wins. An end on another row retains its coordinates.
- Wrapped minimum ranges preserve modulo columns and quotient rows. Missing/reversed end at an exact multiple of columns ends at the previous row's right edge, without a trailing newline. The source's distinct same-row explicit-end branch retains its own modulo behavior even at an exact multiple.
- Trimming subtracts from both endpoint rows. An end above origin clears all state. A start above origin resets to `[0,0]`. Refresh is true only for clear/reset, including reversed endpoints.
- Compare all original assertions plus a deterministic JS/Rust state matrix over missing endpoints, reversed rows, exact-EOL lengths, multiple wraps, select-all, scrollback and trims. Include seed, case count, source hash and retained failure index. Coordinates and dimensions must have documented numeric boundaries rather than unchecked casts.

## Selection service, criterion 29

Reference `src/browser/services/SelectionService.ts` and its entire test suite.

- `selectionStart`, `selectionEnd`, `hasSelection`, `selectionText` and column mode use the final model endpoints. Empty selections are absent. Normal text joins wrapped lines, trims the appropriate right edge, preserves selected spaces and uses the source platform newline. Column text uses rectangular extents and correct newline handling.
- `clearSelection`, `selectAll`, `selectLines`, `select`, right-click word selection and cursor membership preserve row/column boundaries, redraw events and selection-change event ordering. Buffer activation clears selection; trim adjusts it; resize invalidates or redraws it as upstream does.
- Word lookup uses UTF-16 string offsets and buffer cell widths, configured separators, whitespace policy, combined/astral characters, wide-cell continuation and wrapped words. Port every upstream word-selection assertion, including punctuation and trailing blanks.
- Mouse down handles button, disabled selection, forced modifier, macOS option selection, shift-extension and single/double/triple click distinctions. Drag extends in both directions, preserves word/line minima and column mode, clamps above/below viewport and corrects wide-cell endpoints.
- Drag autoscroll uses the source speed bounds and interval contract, scrolls the terminal before updating ranges and stops on mouse-up or disposal. Mouse reporting must not accidentally enable selection.
- Linux primary-selection events, redraw requests and normal selection-change notifications fire on the original transitions, without duplicate callbacks after disposal.
- Rust-backed Chromium, Firefox and WebKit must execute text/column drag, wrapped/wide words, shift-click, select-all, trim during drag, copy, buffer switch and disposal cases. Browser-specific unsupported automation must be reported, not silently skipped.

## Viewport and scrollable system, criterion 30

Reference `src/browser/Viewport.ts` and every module in `src/browser/scrollable/`.

- Scroll state validates dimensions and clamps top/left to content bounds. Fractional values survive unless integer policy is enabled. Dimension-change events distinguish changes in size, content and position.
- Wheel normalization covers pixel/line/page units, horizontal/vertical deltas, sensitivity, fast modifiers and mouse protocol suppression. Touch translation has the source sign and clamp behavior.
- Smooth scrolling uses injected animation time/scheduler, interpolation, cancellation and animation reuse. No test relies on arbitrary sleeps. Disposal cancels callbacks and removes listeners/nodes.
- Scrollbar geometry preserves minimum slider size, arrows, visibility modes, opposite scrollbar space, track clicks, drag capture, hover/active classes and option updates. Cover zero content, equal content/viewport, oversized content and resize while dragging.
- Viewport converts rows to CSS pixels with current render dimensions. Scroll events round to rows as JavaScript does and request only the delta from `ydisp`. Dimension changes suppress scroll callbacks. Sync avoids reentrancy and duplicate scheduled callbacks.
- Buffer activation clears cached display row. External scroll stops animation; explicit smooth-scroll disable takes effect. Synchronized output defers DOM movement until render, then applies the latest position atomically.
- Colors, scrollbar width/arrows/visibility and sensitivity respond to option/theme changes. DOM structure and cleanup match upstream.
- Browser tests exercise normal/alternate buffers, scrolling at both boundaries, fractional device scale, resize, wheel/touch, synchronized output and post-disposal events in all supported engines.

## Accessibility and localized strings, criterion 31

Reference `src/browser/AccessibilityManager.ts` and `LocalizableStrings.ts`.

- Enabling before open returns the source error. Enabling screen reader mode attaches one accessibility container with a role=list tree, exactly one role=listitem per visible row, tabindex=-1 and an assertive live region. Disabling/disposal removes DOM and every subscription/listener.
- Rendered rows use buffer display offset, translated text and UTF-16-to-cell column map. Empty rows contain NBSP with `[0,1]` mapping. `aria-posinset` is one-based absolute buffer row and `aria-setsize` is full buffer length.
- Row height, container width/font size and scaleX alignment track render metrics, font options and DPR. Zero-height or missing columns follow source guards without divide-by-zero artifacts.
- Announcements consume queued typed characters one at a time; mismatches still announce output. Control-containing keys do not enter the echo queue. Key and blur clear the live region. Tab adds exactly the requested spaces; line feed adds newline.
- At newline 21 the localized overflow message replaces current live text. Further input stops until the source's subsequent announcement-clear transition. Pending announcements and echo queue retain the same clear behavior as upstream.
- Top/bottom boundary focus scrolls one row only when arriving from the adjacent row and more scrollback exists. DOM rows rotate, boundary listeners move, focus transfers correctly and default/propagation are prevented only on handled transitions.
- Accessibility DOM selection sorts anchor/focus, clamps to tree boundaries, maps UTF-16 offsets through columns, normalizes right-edge columns and calls terminal select with the correct cell count. Collapsed in-tree selection clears terminal selection; outside selection does not. Null or stale nodes are handled as upstream, and invalid ranges fail rather than returning success.
- Resize grows/shrinks rows and preserves active boundary listeners. Localization getters/setters retain exact defaults; subsequent announcements use current overflow text while the input prompt label follows the source construction-time contract.
- Browser DOM and accessibility assertions cover all these transitions, including wide/combined text, reverse selection, selection crossing outside nodes, output burst, keyboard echo, resize and disposal across Chromium/Firefox/WebKit.

## Reporting

Each subsystem must list discovered upstream assertions, mapped assertions, executed assertions, passes/failures/skips/unrun and additional oracle cases separately. Record commands, tool versions, platform, seeds and raw results. Do not sum randomized comparisons as ported upstream tests. Native Windows evidence is not Linux/macOS evidence. A component without terminal wiring is partial and cannot satisfy the browser criterion.
