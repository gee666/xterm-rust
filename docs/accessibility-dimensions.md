# Live accessibility dimensions

Task 547 addresses one part of input/accessibility task 319. Overall equivalence remains NOT PROVEN.

## Source contract

`source_repo/src/browser/AccessibilityManager.ts` reads dimensions and terminal rows at individual DOM operations. `_refreshRowsDimensions` first checks cell height, selects the container's style object, then evaluates width and font size before `Object.assign` writes either property. A width setter can change later row heights but cannot change the already evaluated font-size argument.

Each row gets a fresh style receiver and cell-height value. The loop rereads terminal rows on every condition. After setting height, it selects the current row again for alignment. A callback can shrink, replace or dispose rows between those operations. Alignment clears transform, measures, reads current cell width when mapped columns exist, then selects style again for the scale assignment. Getter/setter errors stop the sequence and retain the original thrown value.

The previous Rust path cached a complete `Dimensions` and row count, used `style.setProperty`, and retained the height row for alignment. CSS property overrides therefore saw different calls, state and errors.

## Rust implementation

`accessibility/browser_dimensions.rs` owns these phases. `browser_rows.rs` keeps row-tree construction, rendering, focus rotation and resize. Row creation and dimension refresh share the same height-assignment method. Existing primitive `begin_style` and strict `assign_style` helpers are reused; no DOM cache is introduced.

The additional `Object.assign` binding performs only that language/DOM operation. Rust evaluates its receiver and arguments and owns all dimensions/row traversal. It does not delegate accessibility behavior to JavaScript.

`AccessibilityHost` keeps the existing aggregate `dimensions` contract and adds individual cell-height, cell-width, canvas-width and font-size reads. Defaults use the aggregate snapshot for existing hosts. The actual Terminal overrides those reads so unrelated dimension accesses do not repeatedly coerce its font-size option. No Rust borrow spans a DOM property, host callback or value conversion.

## Verification scope

`scripts/accessibility-dimensions-source.mjs` extracts three unchanged method bodies from the original file and checks the committed test-only fixture. It records the original raw SHA-256, while normalizing only source line endings when copying bodies. The library never imports this fixture. These are additional reference-derived tests, not claimed upstream test cases.

Eight shared real-DOM scenarios compare source and Rust traces, resulting CSS values, cutoff points and thrown-object identity. They cover ordinary assignments, style-getter mutation, width-setter mutation, changing later row heights, shrinking the loop bound and errors from container style, width assignment and row-height assignment. A separate actual opened-Terminal test checks that resize propagates a width-setter error through real render/accessibility services. Existing component and real-Terminal assertions remain unchanged.

On native Windows, both the initial and restored final fixed runs passed all 33 component tests and eight actual-Terminal tests in Chromium 148.0.7778.96, Firefox 150.0.2 and WebKit 26.4. Every run had zero failures, ignored/filtered tests or page errors. The four component-console warnings per engine are the existing invalid-selection controls, two invalid-row and two missing-column messages. No unexpected console error/warning occurred in the fixed runs. Actual-Terminal runs had none. The existing 25 component and seven Terminal tests were kept intact. The added tests are eight component functions and one Terminal function, not claimed unchanged upstream cases.

The exact four production files from base `0442e357` were temporarily restored for Chromium controls. They produced 26 passed/seven failed component tests and seven passed/one failed Terminal test. All failures were the intended new regression assertions; the existing tests and the already-correct style-getter error control passed. A `finally` block restored and SHA-verified all four fixed files. Both targets then passed again in all three engines. Initial and final compiler WASM, served WASM and binding hashes are identical for each target.

The focused native accessibility run passed 11 tests, including the existing seeded announcement and BMP-classification differentials. Warning-denied focused native Clippy and WASM library/both-test Clippy passed. The latter was rerun after the test packaging correction. The source verifier and eight source-only controlled-field probes passed; these Node probes are not DOM or Rust evidence. Focused rustfmt passed. Exact `cargo fmt --all -- --check` still failed with Windows OS206, retained as a failure. Full workspace/release/native OS matrices remain task 304.

Failed development attempts are retained. An initial WASM type error required explicitly binding the current row before borrowing it. The first browser attempt could not load a relative source-fixture import because wasm-bindgen copies declared snippets, not their relative JS dependencies. The generator now combines the same unchanged source bodies and cases into one served test module. No assertion, production behavior or browser deadline was weakened. An initial source-only mock-host ownership error is retained separately.

Raw reports, engine logs, source/restore pins and quality logs are under `results/accessibility-dimensions547/`. Generated WASM, bindings and scratch are disposable; their hashes are retained for review and cleanup. These scoped results do not establish full accessibility or terminal equivalence.

Commands, using an owned O: worktree, existing matching wasm-bindgen 0.2.100 and Playwright installation:

```sh
node scripts/accessibility-dimensions-source.mjs
node --test scripts/accessibility-dimensions-source.test.mjs
node scripts/audit-accessibility-dimensions547.mjs
# Before disposal, also verify all recorded runtime artifacts:
node scripts/audit-accessibility-dimensions547.mjs --runtime
cargo test --locked -p xterm-web --test accessibility_state --test accessibility_differential -j2
cargo clippy --locked -p xterm-web --all-targets --all-features -j2 -- -D warnings
cargo clippy --locked -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_accessibility --test wasm_accessibility_terminal --all-features -j2 -- -D warnings
CARGO_BUILD_JOBS=2 CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_accessibility node scripts/browser-host-check.mjs
CARGO_BUILD_JOBS=2 CARGO_FEATURES=acceptance-hooks WASM_TEST=wasm_accessibility_terminal node scripts/browser-host-check.mjs
```

The full original input/accessibility family, every constructor/DOM mutation boundary, raw nonnumeric font-size representations, lifecycle/GC closure and native Linux/macOS proof remain separate requirements. A component pass or source fixture does not establish those contracts.
