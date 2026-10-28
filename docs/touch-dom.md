# Browser touch recognizer

`input::touch::GestureContext` ports `source_repo/src/browser/scrollable/touch.ts`. RuntimeContext retains a context per document and supplies the shared per-window animation queue. Registration tokens remove only their own target; duplicate targets remain distinct. Listener/frame callbacks capture Weak context references.

The implementation preserves source event names and field presence, deepest-first normal routing, ignore filtering, tap-clock timing and insertion-order inertia routing. The live linked list allows containment callbacks to change registrations without holding a borrow across host calls. Native payload/history ownership does not require a browser in the core.

Task 200 extends the same producer with exact getter checkpoints and independent X/Y/time histories. Its complete contract is `docs/touch-live-fields.md`. The small static JS bridge performs only native property/error operations; all gesture decisions and arithmetic execute in Rust.

## Checks

- `cargo test --locked -p xterm-web --test touch_policy --test touch_histories`: 10 passed.
- `WASM_TEST=wasm_touch node scripts/browser-host-check.mjs`: 12 tests passed in Chromium 148, Firefox 150 and WebKit 26.4.
- `WASM_TEST=wasm_touch_live node scripts/browser-host-check.mjs`: all 91 original-source scenarios matched in each engine, covering 28,987 observations per engine.
- The actual terminal library suite passed 13 tests in each engine, including touch DOM→MouseBindings→Viewport→BufferService movement, inertia, cancellation and disposal.
- Focused WASM Clippy for both browser targets and the acceptance library passed with `-D warnings`.

The browser fixtures use real DOM delivery with controlled clocks, rAF and touch lists. They are not physical touch-hardware tests. No JavaScript terminal engine executes the Rust producer's decisions. The source has no dedicated original touch test suite; the oracle runs its unchanged implementation.

Task 251 connects actual Viewport scrolling; task 141 owns complete terminal input integration. Component success is not full terminal equivalence. Full equivalence remains NOT PROVEN.
