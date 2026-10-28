# Image renderer terminal lifecycle

Task127 supplies `ImageRendererLifecycle` over the accepted272 browser canvas implementation. It runs against the Rust `Terminal` and its actual RenderService, DOM renderer, options and canvas metrics. The full ImageAddon and its protocol registration remain task38. Overall equivalence is NOT PROVEN.

## Native contract and ownership

Create `ImageRendererLifecycle::new(&terminal)` before or after opening the terminal. Share `.renderer()` with ImageStorage. Use `.show_placeholder(bool)` rather than the low-level canvas method when a terminal repaint is required. The addon must call `.dispose()` in its own disposal traversal, so an exception remains returnable. Dropping the owner also attempts cleanup and reports an exception to the browser console.

The lifecycle owner listens after a successful first open, including when a failed open preceded it. An already-open terminal binds immediately. The canvas host reads the current browser service window/document, screen element and renderer dimensions on each operation. Before open, cell metrics remain unavailable. Placeholder screen width still comes from the module-global screen, matching source.

A fontSize change rescales existing output canvases, then requests rows0 through terminal.rows. Other option changes do not run this image callback. The image listener is registered at construction time, preserving its order relative to the terminal's existing listeners. Placeholder toggles request the same row range even when the placeholder state did not change.

Renderer changes remove a snapshot of image layer keys in insertion order before delegating to the backend setter. Equal backend assignments still run the wrapper. Native interceptors run newest-first, like nested source method wrappers, with no Rust borrow held across callbacks. A thrown removal stops replacement and retains the original JS exception object and partial layer state. Disposal during removal clears the saved delegate and suppresses that pending backend call. If disposal throws before reaching the delegate cleanup, the delegate remains active while the owner lives, and repeated dispose does not silently retry. Destruction still removes its native interceptor without retrying the throwing DOM operation.

Disposal unregisters the option/open listeners, removes top then bottom, detaches the renderer wrapper, clears the retained render service and closes the current placeholder bitmap. The accepted canvas component retains source late-promise assignment behavior. No cancellation or close of an unsettled bitmap is invented. Listener callbacks and the canvas host hold weak state/terminal references, so a live terminal cannot retain an abandoned lifecycle owner.

Rust uses typed post-open hooks and renderer interceptors instead of mutating private JavaScript method properties. Native hook removal replaces restoration of those private method slots. This is not an exported JavaScript ImageAddon binding. Task38 still owns that binding and its addon-manager traversal through terminal disposal. Arbitrary external reassignment of upstream private _core methods is not tested or claimed by this native adapter.

## Commands

Prerequisites and environment overrides are the same as `docs/image-canvas.md`. Both commands compile the full xterm-web crate now. The historical isolated-component mode has been removed.

```sh
node scripts/image-canvas-check.mjs
WASM_TEST=image_lifecycle_browser node scripts/image-canvas-check.mjs
cargo test --workspace --all-targets --all-features
cargo clippy --locked -p xterm-web --lib --test image_canvas_browser \
  --test image_lifecycle_browser --target wasm32-unknown-unknown -- -D warnings
```

On this Windows host the reference dependencies and wasm-bindgen CLI are outside this worktree, so the observed runs set REFERENCE_NODE_MODULES, PLAYWRIGHT_MODULE and WASM_BINDGEN to their existing root-repository installations. No reference file or assertion was changed.

## Evidence

The lifecycle suite passes13 Rust/WASM tests in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. It includes64 steps per browser from seed0x127389, comparing font/layer/replacement/placeholder state and all pixels in each extant output canvas with unchanged ImageRenderer.ts. That is192 lifecycle steps, not an exhaustive proof. It also checks real render notifications, ownership/drop, before/after open, failed open, font filtering, exception identity, insertion order, reentrant disposal and failed-disposal partial state. Source traces check refresh bounds and restored source methods independently.

The accepted canvas suite also passes14 tests per browser against the full browser crate, including its128 draw/extract pairs per browser from seed0x272389. The earlier272 isolated results remain historical evidence, not the current build route.

Latest reports and immutable per-run logs are in `results/browser-host/image_lifecycle_browser/`, `results/browser-host/image_canvas_browser/`, and their latest JSON files. `results/image-lifecycle/verification.json` and `results/image-canvas/verification.json` retain source and Rust hashes. Failed test-development runs are retained, including a malformed test-only JS function expression corrected before the passing run. Independent review found a dead interceptor after failed disposal followed by owner drop, and a missing final layer-map clear after reentrant insertion. Both counterexamples are retained in results/image-lifecycle/review-counterexamples.json and now have three-browser source/Rust regression tests.

Native `cargo test --workspace --all-targets --all-features` passes1942 tests across337 result summaries, with0 failed or ignored. This is local Windows evidence only. `results/image-lifecycle/workspace-tests.log` retains the full output.

The required workspace rustfmt command fails Windows error206 because its command line is too long. Individual changed-file rustfmt checks pass. Native workspace Clippy fails the existing excessive_precision literal in renderer/width_cache_tests.rs131. Scoped WASM Clippy fails existing Closure type_complexity in links/animation_queue.rs40 and selection/dom.rs70. Logs remain under `results/image-lifecycle/`. No lint suppression or fixture change was used to hide these failures. Full release/feature/platform gates remain open.

## Main integration

The canonical worktree helper rebased accepted272 and127 onto current main and fast-forwarded main to7ad4129. It ran298 xterm-web tests across92 summaries, native dependent checks and the browser WASM library checks, all passing. The complete receipt is results/image-lifecycle/gated-merge.log. After integration both browser suites were rerun on that committed tree:13 lifecycle and14 canvas tests passed in each engine. Reports link the new immutable runs, and results/image-lifecycle/integrated-lifecycle.log and integrated-canvas.log retain their summaries.

The helper removed the private target after merging. A follow-up browser run initially failed preflight because statfs could not inspect the absent target. Creating the same designated target directory before rerunning resolved it. No second cache or headroom bypass was used. The failed run receipts remain in the immutable browser results.

Task277 still owns accepted QOI branch integration. The lifecycle branch includes accepted272 canvas commits rebased onto the compiling main checkpoint. Task38 must still activate all image protocols, prove original full-addon suites and bind this lifecycle through the actual addon manager. This task does not claim that work has passed.
