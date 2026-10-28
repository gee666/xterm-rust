# Browser touch recognizer

Task 108 ports the DOM recognizer in `source_repo/src/browser/scrollable/touch.ts`. Portable arithmetic and exact-source policy traces belong to accepted task 139. Real engine consumption is task 141. These scopes must not be merged into a claim that browser scrolling has passed.

## Ownership and interfaces

`input::touch::GestureContext` owns removable document touch listeners. `new(document)` is an isolated convenience constructor. Actual RuntimeContext integration uses `new_with_queue(document, queue)` with its shared per-window `WindowAnimationQueue`, preserving the source Dom scheduler's batching and priority order. The context stores only the latest cancellation token, as source does; dropping an overwritten token does not cancel a different pending queue item.

`add_target(&HtmlElement)` and `ignore_target` return independently disposable registration tokens. Duplicate registrations remain distinct. Non-touch registrations return a shared immutable no-op token corresponding to `Disposable.None`. The no-op token has no gesture state. Active records, target lists and tap state are document-context data, never mutable core globals. Listener and animation callbacks capture Weak context references, not reference cycles.

`touch_events.rs` implements per-touch start/move/end checkpoints. Move dispatch occurs before committing the captured record. A nested start can replace the active map entry without redirecting the outer commit. End snapshots active-touch count before its loop, dispatches TAP/CONTEXT_MENU and END, then deletes the current ID property, even if a callback replaced it. Unknown-touch warning strings retain the original reversed wording.

`touch_dom.rs` creates actual CustomEvent objects with the original event names, including `-xterm-gesturesend`. It preserves field presence, null versus undefined initial targets, non-bubbling/cancelable flags, and fractional numeric coordinates. Touch event lists follow the source's structural property contract instead of imposing an extra instanceof check.

Normal dispatch updates the tap state before ignored-target filtering, walks the source's live linked-list registrations, builds a containing-target snapshot and stable-sorts by increasing ancestor depth. The private `touch_targets.rs` list preserves a generator's next-link behavior when containment callbacks append or remove registrations. Previous links are Weak, and uniquely owned next-chains drop iteratively to avoid reference cycles and recursive cleanup. This list is needed because a Vec snapshot and a RefCell guard across overridable `contains` calls would change source reentry. It holds no record/list borrow across DOM host callbacks. Event cancellation happens after the touch batch, and the dispatched flag clears only after preventDefault and stopPropagation return. Inertia bypasses normal routing and tap state, dispatches its original insertion-order target snapshot and emits the final zero-displacement frame.

Source clocks remain distinct. TAP reads `new Date().getTime()`, while touch batches, hold calculations and inertia frames read Date.now. All host clock calls occur before taking RefCell guards, permitting clock hooks to reenter without a borrow panic. Native policy retains nonfinite and backwards number behavior; it does not clamp velocities or spin an animation until completion.

## Tests and current evidence

The browser component target is `crates/xterm-web/tests/wasm_touch.rs`, with focused callback-reentry cases in `tests/cases/touch_reentry.rs` and DOM/clock fixtures in `tests/support/touch_dom.rs`. Eleven tests cover:

- Nested, duplicate and ignored target registration, exact fields and flags, and fractional coordinates.
- Hold/tap boundaries, ignored tap timestamps, and use of the last move rather than touchend coordinates.
- Insertion-order inertial routing, ignore bypass, final zero frames and cancellation through the shared frame queue.
- Multi-touch end using the original active-count snapshot.
- Token removal during dispatch and repeated context disposal.
- Reentrant start replacing the old movement record and reentrant END replacement deletion.
- Cancellation-method reentry while the outer dispatched flag is still set.
- A nested tap during inertia whose timestamp survives later inertial changes.
- Reentrant containment callbacks whose new target or ignore-target registration participates in the same live source iteration.

The tests dispatch real DOM events into the Rust listeners. Touch-list objects and a deterministic Date/rAF host are controlled fixtures. They are not physical touch-hardware tests. The shared Rust Dom animation queue still executes the frame callbacks. No JavaScript terminal engine is loaded by the component harness.

Run after resource clearance:

```sh
WASM_TEST=wasm_touch node scripts/browser-host-check.mjs
cargo clippy -p xterm-web --target wasm32-unknown-unknown --test wasm_touch -- -D warnings
```

The harness runs Chromium, Firefox and WebKit, shuts down its browsers/server and retains per-test/run receipts under `results/browser-host/wasm_touch/`. Two pre-pause attempts failed while compiling unrelated accessibility/runtime integrations. Neither reached browser execution. The current eleven-test target remains unrun until a successful retained receipt exists. Local builds are paused for coordinated disk recovery; no passing browser result is inferred.

Independent offline source review by Ashley found three issues in the first implementation: cancellation flag clearing before host calls, the wrong TAP clock and distinct unsupported-device no-op identities. All three were corrected, with callback-order regressions added. Portable task 139 has independent native debug/release execution over 639 original-source traces and 8,061 snapshots. Those native results do not close this DOM target or real gesture-to-scroll integration.

Task 141 must demonstrate the emitted gestures driving the actual Rust terminal's mouse/viewport paths. Native Linux/macOS, hardware-specific touch behavior and the complete terminal browser matrix remain separate gates. Overall equivalence is NOT PROVEN.
