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

## Observed checks

On main base `1106d480`, unchanged source method checks passed all 26 cases. The opened original Terminal passed the same 26 cases in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. The actual Rust Terminal passed one function replaying all 26 cases per engine, both initially and after the negative control. The two exact old production files compiled successfully and failed Chromium at `hover: provided=true, expected false`. Both final files were restored with matching SHA-256 hashes. Initial and final compiler WASM, served WASM and bindings hashes match; the negative artifact hashes differ. All three new-test compiler records report `fresh=false`.

Seven retained actual Rust browser targets passed 18 functions per engine: public providers, provider transaction reentry, hover-event sampling, live columns, linkifier, disposal and terminal links. These are distinct from the new 26-case function and do not establish full family coverage.

`cargo test --locked -j2 -p xterm-web --all-targets --all-features` passed 415 tests across 172 summaries, with zero failed, ignored, measured, filtered or warnings. Strict Clippy passed for native workspace all-targets/all-features, WASM workspace all-feature libraries, and the new WASM test. These checks used Rust/Cargo 1.85.1 on native Windows, Node 24.15.0 and wasm-bindgen 0.2.100. They are not a current stable-toolchain gate or native Linux/macOS evidence.

Raw tool logs retain their terminal blank lines. Git's full diff whitespace check reports those log endings; the source-only whitespace check passes. No raw receipt is trimmed to make that check green.

All five whole-member formatting checks passed. Exact `cargo fmt --all -- --check` failed with Windows OS error 206. The failure is retained, not waived or reported as a workspace formatting pass. Both the original and physical reference copy matched all 776 baseline hashes. `results/link-mouse-path/index.json` pins source/build inputs, raw receipts, generated runtime artifacts and cached native link executables for independent inspection. Runtime files are temporary and remain only until review and owned cleanup; their recorded hashes remain durable evidence afterward.

## Scope limits

This is not a full original suite, demo distribution, browser compatibility matrix, stable/release workspace gate or native Linux/macOS proof. Sparse/custom mapped provider replies, raw selection/underline number domains, deeper callback replacement and full lifecycle/GC proof remain open. Current main's accepted fixes, including task 500 underline geometry and task 499 frame callbacks, must be preserved. Raw results and final observed counts are recorded separately under `results/link-mouse-path/` after execution.
