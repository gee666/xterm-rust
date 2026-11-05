# Live browser buffer decorations

`browser/decorations/BufferDecorationRenderer.ts` is implemented in the Rust/WASM renderer with actual Document, HtmlElement, CSSStyleDeclaration and requestAnimationFrame callbacks. The live Terminal now constructs it through `terminal/decoration_services.rs`; previously only controlled component fixtures constructed it.

## Contracts

`renderer/buffer_decoration.rs` owns subscription and element lifetime. `buffer_decoration_style.rs` implements source geometry/style order. `buffer_decoration_microtask.rs` and its small JavaScript scheduling bridge connect the portable line cache to browser microtasks. The bridge contains no terminal or indexing logic. It releases its one-shot Rust job after invocation or enqueue failure, reports the original error through the microtask, and does not rely on leaked event closures.

`BufferDecorationHost` supplies the live BufferSet, DecorationService, document/screen, metrics and render events. Its refresh callback capability must use the same RenderService queue as row rendering. BrowserServices owns this component. A separate `bind_microtasks` subscription belongs to browser Terminal construction, before open, because decoration indexing must keep working before DOM creation and after renderer replacement. It captures a weak service wrapper; individual queued jobs temporarily own only the cache handle.

The source quirks are deliberate:

- A pending refresh sentinel clears after onRender callbacks. Registration during that refresh does not enter the captured decoration snapshot and does not request another component frame.
- Width and height zero use one cell in this DOM renderer, even though zero height indexes no core lookup buckets.
- The top-layer class is assigned only when creating a node. A later layer mutation does not rewrite that class.
- Only the selected anchor side changes. Switching anchors does not clear the old opposite-side CSS value.
- Dimension refresh emits onRender before its separate X-position update.
- Off-viewport existing nodes are hidden and still emit onRender. Off-viewport decorations without nodes remain uncreated.
- Alternate-buffer activation hides all decoration nodes without updating their size/top until normal rendering resumes.
- The source removes a decoration then recursively invokes its disposal, before the outer disposal traversal finishes. The native event implementation must preserve that ordering.
- Disposing this component alone does not cancel its already queued RenderService callback. Such a callback can create children in its detached container. Whole-terminal disposal separately owns cancellation of the shared render service.

Opaque core render handles contain the same `Rc<HtmlElement>` value used for element storage. Search and other addons can downcast without losing node identity.

## Verification commands

Run in an owned O: worktree with its private target and Cargo `-j2`. Coordinate the four-heavy-job and two-browser-matrix limits with the integration owner. The former global build queue is retired.

```sh
node scripts/buffer-decoration-oracle.mjs
CARGO_BUILD_JOBS=2 WASM_TEST=wasm_buffer_decoration node scripts/browser-host-check.mjs
CARGO_BUILD_JOBS=2 WASM_TEST=wasm_terminal_decoration node scripts/browser-host-check.mjs
```

Point `PLAYWRIGHT_MODULE` and `WASM_BINDGEN` at the existing installed Playwright module and matching wasm-bindgen 0.2.100 CLI when they are outside the worktree. The source oracle uses the reference's existing Node dependencies. Do not install another copy just for this check.

After deliberate initial generation, run the source oracle without `--generate` to verify fixture drift. It bundles the unchanged original component and real common Buffer/Decoration services, then records eight DOM observation steps in Chromium. No upstream BufferDecorationRenderer.test.ts exists, so the report records zero original test functions instead of inventing a baseline count. These observations are not the original full browser suite.

Eight Rust WASM tests cover real CSS geometry/layers/identity, zero defaults, anchor/layer quirks, viewport/alternate transitions, marker disposal, registration during rendering, detached pending callbacks, microtask indexing independent of renderer lifetime, and the exported Rust Terminal open/refresh/dispose path. Two separate actual-Terminal regressions cover pre-open cache sync, public render ordering, alternate/normal transitions, resize, reset and whole-terminal cancellation of pending work. Reset and disposal clear the shared decoration service even before BrowserServices exists, so an old queued insert cannot shift decorations registered after reset. The browser host runs Chromium, Firefox and WebKit by default. A single-browser pass is not a three-browser pass.

The recovered component suite now passes eight tests per browser with the actual root overlay installed. Receipts and exact scope are retained under `results/browser-decoration/`. Public `registerMarker`, `markers` and `registerDecoration` browser bindings remain separate missing work. These tests register through the shared native service and do not prove those absent public APIs. Overall equivalence remains NOT PROVEN.
