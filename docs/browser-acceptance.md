# Browser terminal integration acceptance

These checks describe required behavior, not completed behavior. Source reference is `source_repo/src/browser/CoreBrowserTerminal.ts`, `browser/public/Terminal.ts`, `browser/Types.ts` and browser integration tests. Task 31 owns integration and WASM binding. Tasks 28, 30 and 32-35 own engine, public wrapper, rendering, input, selection/accessibility and links respectively. Their completion is a dependency, not an external blocker.

Euphemia's `docs/public-api-acceptance.md` owns public-wrapper field/method/error enumeration, integer quirks and proposed-API gates. The checks below cover the browser host and coordination beyond that wrapper. Passing host component tests cannot satisfy a complete-terminal check.

## D1. Open, document choice and DOM creation

For each item, run against built Rust WASM in Chromium, Firefox and WebKit. Assert DOM properties, parentage and node identity, not screenshot alone. Compare equivalent source behavior.

1. `open` rejects a missing parent at the JS boundary with the source error. A disconnected HTML parent remains accepted; any source debug diagnostic remains a debug diagnostic, not a hard failure.
2. Select parent's ownerDocument unless documentOverride is an actual Document. Cover same-window parent, iframe parent, same-origin moved/adopted parent, valid override, and invalid override objects.
3. Create exactly one div.terminal.xterm with dir=ltr. Toggle allow-transparency initially and on option changes without replacing the root.
4. Preserve child structure and order: viewport then screen; helpers under screen; textarea then composition-view under helpers. Renderer adds its rows without stealing helpers.
5. Set textarea class, current global prompt label, tabIndex=0, autocomplete/autocorrect/autocapitalize=off, spellcheck=false. aria-multiline=false except on ChromeOS. Cover localized non-ASCII labels and ChromeOS detection from actual adapter input.
6. Initialize readonly from disableStdin and update it on option changes without discarding typed/composing text or replacing the textarea.
7. Repeated open preserves node identity and original parent. Source updates the browser window when its existing root was adopted into another document; verify renderer/device-pixel-ratio listeners use the new window without duplication.
8. DOM construction exceptions leave no attached partial root. Dispose removes only the terminal root, never the user's parent or siblings. Repeated dispose and dropped ownership cannot detach unrelated nodes.

## D2. Focus, cursor and textarea synchronization

1. Focus before open and blur before open behave as source. Repeated calls and disabled stdin preserve source behavior. Already-focused textarea avoids redundant focus operations; use preventScroll where supported by source.
2. Focus emits sendFocus reply ESC[I only when enabled, adds focus class, initializes cursor once, refreshes cursor and emits onFocus in source order. Blur clears textarea, refreshes cursor, emits ESC[O only when enabled, removes focus class and emits onBlur in source order.
3. Test focus listeners reentering resize/write/dispose, subscription removal and thrown JS callbacks. No Rust RefCell borrow may remain held across user callbacks. Propagate errors or record the exact source logging behavior, never swallow them.
4. Cursor textarea placement uses viewport-visible cursor, clamped cursor column, actual wide-cell width and CSS cell dimensions. Assert left/top/width/height/lineHeight/zIndex=-5. Skip synchronization while composition is active, cursor is outside viewport, DOM is unopened or dimensions unavailable. Include wide cell, trailing fullwidth cell, cursor at cols, alternate buffer and nonzero ybase.
5. Composition start updates cursor position before IME anchoring. Cursor movement, resize and rendered viewport changes update composition placement without premature text removal.

## D3. Rust engine, renderer and browser-service event routing

1. Instantiate services in source order. Forward bell/title/cursor/a11y/linefeed/writeParsed events exactly once. Refresh requests without a range cover the full viewport. Resize exposes committed dimensions before callbacks.
2. `refresh` delegates inclusive ranges to renderer, including synchronization on reset. Same-size resize remeasures invalid character dimensions; changed-size resize updates core first and renderer/textarea afterward.
3. Visible scroll operations use viewport smooth-scroll behavior where present and direct engine behavior before open. Cover scrollLines, page factor rows-1, top, bottom, forced immediate bottom, already-current line and out-of-range clamping.
4. Clear retains the cursor's current line as line zero, clears markers, resets y/ybase/ydisp, fills remaining viewport with default blank lines. Assert scroll event precedes viewport refresh, including synchronous event-handler writes.
5. Reset preserves cols/rows and custom key handler, resets mouse/selection/decorations, issues full synchronous refresh and does not discard pending write buffers or reset the external parser unexpectedly. Compare direct reset with in-band RIS and DECSTR.
6. Character joiners reject registration/deregistration before open, return stable IDs, and refresh only when source does. Link providers and decorations remain disposable; addon disposal does not dispose the terminal.
7. Source-provided renderer wins over default renderer during open. onWillOpen handlers run before default renderer selection; throwing handlers are logged per source. Dimensions getters clone public values so client mutation cannot mutate renderer state.
8. Bind copy/paste, key, composition, input, mouse, wheel, touch, selection, screen-reader and scrollbar services to live Rust state. Assert native engine parser writes, user data, generated replies and rendered text stay distinct. Do not echo user input into the display as a stand-in for engine I/O.

## D4. Browser protocol reports and theme changes

1. OSC4/104 targets ANSI indices and OSC10/11/12 or110/111/112 target foreground/background/cursor. Report precise RGB strings, set only addressed entries, reset addressed/all entries, and preserve request order for mixed set/query batches.
2. CSI996 color query uses current relative luminance classification. Theme changes produce unsolicited updates only when DECSET2031 enables them. Test equal-luminance and transparent colors with the source theme algorithm.
3. Window option reports use CSS canvas/cell sizes rounded with JS toFixed(0), not device pixels. Cover fractional sizes, retina devicePixelRatio, zero/unmeasured dimensions and unavailable renderer.
4. clearTextureAtlas routes to the active renderer only. DOM/no-atlas behavior must be source behavior, not a promise of WebGL equivalence.

## D5. WASM ABI and callback semantics

1. Build the wasm32 crate and binding JS with matching locked wasm-bindgen version. Load it in all three browser engines with no upstream terminal engine import or hidden JS fallback.
2. Preserve JavaScript strings as UTF-16 code units at the API boundary, including valid surrogate pairs, isolated high/low surrogates, NUL, U+FEFF and chunk splits. Do not use a lossy Rust String parameter for terminal write/input/event payloads. Typed byte inputs must be copied/snapshotted according to source async write behavior.
3. Number inputs are validated before conversion. Cover NaN, infinities, -0, negatives, fractions, 2^32 and 2^53 boundaries. No u32/i32 ABI truncation may bypass the source error contract.
4. Retain JS callbacks and Rust closures only while registered. Subscription disposal is idempotent. Dropping a JS handle without calling dispose must retain subscriptions if upstream does; explicit terminal dispose removes all listeners, scheduled jobs, observers and DOM nodes.
5. Callback ordering, optional/default arguments, getters returning undefined, own/property mutability, constructor-only options and proposed APIs match the shared public acceptance matrix. JS exceptions during callbacks propagate or log exactly as source; no catch-all success fallback.
6. Test reentrant JS writes, subscription mutation, disposal from event callbacks and callbacks queued after dispose. No panics, leaked cycles, use-after-free, or Closure::forget for indefinite listener ownership.

## Evidence and completion

For every D1-D5 bullet, record source branch, Rust implementation, assertion IDs, command, browser/version, exact pass/fail/skipped/unrun counts and raw artifact path. Test source and Rust with the same operation trace. A component test is marked component-only; it cannot count as a full Terminal integration test. Run native API/unit tests, wasm build/Clippy and browser runtime separately. Keep failing traces and seeds under results/browser/. No ignored failures, successful missing methods or source-backed fake terminal are accepted.

Current status: BrowserDom and API validation are partial host components. Full Terminal integration and D2-D5 behavior remain unproven until the core and browser services are connected. Overall equivalence is NOT PROVEN.
