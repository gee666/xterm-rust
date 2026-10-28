# Browser link component contract

Cameron owns `crates/xterm-web/src/links/`. Xavier owns Terminal/runtime and DomListener; Lane owns utility addon provider bridges; Cornelia owns rendering. Do not edit another owner's module to resolve interface errors without coordination.

## Pre-open provider service

`browser_linkifier::BrowserProviders` is `Rc<LinkProviderService<dyn BrowserLinkProvider>>`. The exported terminal creates it during construction, not open. Addons may register before open. `BrowserLinkifier::with_providers(element, cols, scroll_offset, coordinates, underline, providers)` receives that same Rc on open. Controller disposal does not dispose the externally owned service. The terminal disposes the service at its own lifecycle boundary.

Providers implement `provide_links(y, callback) -> Result<(), JsValue>`. Rows are one-based absolute buffer rows, including scrollback. Callback distinguishes None from Some(empty). It may run asynchronously or more than once, matching the source's callback API. It holds only weak controller state. Replies after mouse-out or disposal are ignored. Late replies from an older row deliberately use the source's uncanceled callback semantics, not a silently different generation filter.

`BrowserLink` holds `Rc<Link>` data, optional decoration flags, required activation callback and optional hover/leave/dispose callbacks. `Link.text` is Vec<u16>, not String. `Range` coordinates are one-based and inclusive at the end. Callback functions and fallible coordinate providers return Result so original JavaScript exceptions can retain identity rather than becoming successful no-ops. `with_fallible_providers` installs real computed-style/native-state coordinate reads; the original infallible constructor remains a convenience wrapper. Provider registration handles use explicit disposal, like core lifecycle handles. Every disposal removes the first matching provider identity, including repeated registration of the same object.

## Coordinate and renderer hooks

The coordinate callback receives MouseEvent and returns a one-based Position or None. It must use live renderer measurements, current terminal dimensions and current ydisp. The controller cannot infer geometry from guessed font sizes. Underline callbacks receive show/hide plus zero-based viewport coordinates, possibly negative above the fold, and the current column count.

- Call resize(cols, ydisp) after core dimensions change. It clears hover and marks the row cache stale for the next changed cell.
- Call set_scroll_offset(ydisp) when the displayed buffer offset changes.
- Call on_render(start, end) with zero-based viewport row bounds after rendered rows change. Full-range containment follows upstream, including the start-zero case.
- Call set_current_underline or set_current_pointer_cursor for binding decoration setters.
- Dispose the controller's DOM listeners before disposing shared providers and terminal state. `dispose(&self)` is idempotent and can be called through Rc.

No RefCell state borrow is held across activation/hover/leave/dispose callbacks. Leave keeps the current link available until callbacks finish successfully, preserving partial failure state.

## Native OSC provider

Register `NativeOscLinkProvider::new(Rc::downgrade(native_terminal), window, handler_settings)` before external providers, as upstream does. The settings callback returns Result<Option<OscLinkHandler>, JsValue> and is sampled once for each existing requested row. Missing rows do not read options. The allow_non_http_protocols member is a fallible getter, evaluated after URI lookup and wrapped-range expansion for each completed nonempty link run. None uses confirmation/default navigation and only valid http/https URIs. A custom handler specifies the protocol bypass and activation/hover/leave callbacks.

Discovery reads the active Rust buffer and `TerminalState.links` directly. The OscBuffer.with_line contract releases each row borrow before URI lookup or host callback. It does not snapshot or clone scrollback. URL IDs use u64 to retain source safe-integer identities, including 2^53, and URI strings retain unpaired UTF-16 surrogates. WHATWG URL's USVString conversion replaces surrogates only at the URL-constructor boundary, never in stored callback text.

A full JavaScript custom-handler binding still needs deliberate shared range-object mutation/identity behavior. Rust callback arguments are immutable references, which alone do not prove the mutable JavaScript contract. Track that gap rather than silently advertising equivalence.

## Scheduling helpers

`BrowserRenderDebouncer::try_new(window, render)` and `BrowserTimeDebouncer::try_new(window, threshold_ms, render)` take fallible callbacks. Existing new constructors wrap infallible callbacks. Host closures return Result to the JavaScript event loop; callback failure stops subsequent callbacks/cleanup exactly where source execution stops. Native debouncers expose on_frame/on_timeout as Result, and tests must check it. BrowserRenderDebouncer.add_try_refresh_callback propagates callback failures without clearing the pending callback array. BrowserTimerScheduler::new(window, Function) is shared by other timeout-driven browser components; callers own the Rust Closure until outstanding jobs are canceled. Reflect-based clearTimeout preserves host exceptions.

Use one `WindowAnimationQueues` registry per browser platform and share it across all services/terminals. for_window coalesces calls targeting the same Window. Scheduled runners receive a queue handle, so reentrant scheduling need not capture a strong queue and form a reference cycle. WindowIntervalTimer runners likewise receive their timer handle for safe reentrant cancellation.

Current runtime evidence and unrun checks are in `docs/links.md`. Complete acceptance is `docs/link-acceptance.md`.
