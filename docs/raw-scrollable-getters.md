# Live Scrollable update getters

Task432 adds raw update-object entry points to the shared `Scrollable<S, JsValue>` engine. Typed numeric methods use the same update phases. No second state cache or JavaScript scroll algorithm is introduced.

- `set_raw_scroll_dimensions`, `set_raw_scroll_position_now` and `try_validate_raw_scroll_position` select the old state before property access. Reentrant getter calls can replace the current model state, but do not change that receiver.
- `set_raw_scroll_position_smooth` reads each axis separately. If an animation already exists, an undefined first read falls back to the operation current after that read. The fallback can fail before the next getter if reentry removed the animation. Only after both axes are expanded does the method select the state for target validation.
- A defined first read performs a second read, preserving inherited accessors, proxy receivers, changed results and thrown-value identity. A second read producing undefined remains explicit for direct updates. In the expanded smooth-update object it falls back to the selected state's raw request, matching the source's subsequent `withScrollPosition` call.
- Frame delivery treats undefined tick endpoints as absent properties of a plain update object. A getter's second read can leave an undefined smooth target, so this matters when the animation finishes after an earlier NaN intermediate tick. The current raw request must survive that final tick.
- `*_resolving` Rust methods expose these same phases to fallible host resolvers. Callers must preserve property-access order inside their resolver. No model borrow spans a resolver or value coercion.

Update phases live in `scroll/scrollable/updates.rs`, separate from event/animation ownership and frame delivery. The raw property helpers are shared with standalone `ScrollState<JsValue>` updates in `raw_state.rs`.

## Scope

This closes only live update-object evaluation. BrowserScrollable, Element, scrollbar geometry and Viewport still use numeric state. State-object identity and mutation, raw event exceptions and browser scheduler reentry remain301. Task443 adds raw Date.now samples/errors in the shared engine; the numeric BrowserScrollable clock binding still needs migration. See docs/raw-scroll-clock.md. The test scheduler does not prove those contracts. Full equivalence is NOT PROVEN.

## Tests and commands

`raw_scrollable_browser` retains605 previous live-engine scenarios and adds5804 source/Rust getter programs per engine. The getter matrix covers four update/validation paths, animation present/absent, duration0/90, own/inherited/proxy access, four changing/undefined patterns, six reentry/error actions, either getter read, each property, plus28 primitive update inputs and16 identity/coercion/reuse/raw-position programs. Source and Rust receive independent objects. Traces compare getter/proxy accesses, errors including token identity, current/future values, events and requested/cancelled/pending frames. The original source and assertions are unchanged. The generated oracle exceeds350 lines because it bundles original Scrollable/Event/Lifecycle plus the full action corpus.

Four new native tests cover old-state selection, validation/clamping and missing-axis reentry/cancellation. Final focused Windows all-feature native tests passed76, comprising56 library, nine Scrollable and11 ViewportController tests. Strict focused native/WASM all-feature Clippy passed again after the frame fix. Source oracle checks, changed-file formatting and all five member-specific cargo fmt checks passed. Exact workspace cargo fmt failed before formatting with Windows OS206. These are not full-workspace or Linux/macOS runtime receipts.

All5804 getter programs and605 retained live programs passed in each of Chromium148.0.7778.96, Firefox150.0.2 and WebKit26.4, two Rust test functions per engine, with no ignored/filtered tests. Both the post-fix and final matrices passed. The initial matrix failed the same real final-tick mismatch in all three engines. `browser-initial.json` and initial logs retain that failure, while `browser-fixed.json` and `browser-final.json` retain the passing runs. Source inputs and assertions were unchanged between these runs.

A Chromium negative control replaced the raw property helper's second read with a cached first value. Exactly the getter test failed at `rawDimensions/false/0/own/defined/none/1/width`; the605 retained live programs still passed. `negative-command.json` records that expected failure and mutated source hash. The helper was restored in finally, then native tests, both strict Clippy targets and the complete three-engine matrix passed again. Final source hashes match the restored implementation. No browser watchdog was increased.

In an owned O: worktree, set TEMP/TMP to its tmp directory and Cargo jobs to2. Reuse installed esbuild/playwright and wasm-bindgen0.2.100. Run:

```text
node scripts/raw-scroll-state-source.mjs --live
cargo test -p xterm-web --all-features --lib --test scrollable --test viewport_controller -j2
cargo clippy -p xterm-web --all-features --lib --test scrollable --test viewport_controller -j2 -- -D warnings
cargo clippy -p xterm-web --all-features --target wasm32-unknown-unknown --lib --test raw_scrollable_browser -j2 -- -D warnings
WASM_TEST=raw_scrollable_browser node scripts/browser-host-check.mjs
```

Retain exact command logs and source/artifact hashes under `results/raw-scrollable-getters` after execution. Do not substitute earlier426 receipts for this revision's checks.
