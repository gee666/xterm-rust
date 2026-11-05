# Image constructor option-copy failures

Task425 fixes one constructor ordering defect. It does not complete image addon parity.

The source `ImageAddon` constructor performs two consecutive `Object.assign` calls. The first captures current options and the second captures reset defaults. Rust previously converted the first copy to typed numbers before making the second copy. A first-copy numeric object could therefore throw from `valueOf` before the caller's second `ownKeys`, property descriptor, or getter callback ran. The source instead reaches that second copy and propagates its exception unchanged.

`crates/xterm-web/src/image/addon/options.rs` now makes both shallow copies before constructing the two typed option records. The actual public Rust/WASM `ImageAddon` constructor consumes that pair. No terminal implementation or decoding work is delegated to JavaScript. The existing `Object.assign` helper remains the binding operation for JavaScript enumeration and getters. No dependency, unsafe code, fallback or lint suppression was added.

## Verification

`crates/xterm-web/tests/image_addon_support/constructor_order.rs` adds two tests against the unchanged source `ImageAddon` class.

- Twenty failure cases combine four second-copy failure sites with five thrown values. The failure sites are `ownKeys`, `getOwnPropertyDescriptor`, a string-key getter and a symbol-key getter. Thrown values are an object, undefined, null, zero and a symbol. Each case compares callback order and exact exception identity. The first-copy numeric value throws a different error if coerced, and a nonenumerable getter must never run.
- A successful proxy trace checks both complete enumeration passes, distinct current and reset-default values, retained option inspection, and reset without calling the original getters again.

On Windows with Rust/Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100, the full directed image-addon target passes28 tests in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. This includes the two new tests and26 existing image-addon tests. These are directed Rust/WASM test functions, not the unchanged639-case upstream family or img01..48.

A negative control temporarily restored both production files to their exact old constructor implementation while retaining the new tests. Chromium ran28 tests, passed27 and failed the new exception-order test. The control was restored in `finally`, then the fixed target passed28/28 in each of the three engines again. `negative-browser.json` retains that expected failure. No original source assertion was changed.

Focused WASM Clippy passes with warnings denied. Formatting passes separately for all five workspace packages. The exact aggregate `cargo fmt --all -- --check` command still fails with Windows OS206, command line too long. The failed aggregate receipt is retained, not reported as a pass. No native Linux/macOS execution or final full-workspace gate is established by this task.

Raw reports, both positive runs, the negative control, source input hashes and quality logs are in `results/image-constructor-copy/`. Historical artifact paths refer to the owned O: worktree and are not promised to remain after cleanup. The reports retain hashes and browser outputs without committing WASM binaries.

## Reproduce

Use the prerequisites and explicit dependency/tool environment variables from `docs/image-canvas.md`. New temporary targets and scratch belong on O:.

```sh
WASM_TEST=image_addon_browser CARGO_BUILD_JOBS=2 node scripts/image-canvas-check.mjs
cargo clippy --locked -j2 -p xterm-web --target wasm32-unknown-unknown --lib --test image_addon_browser -- -D warnings
cargo fmt --all -- --check
```

## Remaining differences

Successful constructor copies still undergo Rust's existing eager typed numeric conversion. The source stores raw values and defers coercion to consumers. This task fixes the precedence of failures in the two copy passes only. It does not claim raw numeric/object option parity, a writable `_opts` record, complete private `_inputHandler` bindings, original full image suites, img01..48 or full native/browser equivalence. Those remain311/324 and the full project acceptance work. Overall equivalence is NOT PROVEN.
