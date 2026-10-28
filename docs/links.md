# Browser links and render debouncing

Status: partial implementation, full browser equivalence NOT PROVEN.

## Implemented contracts

`crates/xterm-web/src/links/` contains portable OSC 8 discovery, provider registration, asynchronous reply prioritization, overlap filtering, hover/activation decisions, underline coordinates, animation-frame and time-based render debouncing. URI text is raw UTF-16, including lone surrogates. Host callbacks are explicit effects rather than invoked under a mutable state borrow.

`browser.rs` uses the actual browser WHATWG URL parser for http/https filtering and preserves the upstream default confirmation/open/opener/location sequence. `browser_debounce.rs` owns real animation-frame and timeout closures, cancels before releasing callbacks, and captures weak portable state to avoid reference cycles. Fallible constructors return render/callback exceptions to JavaScript through Result-returning closures. Flags and callback arrays retain the source's partial state on failure. Drop reports cleanup errors where Rust cannot return them.

`browser_linkifier.rs` installs real DOM mouse listeners and composes the reply/hover algorithms with activation, dynamic decorations, renderer invalidation, disposal and pre-open provider registration. `NativeOscLinkProvider` reads actual Rust terminal cells and the same marker-backed OSC link store used by the parser, with brief fallible row borrows rather than copying scrollback. IDs retain all safe JavaScript integers through 2^53. `animation_queue.rs`, `dom_helpers.rs` and `window_interval.rs` implement per-window priority queues, owner-document/view selection, page coordinates and cancelable interval timers. These latest additions still need the pending browser test runs and complete Terminal composition described below.

The OSC range algorithm deliberately preserves two surprising source behaviors. Starting a new run at trimmed EOL continues without emitting it. Wrapped-range probing also mutates the same work cell used by the outer scan, so the next run may use the last probed id rather than the original column's id. A seeded differential test found this distinction. `results/links/counterexample-shared-work-cell.json` retains the input and cause.

## Verification commands and observed results

Run at repository root with Node 24.15.0 and Rust/Cargo 1.85.1:

```sh
node scripts/link-oracle.mjs
node scripts/debounce-oracle.mjs
node scripts/link-replies-oracle.mjs
cargo test -p xterm-web --test links --test link_state --test osc_links_differential --test osc_links_live --test debounce --test debounce_errors --test debounce_differential --test link_replies_differential
WASM_TEST=wasm_links node scripts/browser-host-check.mjs
WASM_TEST=wasm_linkifier node scripts/browser-host-check.mjs
WASM_TEST=wasm_dom_support node scripts/browser-host-check.mjs
```

The oracle transpiles read-only TypeScript modules in memory using the reference's TypeScript dependency. It runs all seven original OscLinkProvider/Linkifier tests without editing their bodies or assertions, using actual upstream BufferLine/CellData/TestUtils and Chai/jsdom. All seven passed. Source hashes and original test names/outcomes are in `results/links/oracle.json`.

Seed `0x01a8c055` generated 4,096 random multi-row cell/id/wrap cases plus one fixed case, stored in `crates/xterm-web/tests/fixtures/osc-links.tsv`. The native test compares all 4,097 expected link lists, ranges and URI text. This corpus uses seven specified URIs; it does not claim arbitrary URL parser coverage. Separate browser tests exercise the real URL parser.

On native Windows after run2 resume, the first focused command passed 27 Rust tests: 11 link cases, 6 reply/hover cases, 9 debounce cases and 1 differential loop over 4,097 cases. A subsequent run passed 3 additional callback-failure tests. Another focused run passed 4 tests covering live-source error/large-id behavior and the two new source differential traces. These runs total 34 distinct native tests, but occurred at different snapshots, not as a single warning-free workspace gate. Later cache-disposal-order and callback-state changes need rerunning.

Seed `0x6deb0ace` records 8,192 virtual event-loop operations against the unchanged RenderDebouncer and TimeBasedDebouncer algorithms. Seed `0x11f1e255` records 12,219 priority/cache/overlap operations across 2,048 scenarios against the unchanged Linkifier methods. Both Rust differential tests passed. A negative-zero mismatch was found and fixed; `results/links/counterexample-signed-zero.json` retains the input and cause. These source modules have no original dedicated debounce test files; custom traces are not counted as original assertions.

The earlier `wasm_links` snapshot passed all 5 Rust browser assertions in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Raw commands, browser versions and outputs are under `results/browser-host/wasm_links/2026-10-27T23-52-20-179Z/`. This is component evidence, not full-terminal equivalence. Later Result-returning callback and native-OSC integration changes have not yet rerun at the current snapshot.

The first live WASM test attempt failed during dependency compilation, before the link tests ran. Its exact command and diagnostics are retained at `results/browser-host/wasm_links/2026-10-27T23-49-46-271Z/`. At that snapshot decoration.rs used the removed InvalidArrayLength type and decoration_service.rs did not propagate SortedList.delete's Result. Owners were notified. These are integration work, not external blockers.

## Remaining integration and evidence

- Run `wasm_linkifier` and `wasm_dom_support` against all three browsers and fix failures. The new controller and shared native-OSC paths have test code but no observed browser pass yet. The real terminal write/wrap/click/trim case must execute before criterion250 can be accepted.
- Wire BrowserLinkifier and NativeOscLinkProvider into the exported Terminal runtime, using its actual renderer coordinate service and shared pre-open provider registry. Lane's web-links adapter consumes this same registry. Bradley owns the core OSC handlers/store.
- Wire BrowserRenderDebouncer into the renderer and BrowserTimeDebouncer into accessibility rendering. Live host wrappers alone do not prove terminal integration.
- Validate and compose per-window animation queues and interval/window helpers into the actual browser runtime. The browser platform must share one WindowAnimationQueues registry across services/terminals, rather than creating independent same-window queues.
- Rerun the five existing WASM assertions after fallible-callback migration, then add full-terminal browser assertions after actual Terminal integration.
- Extend differential traces to reentrant provider-cache disposal, live mutation during traversal, callback exceptions and mutable JavaScript custom-handler range identity. Current native callbacks use immutable Rust link data; a complete JavaScript binding must not silently drop the source's mutable-object behavior.
- Builds are currently paused by the team because P: approached1GiB free. Kassa coordinates generated-artifact cleanup. Preserve all source, fixtures and evidence; do not create duplicate Cargo target directories or claim the pending runs passed.
- Independent review, complete workspace Clippy/fmt/test gates, and non-Windows runtime evidence remain required.

`docs/link-acceptance.md` controls the complete path and edge-case acceptance requirements. Criteria45-49 remain incomplete until that full evidence exists.
