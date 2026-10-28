# Actual Viewport implementation contract

Task239 owns `src/viewport.rs`, `src/viewport_browser*.rs` and `src/terminal/viewport_host.rs`. Norman owns the actual SmoothScrollableElement under167 and its root scroll exports. Xavier owns BrowserServices fields, constructor integration, root modules and renderer API changes. No successful missing-Viewport branch or synthetic native scroller is an implementation of the browser feature.

The portable controller implements the actual queue/sync/scroll/touch logic from `source_repo/src/browser/Viewport.ts`, not a second terminal state. Its host reads live renderer/buffer values at each source checkpoint and delegates position/dimension writes to the real element. The same controller is used by the WASM adapter. It holds no RefCell guard across a host call or event callback.

A `ViewportHost` supplies live `display_row`, `cell_height`, `canvas_height`, `buffer_line_count`, synchronized-output state, actual `scroll_top`, actual dimension/position setters and a render-service refresh-callback registration. Setter reuseAnimation is `Option<bool>` so omitted differs from explicitfalse/true. Scheduling returns the actual refresh ID; callback ownership mirrors the existing renderer debouncer rather than a new timer framework.

`ViewportController` exposes queue_sync, sync, on_render, scroll_lines, scroll_to_line, handle_touch_scroll, handle_scroll and on_request_scroll_lines. FieldslatestYDisp/queuedID/isSyncing/isHandling/suppress/needsSync remain one authoritative controller state. Source exception partial state is preserved; flags are not reset by a convenience finally guard after a failed host callback.

Required checkpoints:

- queueSync updates latest row before coalescing, clears queuedID before sync and preserves callback-before-ID-assignment reentry.
- sync evaluates its default ydisp before its early guard, defers during synchronized output, sets syncing/suppression before live geometry reads, writes realdimensions, clears suppression, then stops smooth movement only if externalydisp differsfrom latest.
- render clears deferred flag before retrying sync. Activation clears latestbeforequeueing. Bufferresizequeues, bufferscrollsyncs immediately.
- handleScroll suppresses dimension-feedback and nestedscroll callbacks, uses JS Math.round including negativezero, commits latestbefore dispatching requested rowdifference and leaves partialflags onthrow.
- scrollLines reads actual position then currentcellheight and uses reuseAnimationtrue; scrollToLine cacheslatest onlywhen smoothdisabled; touchsubtracts pixels withreuse omitted.

Browser construction must compose actual167+161model, all source options, protocol-wheel policy, colors/styleelement, buffer/core/render/theme subscriptions and DOMremoval. ActualBrowserServices central methods must use that ownedViewport and clone itsRc before callback. Renderer geometry remainslive and currentDPR-aware.

This numeric controller uses binary64 fields matching normal source scroll-state types. Arbitrary JS string/object scroll-position coercions remain explicit215work; no sourceequivalence is inferred for those until its sharedraw contract is implemented. Nativecontroller tests are not browser proof. Actualelement/factory/three-engine and gesture-to-core scroll evidence remain mandatory.
