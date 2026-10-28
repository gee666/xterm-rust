# Browser Viewport API

Ashley239/71 owns `crate::viewport::Viewport`, implemented by the shared `ViewportController` and actual task167 `SmoothScrollableElement`.

```rust
Viewport::new(
    root: HtmlElement,
    screen: HtmlElement,
    services: ViewportServices,
) -> Result<Rc<Viewport>, JsValue>

pub struct ViewportServices {
    pub buffers: Rc<BufferService>,
    pub core: Rc<CoreService>,
    pub mouse: Rc<MouseStateService>,
    pub options: Rc<OptionsService>,
    pub theme: Rc<ThemeService>,
    pub render: Rc<RenderService>,
    pub main_document: Document,
    pub queue_source: scroll::browser_scheduler::QueueSource,
    pub environment: Rc<ScrollEnvironment>,
}
```

`queue_source` reads the current CoreBrowserService window and obtains that window's shared RuntimeContext animation queue on each animation request. `main_document` is the source CoreBrowserService's main document for style creation, not inferred from an arbitrary node. The shared environment must come from the same BrowserRuntime as other scroll elements.

Public methods all return `Result<(), JsValue>` except the event getter:

- `queue_sync(self: &Rc<Self>, row: Option<f64>)`
- `sync(&self, row: Option<f64>)`
- `scroll_lines(&self, displacement: f64)`
- `scroll_to_line(&self, line: f64, disable_smooth: bool)`
- `handle_touch_scroll(&self, translation_y: f64)`
- `dispose(&self)`
- `on_request_scroll_lines(&self) -> Event<f64>`

Joyce owns `terminal/viewport_services.rs` central method glue. `BrowserServices` retains an `Rc<Viewport>` created during real open. `scroll_lines` routes to Viewport then refreshes all rows. The Viewport request callback bypasses Viewport and calls the actual BufferService with suppress=false, then refreshes. Do not recursively call the central viewport entry from that callback. Before open, use only the source-defined native CoreTerminal fallback. After open, missing Viewport is an initialization error, not successful scrolling.

Xavier owns the Root BrowserServices field/constructor assembly and provides current RenderService/metrics/CoreBrowserService. Only the approved viewport seam is edited by collaborators.
