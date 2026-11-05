# Link mousemove composed paths

Task 514 repairs raw path observation in the actual Rust browser Linkifier. Full selection/link task 321 and global equivalence remain NOT PROVEN.

## Contract and implementation

The unchanged `src/browser/Linkifier.ts::_handleMouseMove` samples coordinates, clears its mouse-out flag, then calls the event's current `composedPath` method. It reads the returned path's current length on each loop condition. Every visited entry supplies `classList.contains('xterm')`, then, unless that result is truthy, a separate `classList.contains('xterm-hover')`. A truthy root result stops traversal but allows hover handling. A truthy hover result suppresses hover handling. Method receivers, property access, numeric coercion and thrown values remain observable.

The previous Rust controller narrowed each path entry to a DOM Element. It silently skipped other targets, bypassed their getters and methods, and treated the returned path as an Array. This could request links beneath an embedder hover target that the source suppresses. It could also turn malformed paths into successful handling instead of propagating errors.

The new private Rust traversal controls the loop and branching. It reuses existing fallible JavaScript property, call and comparison primitives, with a numeric-key overload of the existing property primitive. No terminal-engine logic is delegated to JavaScript, no dependency is added, and no state borrow spans an embedder callback. All existing native/JavaScript provider, cache, underline and disposal behavior stays in the same controller.

## Reproduction

Use an isolated owned worktree and target. Set both TEMP and TMP to its scratch directory. SOURCE_ROOT names the unchanged reference, PLAYWRIGHT_MODULE its installed Playwright `index.mjs`, and WASM_BINDGEN the existing 0.2.100 CLI. Compiler/browser runs require the current resource owner's allocation.

```sh
node scripts/link-mouse-path-methods.mjs
node scripts/public-link-provider-source.mjs --mouse-path
WASM_TEST=wasm_link_mouse_path CARGO_BUILD_JOBS=2 node scripts/browser-host-check.mjs
node scripts/link-mouse-path-negative.mjs
WASM_TEST=wasm_link_mouse_path CARGO_BUILD_JOBS=2 node scripts/browser-host-check.mjs
```

The source-method runner transpiles unchanged original methods in memory. Only geometry and observation of downstream hover handling are supplied by the test. The shared 26-case program also runs against opened original and Rust terminals with real MouseEvents, public provider registration and DOM error delivery. It covers ordinary/empty paths, roots, hover targets, truthy non-booleans, repeated classList reads, contains getters and receivers, missing/null properties, exact thrown markers, array-like paths, numeric length coercion, live append/truncate, index getters and primitive paths. The fixture comes from the unchanged source method runner. The original three-engine run must agree with it; it must not be edited to hide a Rust failure.

The negative driver compiles two exact production files from base e64f4870 with the new assertions. It requires the intended Chromium runtime failure, restores exact final bytes in `finally`, and checks SHA-256 equality. A final fixed three-engine run is required after restoration. The existing runners bound compiler/browser/server lifetimes and own their cleanup.

## Scope limits

This is not a full original suite, demo distribution, browser compatibility matrix, stable/release workspace gate or native Linux/macOS proof. Sparse/custom mapped provider replies, raw selection/underline number domains, deeper callback replacement and full lifecycle/GC proof remain open. Current main's accepted fixes, including task 500 underline geometry and task 499 frame callbacks, must be preserved. Raw results and final observed counts are recorded separately under `results/link-mouse-path/` after execution.
