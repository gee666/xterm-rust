# Browser links and render debouncing

Status: partial implementation, full browser equivalence NOT PROVEN.

## Implemented contracts

`crates/xterm-web/src/links/` contains portable OSC 8 discovery, provider registration, asynchronous reply prioritization, overlap filtering, hover/activation decisions, underline coordinates, animation-frame and time-based render debouncing. URI text is raw UTF-16, including lone surrogates. Host callbacks are explicit effects rather than invoked under a mutable state borrow.

`browser.rs` uses the actual browser WHATWG URL parser for http/https filtering and preserves the upstream default confirmation/open/opener/location sequence. `browser_debounce.rs` owns real animation-frame and timeout closures, cancels before releasing callbacks, and captures weak portable state to avoid reference cycles. Errors are returned, or reported on Drop where Rust cannot return them.

The OSC range algorithm deliberately preserves two surprising source behaviors. Starting a new run at trimmed EOL continues without emitting it. Wrapped-range probing also mutates the same work cell used by the outer scan, so the next run may use the last probed id rather than the original column's id. A seeded differential test found this distinction. `results/links/counterexample-shared-work-cell.json` retains the input and cause.

## Verification commands and observed results

Run at repository root with Node 24.15.0 and Rust/Cargo 1.85.1:

```sh
node scripts/link-oracle.mjs
cargo test -p xterm-web --test links --test link_state --test osc_links_differential --test debounce
WASM_TEST=wasm_links node scripts/browser-host-check.mjs
```

The oracle transpiles read-only TypeScript modules in memory using the reference's TypeScript dependency. It runs all seven original OscLinkProvider/Linkifier tests without editing their bodies or assertions, using actual upstream BufferLine/CellData/TestUtils and Chai/jsdom. All seven passed. Source hashes and original test names/outcomes are in `results/links/oracle.json`.

Seed `0x01a8c055` generated 4,096 random multi-row cell/id/wrap cases plus one fixed case, stored in `crates/xterm-web/tests/fixtures/osc-links.tsv`. The native test compares all 4,097 expected link lists, ranges and URI text. This corpus uses seven specified URIs; it does not claim arbitrary URL parser coverage. Separate browser tests exercise the real URL parser.

On native Windows after run2 resume, the focused native command passed 27 Rust tests: 11 link cases, 6 reply/hover cases, 9 debounce cases and 1 differential loop over 4,097 cases. This run emitted unrelated core warnings in active parser/terminal/buffer/write-queue files, so it is not a warning-free workspace gate.

The first live WASM test attempt failed during dependency compilation, before the link tests ran. Its exact command and diagnostics are retained at `results/browser-host/wasm_links/2026-10-27T23-49-46-271Z/`. At that snapshot decoration.rs used the removed InvalidArrayLength type and decoration_service.rs did not propagate SortedList.delete's Result. Owners were notified. These are integration work, not external blockers.

## Remaining integration and evidence

- Compose portable reply/hover state into a live DOM Linkifier with asynchronous provider callbacks, per-row caching, mouse path guards, renderer invalidation and custom handlers. The existing helpers are not a claim that a complete Linkifier is wired.
- Wire OSC lookup to the live marker-backed OscLinkService and terminal buffer. Bradley owns that core service.
- Wire BrowserRenderDebouncer into the renderer and BrowserTimeDebouncer into accessibility rendering. Live host wrappers alone do not prove terminal integration.
- Complete per-window DOM animation queue and helper behavior beyond the shared DomListener supplied by Xavier.
- Run the five WASM assertions against Chromium, Firefox and WebKit, retain exact outcomes, and fix failures. Add full-terminal browser assertions after real Terminal integration.
- Add original-source differential traces for reply permutations and virtual-clock debounce operations, beyond native explicit boundary/reentrancy tests.
- Independent review, complete workspace Clippy/fmt/test gates, and non-Windows runtime evidence remain required.

`docs/link-acceptance.md` controls the complete path and edge-case acceptance requirements. Criteria45-49 remain incomplete until that full evidence exists.
