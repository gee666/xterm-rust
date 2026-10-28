# Browser mouse DOM binding

Task #110 ports listener ownership and DOM routing from `source_repo/src/browser/services/MouseService.ts`. It uses the shared `MouseStateService<()>` from the actual terminal and the pure coordinate, wheel, touch-accumulator and report policy from task #85. It does not create another terminal engine or another mouse mode service.

## Binding contract and ownership

`input::dom::mouse::MouseBindings::new` installs the source's protocol subscription, mouseEventsRequireAlt option subscription, initial active-protocol reassignment, always-on mousedown/passive-wheel listeners and gesture consumer. `MouseHost` supplies live metrics/options, native WheelEvent veto, selection/focus effects, binary versus user-data output, viewport touch scrolling and the real Gesture.addTarget token. The real gesture producer is task #108; a manual custom event is not proof of touch recognition.

The terminal coordinator owns this binding alongside other input bindings. It must call reset when the terminal mouse service resets, and dispose input before removing DOM nodes. Protocol and option subscriptions, requested move/wheel listeners, current-document global release/drag listeners, Alt listeners and gesture tokens are all owned resources. Construction unwinds through RAII on failure. No Closure::forget or reference cycle is used. Listener closures capture Weak state.

Every DOM callback uses the reentrant fallible listener contract. It may recursively dispatch another DOM event without an FnMut alias failure. Getter, veto and output errors cross the WASM boundary after Rust guards drop. No RefCell borrow is held across a host callback. Protocol/option callback errors stop their core dispatch rather than becoming a successful no-op; a JsCallbackError retains the thrown value. MouseStateError and native error-source chains preserve that exact value for the coordinator's to_js_error conversion; no stale out-of-band error slot is used.

## Required source paths

- Mousedown always prevents default and focuses first. Disabled mouse reporting or forced selection skips sending and globals. Otherwise attach requested release and drag listeners to the element's current ownerDocument, not the document from open.
- Global mouseup sends before clearing globals on the final released button. Drag handles held buttons; element move handles no held buttons. Protocol changes attach/remove requested move/wheel listeners and clear no-longer-requested global handlers.
- Wheel listeners explicitly use passive=false. Active reporting always cancels default and propagation even when the report is vetoed or suppressed. Passive fallback returns immediately when wheel reporting is active. Its veto leaves default untouched. In a buffer without scrollback, consume wheel fractions and emit exactly one application/normal cursor sequence when a nonzero amount is consumed. Normal scrollback leaves viewport wheel handling to its owner.
- Geometry samples the screen's current bounding rect and owner window's computed padding. Report coordinates are computed before character-size rejection, as upstream does. Re-read routing after the native wheel veto because it can change modes/options or resize.
- The browser native-wheel veto receives the original WheelEvent. Restrictions and encoding use the terminal's unit-typed shared mouse service, while the DOM host owns the original-event veto. The adapter does not call that veto a second time through a unit-payload hook.
- Mouse output uses binary for DEFAULT encoding and user-input data for other encodings. Alt-gated non-wheel reports strip local Alt before encoding. Wheel reports preserve Alt.
- MouseEventsRequireAlt changes synchronously resync selection enablement, root CSS class and Alt listeners. Protocol binding logs occur before mode/class changes. Binding logs happen only at the source's exact debug option; unbinding calls the debug logger unconditionally and lets its filter apply.
- Active Alt gating installs keydown/keyup on the original document, movement on the element and blur on the element's current owner window. getModifierState('Alt') drives the class; blur clears it. These listeners disappear when gating is inactive or input is disposed.
- Gesture change cancels default/propagation first, then routes to mouse wheel, alternate-buffer cursor keys or the viewport. Wheel gestures bypass the native WheelEvent veto, as source does. Touch-start clears accumulation. Repeated output retains individual event boundaries. Nonfinite or unrepresentable synthetic gesture line counts return an explicit error rather than entering the source's infinite loop.

## Verification

Run the existing browser skill, using the shared Cargo target and existing low-artifact profiles:

```
WASM_TEST=wasm_mouse node scripts/build-slot.mjs --owner NAME --lane wasm --wait-ms 600000 --timeout-ms 240000 -- node scripts/browser-host-check.mjs
node scripts/build-slot.mjs --owner NAME-lint --lane wasm -- cargo clippy -p xterm-web --target wasm32-unknown-unknown --lib --test wasm_mouse -- -D warnings
```

The WASM suite exercises actual DOM listener dispatch, cancellation, mode changes, current-document adoption, recursion, original getter errors and cleanup in Chromium, Firefox and WebKit. Its injected MouseHost records outputs and selection/focus effects. This is a browser component test, not the complete exported Terminal or PTY end-to-end test. Its gesture-consumer case uses explicit custom events and does not claim recognition or inertia coverage.

Tests cover global release cleanup, focus/forced selection, data/binary routing, native-wheel veto and cancellation, alternate-buffer keys, Alt option and blur lifecycle, gesture output branches, recursive wheel dispatch, ownerDocument changes and callback-failure partial state. Exact observed browser counts/results and build failures belong under `results/browser-host/wasm_mouse/`.

Current status is pending the first successful complete WASM build and observed browser run. Task172 adds live field/host read timing tests beyond the seven initial listener/consumer cases; see mouse-live-reads.md. Shared core, selection, scheduler, serialization and terminal-module API migrations have blocked earlier builds; those failed logs are retained. Pure mouse policy and unrelated browser tests do not count as this suite passing. Actual terminal reset, viewport/scrollbar behavior, real touch recognition and full Terminal integration still require tasks #33, #71, #108 and #31. Full equivalence remains NOT PROVEN.
