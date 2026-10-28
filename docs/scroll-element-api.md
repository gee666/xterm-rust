# Scrollable element API for Viewport

Batu owns task167 `scroll/element.rs` and companion element modules. Ashley owns Viewport239/71 and `terminal/viewport_services.rs`. Existing portable Scrollable remains Norman's161.

```rust
SmoothScrollableElement::new(
    screen: HtmlElement,
    options: JsValue,
    model: BrowserScrollable,
    environment: Rc<ScrollEnvironment>,
) -> Result<Rc<Self>, JsValue>
```

The shared environment is mandatory because source wheel classification and iframe cache are shared between terminals. It is not a new model instance. `new_owned` accepts screen/options/environment plus the shared WindowAnimationQueue and creates the source integer model with zero smooth duration, mutating caller options.mouseWheelSmoothScroll to false before resolving options.

Methods:

- `get_dom_node() -> Result<HtmlElement, JsValue>`
- `on_scroll() -> Event<ScrollEvent>`
- `get_scroll_position() -> state::Position`
- `get_scroll_dimensions() -> state::ScrollState`
- `set_scroll_dimensions(DimensionUpdate) -> Result<(), JsValue>`
- `set_scroll_position(PositionUpdate, reuse_animation: bool) -> Result<(), JsValue>`
- `update_options(&JsValue) -> Result<(), JsValue>`
- `update_class_name(OptionValue) -> Result<(), JsValue>`
- `delegate_scroll_from_mouse_wheel_event(&Event) -> Result<(), JsValue>`
- `render_now() -> Result<(), JsValue>`
- `dispose() -> Result<(), JsValue>`

Position uses left/top fields; dimensions use width/height/scroll_width/scroll_height. The Viewport callback from element.on_scroll calls the actual shared BufferService directly, not BrowserServices.scroll_lines, which delegates back to Viewport. No feedback loop or fallback implementation is intended.
