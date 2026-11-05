# Link-cache disposal reentry

Task351 fixes cached-link disposal traversal in the browser linkifier. It does not close the selection/link family321.

The source `Linkifier._askForLink` traverses a Map with `forEach`, then each reply array with `forEach`. A dispose callback can synchronously complete a pending provider or replace a later reply. The traversal must observe that later entry. Rust previously collected all disposal callbacks before invoking the first one, so it skipped newly completed replies and disposed replaced links instead of their replacements.

Browser caches now retain `Rc<BrowserLink>` payloads, including their callbacks. The existing generic `ProviderReplies` priority algorithm still selects and filters links. Its typed range adapter accepts `AsRef<Link>` payloads, so native callers and browser callers share the algorithm. Interior mutability already belongs to `ProviderReplies`; the adapter no longer needs an exclusive outer borrow. Five native test bindings consequently no longer need `mut`, with assertions unchanged.

Disposal retains the original map and the currently visited array. It reads later map entries when reached and captures each array's length before invoking callbacks. No cache/group borrow spans a callback. A nested pointer event can replace the active cache without redirecting the outer traversal or dropping its original callback identities. The existing weak controller references remain unchanged.

## Directed source comparison

`scripts/link-disposal-source.mjs` uses the original production Terminal and public `registerLinkProvider`, dispatching mouse events through its actual screen listener. It verifies the embedded Linkifier source in the original source map against the read-only TypeScript file and records both hashes. These are new regressions, not unchanged upstream tests.

The three traces cover:

1. Disposal completes a previously pending provider. Expected callbacks are `first,new`.
2. Disposal replaces a later cached reply. Expected callbacks are `first,new`, never `old`.
3. Disposal dispatches a nested mousemove that creates a different active cache. The outer traversal still completes its original map/array. Expected callbacks are `first,first,tail,tail`, never `new-cache`.

The original Terminal returned all three expected traces in Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4 on Windows. Before the fix, Rust Chromium passed the third regression and failed the first two. The corrected before-fix test source and after-fix test source have the same hash in their reports. Firefox/WebKit were not run before the fix.

After the fix, the new WASM component suite passed3/3 per engine. Existing component `wasm_linkifier` passed6/6 per engine, exported Terminal `wasm_terminal_links` passed2/2 per engine, and real web-links integration `wasm_utility_links` passed3/3 per engine. That is42 passing Rust test executions across these four suites and three engines. The new component tests exercise the production Rust DOM controller and provider service, not a JavaScript algorithm substitute. The new disposal scenarios do not yet run through exported Rust `registerLinkProvider`, which remains missing.

## Other checks

- Native focused link targets passed30 Rust test functions, including the existing4097 OSC cases and12219 provider/cache operations across2048 scenarios.
- The source oracle passed all7 unchanged OscLinkProvider/Linkifier assertion bodies. Seeded corpora were regenerated without changing fixture bytes. Seeds are `0x01a8c055` and `0x11f1e255`.
- Native xterm-web all-target/all-feature Clippy and scoped WASM library/four-suite Clippy passed with warnings denied. Full workspace rustfmt passed after rebasing onto main with the independently integrated marker-module ordering fix.
- Native tests and both Clippy commands passed again after rebase. Browser runs preceded that rebase; the owned implementation/test/script hashes remain unchanged. This is not a full integrated browser matrix.

The environment was Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100. Reports and concise logs are in `results/links/disposal-reentry/`. Reports retain hashes of the executed binaries and bindings; disposable binary artifacts are removed during cleanup. Native Windows execution does not establish Linux/macOS behavior.

## Reproduction

Point `NODE_PATH` and `PLAYWRIGHT_MODULE` at existing reference dependencies. Set `SOURCE_ROOT` to the read-only original tree with `lib/xterm.mjs` and its map, and `WASM_BINDGEN` to the locked CLI when it is outside PATH. Use owned O: targets/TEMP/TMP and a resource allocation for compiler/browser work.

```sh
node scripts/link-disposal-source.mjs
node scripts/link-oracle.mjs
node scripts/link-replies-oracle.mjs
WASM_TEST=wasm_link_disposal node scripts/browser-host-check.mjs
WASM_TEST=wasm_linkifier node scripts/browser-host-check.mjs
WASM_TEST=wasm_terminal_links node scripts/browser-host-check.mjs
WASM_TEST=wasm_utility_links node scripts/browser-host-check.mjs
cargo test --locked -j2 -p xterm-web --test links --test link_state --test osc_links_differential --test osc_links_live --test osc_links_policy --test link_replies_differential --test link_replies_live
cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_link_disposal --test wasm_linkifier --test wasm_terminal_links --test wasm_utility_links -- -D warnings
cargo fmt --all -- --check
```

The isolated task351 reviewer accepted `ebaba6e` after independently rerunning full rustfmt, all30 focused native tests, native all-target/all-feature Clippy and the scoped WASM Clippy command. The reviewer inspected all retained hashes and baseline/fixed test identity, and verified the42 browser pass records without rerunning browsers. No source change followed that review.

Full equivalence remains NOT PROVEN. The browser cache still uses typed text/range data. Live JavaScript link objects, getter/Proxy timing, live callback replacement, dynamic decorations, public custom-provider registration and the complete source family remain work in321. `docs/live-link-acceptance.md` retains the full contract. This repair must not be treated as implementing that contract.
