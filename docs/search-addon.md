# Search addon integration

The Rust search addon uses `crates/xterm-addons/src/search/`, `crates/xterm-web/src/search_host.rs`, `crates/xterm-web/src/search_addon/` and `terminal/selection_api.rs`. Task290 restored the real browser regression build and fixed repeated lowercase work. Its retained native and three-browser evidence is in `results/search-browser/` and `docs/search-browser-verification.md`. Full equivalence is NOT PROVEN; task322 tracks the remaining family audit and raw-value gaps.

## Implemented source contracts

The portable addon coordinates the Rust search engine, an actual headless terminal, real markers and DecorationService. SearchHost supplies the shared selection service, DOM style application, monotonic clock and cancelable timeouts. It must not create a search-only selection or substitute a decoration command log for rendering. BrowserSearchHost uses Terminal's actual shared selection and decoration services. Source optional before-open operations remain optional; select throws TypeError before open.

SearchResultTracker preserves insertion order, row/column/size identity, limits, selected-resource disposal, result-change gating and reentrant callbacks without retaining RefCell borrows. Exceptions stop later operations. Selected decoration assignment does not automatically dispose its previous value, matching the internal source setter.

DecorationManager splits wrapped matches into rows, computes relative marker offsets, creates real top/bottom decorations and deduplicates overview markers by buffer row. Its managed resources use weak callback captures rather than subscription cycles. Explicit disposal invokes fallible marker callbacks. Drop only removes retained subscriptions and does not conceal callback errors.

The source passes false to its private style helper for active results too. The port therefore adds the base find-result CSS class, not an invented active class. Active colors and top layer still differ. A color-source contract retains live Rust RefCell or raw JavaScript decoration options, so borders are read when onRender fires rather than frozen before an animation frame.

The addon preserves the source assignment order that stores new options before checking whether options changed. A repeated term with changed flags can consequently leave its inactive highlights unchanged. This is tested as a compatibility quirk, not silently fixed. Actual searches still use the requested flags.

Cache subscriptions are installed at first nonempty search, not prematurely at activation. Linefeed, cursor movement and resize invalidate cache state and cancel its expiry timer. A 15,000ms inactivity timer reschedules against the most recent access. WriteParsed/resize refreshes replace a pending 200ms refresh and run incremental previous search without changing the viewport. Disposal cancels jobs and subscriptions.

## Browser binding and JS glue

The exported SearchAddon has activate, findNext, findPrevious, clearDecorations, clearActiveDecoration, dispose and stable event registration functions for onBeforeSearch, onAfterSearch and onDidChangeResults. Terms cross the boundary as UTF-16 rather than lossy Rust String. Search options remain provider-backed so an onBeforeSearch callback can mutate the original object before search reads it.

Three small static inline-JS operations implement property boxing/access and native JavaScript highlight-limit comparison/slice coercion. They do not perform matching, buffer traversal, selection, result tracking or rendering. They are necessary to preserve getter errors, NaN/fraction/infinity and Symbol/BigInt numeric behavior without dynamic eval. Their generated wasm-bindgen snippet files are mandatory distribution assets and must be included in size accounting. Source LoC includes the inline JS inside its Rust file.

Highlight limits deliberately distinguish comparison from Array.slice conversion. A limit of 1.5 can create two inactive decorations while the tracker reports one result. NaN permits highlight enumeration but slices the tracked results to zero. The Rust numeric implementation shares one slice-end helper; the JS adapter preserves raw coercion timing and original exceptions.

## Current tests and unverified paths

Native `xterm-web/tests/search_addon.rs` and its support/case modules exercise real parser-written buffers, SelectionService and DecorationService with a virtual clock. They cover the two original decoration assertions, result indices/event order, clearing without invented selection changes, active style behavior, numeric limits, option mutation, reentrant disposal/error identity, cache TTL/invalidation and 200ms debounce cancellation. Native element objects are deliberate style unit-test adapters, not browser rendering proof.

The six actual-Terminal browser regressions now execute and pass in Chromium, Firefox and WebKit. They cover shared selection/events, real DOM decoration styling, raw limits, option mutation/error identity, before-open behavior and wrapped-ring errors. The original unchanged search suite also passes all81 cases across its three projects. Public selection numbers outside i64 and negative zero still need the full family audit. Non-string truthy search terms retain an explicit binding restriction rather than full original dynamic-value behavior. General accessor-backed matching options still need precise property-read/exception-order tests; provider snapshots are not proof of every JavaScript getter path. These are implementation gaps, not external blockers.

Actual DOM decoration creation/onRender uses the root-owned BufferDecorationRenderer added by task296. It delivers real HtmlElement objects through Decoration::fire_render, not a search-only render substitute. Overview rendering is a separate renderer contract and is not established by the search suite. No source-JavaScript engine is used as a Rust fallback. Original SearchEngine/Cache assertions and ring behavior are tracked separately in `docs/search.md`.

Task290's passing suites establish only their stated scope. They do not resolve the raw term/options gaps, prove every source assertion is mapped, or establish native Linux/macOS runtime parity. Those remain completion work, not waived requirements.
