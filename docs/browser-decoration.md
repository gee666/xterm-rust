# Live browser buffer decorations

Task 193 ports `browser/decorations/BufferDecorationRenderer.ts` into the Rust/WASM renderer. The implementation uses actual Document, HtmlElement, CSSStyleDeclaration and requestAnimationFrame callbacks. It does not substitute a headless row model for browser rendering.

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

Use the controlled build queue with unchanged shared profiles and target directory:

```sh
node scripts/build-slot.mjs --owner decoration-source --lane wasm --timeout-ms 120000 -- node scripts/buffer-decoration-oracle.mjs --generate
WASM_TEST=wasm_buffer_decoration node scripts/build-slot.mjs --owner decoration-wasm --lane wasm --timeout-ms 420000 -- node scripts/browser-host-check.mjs
```

After deliberate initial generation, run the source oracle without `--generate` to verify fixture drift. It bundles the unchanged original component and real common Buffer/Decoration services, then records eight DOM observation steps in Chromium. No upstream BufferDecorationRenderer.test.ts exists, so the report records zero original test functions instead of inventing a baseline count. These observations are not the original full browser suite.

Seven Rust WASM tests cover real CSS geometry/layers/identity, zero defaults, anchor/layer quirks, viewport/alternate transitions, marker disposal, registration during rendering, detached pending callbacks, microtask indexing independent of renderer lifetime, and the exported Rust Terminal open/refresh/dispose path. The browser host runs Chromium, Firefox and WebKit by default. A single-browser pass is not a three-browser pass.

Current status: implementation and tests are written; Root BrowserServices integration is being wired by Xavier. The first source-oracle slot stopped at disk preflight before spawning a child, receipt `results/build-slots/1793150467482-29060-384c74a6-04be-442d-a065-e63f389995af`. No browser pass is claimed. Overall equivalence remains NOT PROVEN.
