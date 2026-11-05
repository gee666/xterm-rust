# Live typed image option inspection

Task402 exposes `ImageAddon._opts` as a stable read-only diagnostic of the existing native `ImageOptions`. It closes six unchanged original constructor-option assertions that previously read `undefined`. It does not implement the source's writable raw JavaScript option record.

The view has the twelve declared source keys in declaration order. Each enumerable getter reads the same Rust `Rc<Cell<ImageOptions>>` that controls activation, protocol setup, graphics controls and public properties. No option values are mirrored into JavaScript. The view keeps its identity before and after activation, controls, reset and disposal. Retaining the view retains only the option allocation, not a terminal, handler, renderer, event listener or addon owner. It remains readable after the owner is released.

The JavaScript adapter only defines and freezes this getter object. Writes, deletion and adding unknown fields fail rather than pretending to update Rust. Known values use the current typed Rust representation. Arbitrary extra properties, symbol keys, raw object identity, mutable private fields, getters installed on private options and deferred raw JavaScript coercion remain311/324 work. This task neither hides those gaps nor changes the original assertions. Public setters and the existing native policy still perform all mutations.

## Observed verification

Windows, Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100:

- Exact workspace formatting passes. The first invocation at the longer M402 worktree path hit Windows OS206. Moving the owned worktree with Git to the shorter sibling M allowed the exact command to pass.
- Native workspace warning-denied Clippy and all-target/all-feature tests pass2065 tests across415 summaries, with0 failed or ignored tests. Native execution does not run the WASM assertions.
- Focused xterm-web WASM warning-denied Clippy passes.
- The directed coordinator matrix passes24 tests in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. Three new tests cover default/custom/null/inherited/getter constructor inputs, the separate reset-default copy, retained view identity, DECSET and graphics updates, soft/hard reset, successful and failing public setters, disposal and explicit rejection of diagnostic writes.
- The unchanged original `addon-image` constructor-option family discovers6 case-projects, executes6, passes6 and has0 failed, skipped or unrun cases. The production browser binding, not only a Rust helper, supplies this view.

- After refreshing onto main5082b32a, the directed24×3 matrix passes again. The full unchanged image family discovers639 case-projects, executes635, passes622, fails13, skips4 and leaves0 unrun. The thirteen failure titles, projects and messages exactly match the preceding handler-inspection report after removing the six constructor-option failures. Nine still require private core/render services; four still report the retained source-baseline165-versus160 image geometry discrepancy. No assertion was changed or rounded.

Raw directed/source/input identities, original Playwright assertions, reference hashes and native gate logs are retained under `results/image-option-inspection`. Temporary paths in the reports identify the historical build. Generated packages and duplicate test output are disposable after review. Reference copies were hashed against the retained P: source files and match.

## Reproduce

Use the prerequisite paths in `docs/image-canvas.md`. Keep source tests unchanged.

```sh
cargo fmt --all -- --check
cargo clippy --locked -j2 --workspace --all-targets --all-features -- -D warnings
cargo test --locked -j2 --workspace --all-targets --all-features
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
node scripts/e2e-upstream.mjs --suite addon-image --grep 'ctor options' --timeout-ms 180000
```

This is scoped typed inspection evidence. The full image suite is executed but not passing. img01..48, raw options, private core services and cross-platform/runtime equivalence remain NOT PROVEN.
