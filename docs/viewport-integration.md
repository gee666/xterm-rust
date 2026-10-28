# Viewport integration ownership

Ashley owns tasks 71/239, `src/viewport*.rs`, their tests, oracle and `terminal/viewport_host.rs`. Norman retains portable model161; Batu owns Element167. Joyce owns `terminal/viewport_services.rs`. These files are under active implementation. Coordinate new root exports and web-sys features with Xavier, who owns the crate root and manifest. The detailed source matrix is in `docs/selection-acceptance.md`, criteria 30 and 261-266. Preserve all 14 upstream scrollable modules as mapped behavior, not just a DOM scrollTop wrapper.

## Callers waiting for the real implementation

Xavier's `terminal/browser_services.rs` owns the opened browser service graph. Norman's `terminal/input_services.rs` delegates the normal-buffer touch branch to `BrowserServices::handle_touch_scroll(translation_y)`. That method must call the actual Viewport instance. It must not return successful no-op or substitute alternate-buffer cursor keys for normal scrolling.

Suggested public names follow the source, with fallible Rust browser returns:

- `queue_sync(ydisp: Option<f64>)`
- `scroll_lines(displacement: f64)`
- `scroll_to_line(line: f64, disable_smooth_scroll: bool)`
- `handle_touch_scroll(translation_y: f64)`
- `dispose()`

The actual constructor and live service fields are in `docs/viewport-browser-api.md`; Element's constructor is in `docs/scroll-element-api.md`. Options/theme updates use the real service events, not construction snapshots. Renderer dimensions must remain live. Do not hardcode a cell height or infer it from terminal row count.

BrowserServices should retain an Rc<Viewport> or an equivalent cloneable controller and release its RefCell borrow before calling viewport methods. Source scroll callbacks can reenter the terminal. Buffer/line/model borrows must not cross user callbacks.

## Required event wiring

- Buffer resize and activation queue synchronization. Activation clears the cached display row before syncing.
- Buffer scroll synchronizes current display position. External display changes stop smooth animation as source does.
- Both public scroll and InputHandler's internal scroll refresh selection and queue viewport sync. These are distinct events. ED3 can require the internal event without a public onScroll.
- `TerminalEvent::InputScroll(usize)` is being coordinated by Max and Coleman for that internal source path. Xavier owns the browser event bridge.
- Render after synchronized output flushes the deferred viewport position. During synchronized output no premature DOM movement occurs.
- Protocol changes alter whether wheel scrolling is handled. Options/theme changes update scrollbar/sensitivity/colors.
- Dispose removes DOM nodes and subscriptions and cancels animation, gesture and drag jobs.

The source's viewport scroll event rounds pixel position to a row, requests its delta from current ydisp, and suppresses dimension-change feedback. Scroll callbacks must not echo another scroll recursively. Smooth scrolling, scrollbar geometry and wheel normalization need deterministic source comparisons in addition to visible browser assertions.
