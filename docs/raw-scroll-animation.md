# Raw smooth animation

Task417 extends the shared ScrollValue policy from409 into SmoothOperation. Task301 still owns raw live Scrollable and Element consumers. These component changes do not establish full scrolling or port equivalence.

The same Rust curve algorithm now handles f64 and JsValue. It captures cubic deltas during construction instead of recomputing them during ticks. Raw from values survive for addition at each tick; final ticks return raw to values. Long-distance ascending and descending branches use distinct source addition/subtraction operations, not a signed numeric shortcut. Viewport multiplication and horizontal/vertical construction preserve source coercion order. Raw duration addition in start can concatenate. The host clock callback is sampled only after duration addition, preserving mutations made by duration coercion. Samples remain f64 Date.now milliseconds.

During dimension acceptance, undefined target coordinates are absent, so incoming raw state coordinates provide the fallback. Dimension acceptance publishes the new target, horizontal curve and vertical curve in that order. A later throwing coercion does not roll back earlier assignments. The shared state algorithm still clamps dimension updates. Fallible generic methods propagate thrown values; numeric wrappers retain their infallible API. The binding additions implement only language arithmetic and Math operations, not animation logic or a JavaScript terminal engine.

## Source-derived verification

The reference has no dedicated smooth-operation test suite. These additional tests extract the unchanged ScrollState and smooth-operation classes plus their helper functions from source_repo/src/browser/scrollable/scrollable.ts. The complete source SHA-256 is 010ddcb39e40b268e4f37afc1da968ae30b64eaf356c918bb08484558150eba4. The generated test-only oracle is allowed to exceed the file limit because it preserves upstream code with generated formatting and its source-derived cases. Production Rust does not import it.

Three Rust browser functions compare 540 expanded cases per engine:270 primitive/raw-time cases,145 object/coercion/cached-delta cases and125 dimension-acceptance cases. Inputs cover short, long ascending and descending curves, completion before start/at cut/at end, NaN/infinities, raw strings/booleans/null/undefined, BigInt/Symbol failures, changing object coercion, endpoint identity and exception identity. Every case compares complete snapshots and coercion traces against independently constructed source inputs. Acceptance cases continue ticking after captured failures to check partially published curves and target values.

Commands, from the owned O: worktree with installed ESBUILD_MODULE, PLAYWRIGHT_MODULE and matching WASM_BINDGEN supplied:

```sh
node scripts/raw-scroll-state-source.mjs
node scripts/raw-scroll-state-source.mjs --animation
cargo test -p xterm-web --lib --test scrollable --test viewport_controller -j2
cargo clippy -p xterm-web --all-features --lib --test scrollable --test viewport_controller -j2 -- -D warnings
cargo clippy -p xterm-web --all-features --target wasm32-unknown-unknown --lib --test raw_scroll_animation_browser -j2 -- -D warnings
CARGO_BUILD_JOBS=2 WASM_TEST=raw_scroll_animation_browser node scripts/browser-host-check.mjs
```

On native Windows, the focused run passed72 tests:56 library, five Scrollable and11 ViewportController. The native and WASM focused all-feature Clippy commands passed with warnings denied. Changed-file rustfmt and source whitespace checks passed. Unmodified tool logs retain their trailing blank lines. cargo fmt --all -- --check failed before formatting with Windows OS error206; no full formatting pass is claimed.

All540 cases passed in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4. The three Rust functions per engine had zero failed, ignored or filtered tests. Reports retain artifact hashes, complete engine output and exact command scope under results/raw-scroll-animation. Rustc1.85.1, Cargo1.85.1, Node24.15.0 and wasm-bindgen0.2.100 were used. Both the owned physical reference copy and the original P: reference retain the source hash above. No source input or assertion was edited.

Independent review found an undefined target-coordinate fallback error in dimension acceptance. The fix treats undefined as absence in the shared value policy and adds three source differential cases for either or both coordinates missing, with intermediate/final ticks. The updated540 cases passed in all three engines. browser-initial.json retains the earlier537-case report. Native72 and focused strict native/WASM Clippy were rerun successfully after the fix. Resumed independent review and canonical integration are pending. Full workspace quality, native Linux/macOS, live callback/reentry/disposal and Scrollable/Element migration remain separate gates. All new scratch/build outputs are under O:/xterm-rust-swarm-16/tmp/HannoRaw; generated WASM/bindings are disposable, not distributed sources.
