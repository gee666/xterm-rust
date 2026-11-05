# Hover event sampling after decoration getters

Task 478 repairs one browser link callback-ordering defect. Full selection/link parity and global equivalence remain NOT PROVEN.

## Source behavior

`source_repo/src/browser/Linkifier.ts` initializes the current link state by reading the live `decorations` property four times when the getter returns an object. Only after those reads does `_handleNewLink` call `_linkHover` with `this._lastMouseEvent`. A getter can dispatch a mousemove on an embedder `.xterm-hover` widget. That event replaces `_lastMouseEvent` without asking providers for a different link. Hover must receive that exact nested event, not the event used to calculate the original position.

The sampling boundary is before `_linkHover` emits its underline event. A range getter invoked while drawing the underline can update `_lastMouseEvent` again, but hover still receives the event captured at `_linkHover` entry. Disposal during decoration reads can instead clear `_lastMouseEvent`; the JavaScript callback then receives `undefined`.

The Rust controller previously retained the initial `MouseEvent` across coordinate, hit-test and decoration getter calls. It now samples the current event after decoration state initialization and before underline emission. The private payload invocation accepts a JavaScript event value so it can preserve `undefined` after disposal. Native provider callback signatures are unchanged and still require a `MouseEvent`. Native decorations have no JavaScript getters that can clear that event before sampling. No callback-spanning borrow, new dependency, unsafe operation or terminal-engine delegation is introduced.

## Reproduction and verification

The method oracle executes the unchanged original TypeScript in memory. Seven directed cases cover unchanged, first/last decoration read replacement, underline reentry, clearing the event, a thrown getter and a thrown hover. They execute the source `_handleNewLink` and `_linkHover` methods with minimal host dependencies. They are not browser proof.

The shared opened-Terminal program registers a real public provider and dispatches screen/widget DOM events. Seven cases cover unchanged, first/last decoration read reentry, underline-range reentry, thrown getter, thrown hover and terminal disposal during the last decoration read. Assertions check exact event object identity, callback receiver/text, hover count, exact thrown marker identity and descriptor installation. The same program runs against the original bundle and the exported Rust Terminal. The source runner verifies the bundle source map matches the original source files.

```sh
SOURCE_ROOT=/absolute/reference/source_repo node scripts/link-hover-event-methods.mjs
SOURCE_ROOT=/absolute/reference/source_repo PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs node scripts/public-link-provider-source.mjs --hover-event --generate
WASM_TEST=wasm_link_hover_event CARGO_BUILD_JOBS=2 PLAYWRIGHT_MODULE=/absolute/reference/source_repo/node_modules/playwright/index.mjs WASM_BINDGEN=/absolute/wasm-bindgen node scripts/browser-host-check.mjs
```

The generated fixture is an original-source trace. Do not update it to hide a Rust failure. Reports and exact command results are retained under `results/link-hover-event/` and the standard `results/browser-host/` paths. Compiler/browser binaries are temporary review artifacts and are removed during owned cleanup after evidence preservation.

## Observed results

On integrated base `26c7c42dabb5e2e8e4fdf1b157d3a188278dbf3e`, the original seven-case trace passed Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. The Rust test passed one function replaying the same seven cases per engine. The exact two production files from that base failed the new Chromium assertion, `first decoration read: hover must receive nested, got outer`. Both final files were restored in `finally` and their exact hashes checked before the final three-engine pass.

Five retained Rust browser targets passed per engine: public provider one function, provider transactions one function replaying six cases, linkifier seven functions, disposal three functions and terminal links four functions. These are 16 retained functions per engine, separate from the new seven-case function. They exercise actual exported Terminal bindings and native-provider component callbacks, not the demo distribution or the full original suite.

After rebasing onto the integrated base, `cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 397 tests across 163 summaries with zero failed or ignored. Native and WASM-target `cargo clippy --locked -j2 -p xterm-web --all-targets --all-features -- -D warnings` passed. Formatting passed separately for all five workspace packages. The exact `cargo fmt --all -- --check` command still failed with Windows OS error 206, both at the initial path and after shortening the owned worktree path. Those failed logs remain retained; no exact workspace-format-command pass is claimed.

The earlier pre-rebase native run passed 384 tests across 156 summaries and native/WASM-library Clippy. Its logs are explicitly labeled pre-rebase. They do not substitute for the final results. The source method oracle passed all seven cases, and all 776 original reference hashes passed the preflight check. No original source/assertion, AGENTS.md, credential or unrelated work was modified. Tools were Node 24.15.0, Rust/Cargo 1.85.1 and wasm-bindgen 0.2.100 on native Windows. Linux/macOS runtime proof is not claimed.

## Limits

This repair does not close arbitrary mapped-container, sparse/prototype provider reply, out-of-domain coordinate, raw drawing-event number, lifecycle-GC or full selection/link parity. It does not replace accepted task 464's request-flag transaction repair. The two scoped changes touch separate expressions in `browser_linkifier_events.rs` and must both survive integration. Full unchanged browser/demo suites, native OS matrices and workspace quality gates remain required elsewhere.
