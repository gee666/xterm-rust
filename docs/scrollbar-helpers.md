# Scrollbar helper verification

Task165 covers `crates/xterm-web/src/scroll/{scrollbar_state,visibility,fast_dom_node,browser_visibility,widget}.rs`. These are helper components, not proof of full terminal scrolling equivalence. Overall port equivalence remains NOT PROVEN. Raw JavaScript positions remain task215; Element integration is167, oriented scrollbars/Arrow/pointer capture are166, and Viewport integration is71/239/251.

## Contracts and mapping

| Source | Rust | Evidence |
| --- | --- | --- |
| `browser/scrollable/scrollbarState.ts` | `scrollbar_state.rs` | Three directed native tests and 1,024 unchanged-source snapshots, seed `0x5c7011` |
| `browser/scrollable/scrollbarVisibilityController.ts` | `visibility.rs`, `browser_visibility.rs` | Seven native tests, five shared source/Rust browser cases, two existing asynchronous browser cases |
| `browser/scrollable/fastDomNode.ts` | `fast_dom_node.rs` | Eight shared source/Rust browser cases and five existing browser cases |
| `browser/scrollable/widget.ts` | `widget.rs` | Four shared source/Rust browser cases |

There are no dedicated upstream `*.test.ts` files for these helpers in the supplied reference. The shared browser cases are source-derived assertions, not invented counts of ported upstream tests. `scripts/scrollbar-helpers-source-check.mjs` bundles the unchanged source and runs the same fixture functions as the Rust WASM suites. It records all bundled input hashes. The Rust suites instantiate Rust helpers, not the source implementation.

### Geometry

`ScrollbarState` follows the six-number constructor and snake_case setters/getters. Constructor visible/scroll/position values are not eagerly rounded. Scrollbar/opposite/arrow sizes use JavaScript Math.round, including negative zero and ties toward positive infinity. NaN propagates rather than following Rust's NaN-eliding `f64::max`. The source minimum slider size is20 even when it exceeds the available track and creates a negative ratio.

`set_opposite_scrollbar_size` and `set_scrollbar_size` do not refresh computed values. Clone reconstructs and recomputes. Desired offsets/deltas/pages are not clamped here; Scrollable owns clamping. Native tests compare all exposed geometry, setter return values, clones and desired-position methods against source-generated bit patterns.

### Visibility and DOM

`VisibilityController<S: TimerScheduler>` owns class/timer policy. Unknown visibility values remain raw and do not coerce into known numeric constants. A reveal commits visible before scheduling zero-delay work. Timeout delivery clears its token before the class write. Hide distinguishes immediate removal from fade. Disposal closes the timer without resetting the source's visible flags.

`BrowserVisibility` owns the Rust callback until cancellation/drop and uses `BrowserTimerScheduler`. The supplied Window is the source module's ambient timer realm, not necessarily the DOM node's owner window. Explicit host errors retain their thrown JavaScript value identity. Disposed-timer errors retain the source wording.

`FastDomNode` accepts `JsValue`, `OptionValue`, `f64`, UTF-16 `JsString`, and Rust string callers through `DomValue`. Clones share cache identity. Only numeric dimensions gain a px suffix. Other values stay raw until DOM conversion. Cache writes precede host assignment, including when a setter or coercion throws. A private JS value mirror preserves the RHS read after a reentrant style getter. Inline JS performs property assignments and calls, not terminal policy.

`set_width/height/top/left/bottom/right`, position, contain, class, layer hinting, toggle and attribute paths are exercised by shared fixtures. They cover duplicate and initial caches, fractional/negative/NaN/infinite numbers, raw UTF-16, object identity, coercion hints, failed getter/conversion cache state, class reentry, raw toggle force and non-deduplicated attributes. Existing tests retain real asynchronous visibility turns without sleeps.

`Widget` is Rust listener composition rather than inheritance. `on_click`, `on_mouseover`, and `on_mouseleave` register actual disposable DOM listeners. Normalization resolves the current node window at dispatch, including adoption into another document. Shared cases check all three kinds, event identity/coordinates/button metadata, preventDefault/stopPropagation, recursive disposal, iframe adoption and callback-error identity. Broader mouse/wheel normalization remains task167.

## Commands

From a worktree with Rust/WASM tools, the supplied reference and its Node dependencies:

```sh
cargo test --locked -p xterm-web --test scrollbar_visibility --test scrollbar_state_differential
cargo test --locked -p xterm-web --lib scroll::scrollbar_state
node scripts/scrollbar-state-oracle.mjs
node scripts/scrollbar-helpers-source-check.mjs
WASM_TEST=scrollbar_helpers_browser node scripts/browser-host-check.mjs
WASM_TEST=wasm_fast_dom_differential node scripts/browser-host-check.mjs
WASM_TEST=wasm_widget_differential node scripts/browser-host-check.mjs
WASM_TEST=wasm_visibility_differential node scripts/browser-host-check.mjs
cargo clippy --locked -p xterm-web --target wasm32-unknown-unknown --test scrollbar_helpers_browser --test wasm_fast_dom_differential --test wasm_widget_differential --test wasm_visibility_differential -- -D warnings
```

Repeat the two native test commands with `--release`. For an isolated worktree without local Node dependencies, set `ESBUILD_MODULE` to an installed esbuild package and `PLAYWRIGHT_MODULE` to Playwright's `index.mjs`. `CARGO` and `WASM_BINDGEN` can select tool executables. wasm-bindgen CLI must match the locked0.2.100 runtime. Browsers are Chromium, Firefox and WebKit by default. Servers close in finally blocks.

## Observed results and limits

`results/scrollbar-helpers/verification.json` records this run and links its raw reports/logs. Native debug/release helper suites each pass11 tests; the geometry generator matches1,024 snapshots. The existing browser suite passes7 tests per engine. The shared suites pass8 FastDom,4 Widget and5 visibility cases per engine, with identical fixtures passing against unchanged source. Native web all-targets/all-features passes301 tests.

The full workspace gates did not pass on the Ashleyccc2e83 base. Workspace tests fail the unrelated `core_services_integration.rs:188` callback-state assertion. Workspace Clippy fails `public_cell_numbers.rs:69` for `field_reassign_with_default`. `cargo fmt --all -- --check` fails with Windows OS206, command-line length; direct checks of the owned helper/test files pass. These failures are retained, not waived or suppressed. No Linux/macOS runtime result is claimed here.

Initial harness failures are retained too. The first browser runner failed because its newly created private target lacked the standard Cargo cache tag. After creating that tag, it ran unchanged. The new attribute fixture initially made its intercepted method read-only, preventing the intended error phase; its descriptor was corrected for both source and Rust. Chromium normalizes button values on mouseover/mouseleave, so shared assertions use the delivered MouseEvent.button, exactly as the source does. No upstream input/assertion was changed.

Generated WASM/binding bundles remain local under report-linked artifact paths and are hashed. The small JSON reports and raw logs are tracked; the large generated bundles are not required source inputs and are not committed. Re-running the commands rebuilds them. This is component evidence, not full browser-engine, trusted-input or platform equivalence.
