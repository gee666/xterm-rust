# Utility web-link callback ranges

Task450 repairs the browser WebLinksAddon adapter on main `3aa04692`. URL computation already ran in Rust, but the controller received immutable native link ranges while the JavaScript hover callback received a detached range object. Mutating that object could not change later pointer decisions. The old callback adapter also called option methods without their source receiver.

## Implementation

The typed `BrowserWebLinkTerminal` and native addon API remain unchanged. `JavaScriptWebLinkTerminal` delegates buffer snapshots and URL discovery to those existing Rust implementations and registers results with the existing `BrowserLink::JavaScript` controller path. It intentionally ignores the generic provider's native callback wrappers. Public browser callbacks are attached once, after all URL computation, without duplicate option getter reads. No second URL matcher, provider registry or pointer controller is introduced.

`raw_link_bridge.js` constructs only host binding objects. Its callback attachment follows the original WebLinkProvider `_addCallbacks` method. `leave` is captured when each result is attached. `hover` rereads the live options twice, samples `link.range` between those reads, and calls the second value with the options receiver. `activate` retains the raw handler and the controller supplies the live link receiver. The link's retained range is also the object used by Rust hit tests, mousedown equality and selection. Replacing endpoint objects or the entire link range remains observable.

An undefined handler receives a GC-owned Rust callback at addon construction. It calls the existing Rust default opener and retains no terminal, provider or link. Other raw values, including null and false, remain callable-boundary errors instead of selecting the default. JS owns link/range/callback edges, so no pointer-keyed identity cache or Rust callback cycle is needed. No dependency or unsafe code is added.

## Verification commands

Set `SOURCE_ROOT`, `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` to the installed reference and matching tools. New temporary targets and scratch must remain on the user-authorized O: volume. Run sequentially within a granted heavy lane.

```sh
node scripts/web-links-callback-check.mjs
node scripts/web-links-live-range-source.mjs
WASM_TEST=wasm_web_links_live_range node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_links node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_bindings node scripts/browser-host-check.mjs
WASM_TEST=wasm_public_link_provider node scripts/browser-host-check.mjs
WASM_TEST=wasm_linkifier node scripts/browser-host-check.mjs
cargo test --locked -p xterm-web --all-targets --all-features
cargo clippy --locked -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -p xterm-web --all-targets --all-features --target wasm32-unknown-unknown -- -D warnings
```

The source runner verifies original terminal/addon bundle source maps against current source bytes. Only the original generates the expected fixture with `--generate`. Original sources and assertions are not modified. The callback checker executes unchanged source `_addCallbacks` against the production binding factory across 24 directed cases, including primitive options, getter order, receivers, captured leave, replaced ranges, non-callable handlers and exact thrown identity. These are callback-unit cases, not browser or whole-engine equivalence.

The shared browser trace opens an actual original or exported Rust Terminal and explicitly activates the addon. It does not test the distribution wrapper's `loadAddon` method. It covers mutation during hover, same-cell suppression, leave/reentry, mousedown equality, double-click selection, replaced endpoint objects, ordered Proxy reads, exact thrown identity, live hover replacement, captured leave and full range replacement. Additional actual activation probes cover default successful/blocked opening, tolerated opener exceptions, exact location exceptions, null/false/zero/string/object handlers and handler-thrown identity. Every synthetic down has an up, and all owned addons, listeners, mocks and DOM nodes are restored in finally.

## Scope limits

Full selection/links task321 remains unfinished. This repair does not establish raw mapped-container/sparse/malformed provider parity, out-of-domain geometry, all reentrant getters, lifecycle GC, all private addon fields or cross-addon default function identity. It does not run the complete unchanged browser/demo matrices or native Linux/macOS tests. Native helper tests and focused browser regressions cannot prove full equivalence. Overall equivalence remains NOT PROVEN.

## Observed results

The final 28-phase shared trace passes original and Rust in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 on native Windows. Its ten activation probes each use a fresh opened Terminal and assert that the default opener or error path was actually reached. Reusing one terminal initially allowed stale provider cache results to make those probes vacuous; that development trace is not evidence for default activation. The final source selects `hij` after endpoint mutation and `://` after complete range replacement.

A negative control replaces only the two existing adapter/module files with exact main `3aa04692` bytes. The old code compiles and fails the real browser trace comparison in Chromium. It keeps the pointer active outside the mutated range, misses Proxy getter observations and selects the original URL. Finally restores both fixed files with exact SHA checks. The restored final trace passes all three engines. The first negative-control driver looked for the assertion in the output field rather than the console field. The underlying comparison failure and restoration were valid; the retained final driver checks the correct console field.

Before the final default-handler construction was moved from activation to construction, four retained suites passed 14 test functions per engine, 42 expanded cases. These were utility links3, utility bindings3, public providers1 and linkifier7. The utility link suite includes 495 original hostname cases with 3,465 path assertions per engine and wrapped/wide/combining offset assertions. These retained runs are not a claim that all 14 functions ran again on the final source revision. The final shared trace separately exercises the changed default-handler path.

Native xterm-web all-target/all-feature tests passed 360 tests over 149 summaries with zero failed or ignored tests. Native warning-denied all-target/all-feature Clippy passed. Final WASM warning-denied all-target/all-feature Clippy passed, as did formatting of the four changed Rust files. The full exact formatting command failed with Windows OS206 and remains a failure, not a formatting pass. Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 were used. Full workspace quality remains the separate quality lane's responsibility.

Results and exact source/artifact hashes are indexed under `results/web-links-live-range/`. Browser artifact paths in the reports identify disposable build products, not files that must be retained. The reproducible commands, fixture, source and non-secret receipts remain durable.
