# Touch implementation contract

Task 108 owns `input/touch_dom.rs`, `touch_events.rs`, `touch_animation.rs` and DOM integration. The portable policy helper in `input/touch.rs` is delegated to Ashley with tests/oracle. Source is the complete `source_repo/src/browser/scrollable/touch.ts`; edge notes are retained in `tmp/touch-source-notes.md`.

Portable API agreed for DOM integration:

```rust
pub struct TouchData<T> { pub initial_target: T /* private rolling data */ }
impl<T> TouchData<T> {
    pub fn new(target: T, now: f64, page_x: f64, page_y: f64) -> Self;
    pub fn last(&self) -> (f64, f64);
    pub fn translation(&self, page_x: f64, page_y: f64) -> (f64, f64);
    pub fn commit_move(&mut self, now: f64, page_x: f64, page_y: f64);
    pub fn end(&self, hold_now: f64, inertia_start: f64) -> EndGesture;
}
pub enum EndGesture { Tap, ContextMenu, Inertia(Inertia) }
pub struct Inertia { /* exact source velocity/direction/timestamp */ }
impl Inertia {
    pub fn step(&mut self, now: f64) -> InertiaFrame;
}
pub struct InertiaFrame {
    pub translation_x: f64,
    pub translation_y: f64,
    pub continues: bool,
}
```

Times use Date.now-compatible f64 values, including backward or equal timestamps. Preserve four samples, strict distance/hold boundaries, NaN/Infinity arithmetic, friction and the final zero-change frame. `end` computes stationary versus inertia only; DOM selects inertia only when the active count snapshot equals one. DOM dispatches move before calling `commit_move`, preserving the captured old record across reentrant map replacement.

WASM API is `input::touch::GestureContext::new(document: Document) -> Result<Rc<Self>, JsValue>`. `add_target(self: &Rc<Self>, element: &HtmlElement)` and `ignore_target` return `Result<DisposableHandle, JsValue>`. RuntimeContext owns one shared context per document. Tokens remove only their registration, including duplicate registrations. Context listeners and frames capture Weak references, not strong cycles. No mutable global singleton.

Normal dispatch checks ignore targets, collects containing targets and stable-sorts by depth before invoking a snapshot. No RefCell borrow survives event dispatch. Tap state changes before ignore filtering. Inertia uses its original insertion-order target snapshot and bypasses normal routing/cancellation state. `dispose` cancels the frame and removes owned listeners; source target lists and active records are not silently reset by a token removal.

Original touch-specific test suites have not been found. Do not invent source test counts. Add deterministic native policy/original-JS comparisons and actual Rust-backed browser tests for field presence, event flags, nested targets, ignore targets, duplicates, mutation/reentry, cancellation and real gesture-to-scroll integration.
