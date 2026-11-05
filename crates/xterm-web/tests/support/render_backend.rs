use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use xterm_core::{
    event::{Emitter, Event},
    lifecycle::Disposable,
};
use xterm_web::{
    api::{CharacterDimensions, CssDimensions, DeviceDimensions, RenderDimensions, Size},
    renderer::service_api::{BrowserService, RedrawRequest, Renderer, VisibilityRenderer},
};
pub(super) type Log = Rc<RefCell<Vec<String>>>;
pub(super) fn number(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}
pub(super) fn position(value: Option<[i64; 2]>) -> String {
    value
        .map(|p| format!("{},{}", p[0], p[1]))
        .unwrap_or_else(|| "-".into())
}
pub(super) fn invoke(host: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let method: js_sys::Function = js_sys::Reflect::get(host, &name.into())?.dyn_into()?;
    let values = js_sys::Array::new();
    for arg in args {
        values.push(arg);
    }
    method.apply(host, &values)
}
pub(super) fn pending(host: &JsValue) -> Result<String, JsValue> {
    Ok(js_sys::JSON::stringify(&invoke(host, "pending", &[])?)?
        .as_string()
        .unwrap())
}
pub(super) struct Browser {
    pub(super) window: web_sys::Window,
    pub(super) pixel_ratio: f64,
    pub(super) dpr: Emitter<f64>,
    pub(super) changed: Emitter<web_sys::Window>,
}
impl BrowserService for Browser {
    fn window(&self) -> web_sys::Window {
        self.window.clone()
    }
    fn dpr(&self) -> Result<f64, JsValue> {
        Ok(self.pixel_ratio)
    }
    fn is_focused(&self) -> Result<bool, JsValue> {
        Ok(true)
    }
    fn on_dpr_change(&self) -> Event<f64> {
        self.dpr.event()
    }
    fn on_window_change(&self) -> Event<web_sys::Window> {
        self.changed.event()
    }
}
pub(super) struct Backend {
    name: &'static str,
    log: Log,
    redraw: Emitter<RedrawRequest>,
    pub(super) dimensions: Cell<RenderDimensions>,
    pub(super) fail_dispose: Cell<bool>,
    pub(super) failure: JsValue,
}
impl Backend {
    pub(super) fn new(name: &'static str, log: &Log) -> Rc<Self> {
        let cell = Size {
            width: 5.0,
            height: 10.0,
        };
        let canvas = Size {
            width: 20.0,
            height: 30.0,
        };
        Rc::new(Self {
            name,
            log: Rc::clone(log),
            redraw: Emitter::new(),
            dimensions: Cell::new(RenderDimensions {
                css: CssDimensions { canvas, cell },
                device: DeviceDimensions {
                    canvas,
                    cell,
                    character: CharacterDimensions {
                        width: 5.0,
                        height: 10.0,
                        left: 0.0,
                        top: 0.0,
                    },
                },
            }),
            fail_dispose: Cell::new(false),
            failure: js_sys::Error::new("backend dispose failed").into(),
        })
    }
    fn record(&self, message: String) {
        self.log
            .borrow_mut()
            .push(format!("{}.{message}", self.name));
    }
    pub(super) fn request(&self, start: f64, end: f64, sync: bool) -> Result<(), JsValue> {
        self.redraw
            .fire(&RedrawRequest { start, end, sync })
            .map_err(|e| js_sys::Error::new(&e.to_string()).into())
    }
}
impl Renderer for Backend {
    fn dimensions(&self) -> RenderDimensions {
        self.dimensions.get()
    }
    fn on_request_redraw(&self) -> Event<RedrawRequest> {
        self.redraw.event()
    }
    fn dispose(&self) -> Result<(), JsValue> {
        self.record("dispose".into());
        if self.fail_dispose.get() {
            return Err(self.failure.clone());
        }
        self.redraw.dispose();
        Ok(())
    }
    fn handle_device_pixel_ratio_change(&self) -> Result<(), JsValue> {
        self.record("dpr".into());
        Ok(())
    }
    fn handle_resize(&self, cols: usize, rows: usize) -> Result<(), JsValue> {
        self.record(format!("resize:{cols}:{rows}"));
        let mut d = self.dimensions.get();
        d.css.canvas.width = cols as f64 * 5.0;
        d.css.canvas.height = rows as f64 * 10.0;
        self.dimensions.set(d);
        Ok(())
    }
    fn handle_char_size_changed(&self) -> Result<(), JsValue> {
        self.record("char".into());
        Ok(())
    }
    fn handle_blur(&self) -> Result<(), JsValue> {
        self.record("blur".into());
        Ok(())
    }
    fn handle_focus(&self) -> Result<(), JsValue> {
        self.record("focus".into());
        Ok(())
    }
    fn handle_selection_changed(
        &self,
        start: Option<[i64; 2]>,
        end: Option<[i64; 2]>,
        column: bool,
    ) -> Result<(), JsValue> {
        self.record(format!(
            "selection:{}:{}:{column}",
            position(start),
            position(end)
        ));
        Ok(())
    }
    fn handle_cursor_move(&self) -> Result<(), JsValue> {
        self.record("cursor".into());
        Ok(())
    }
    fn clear(&self) -> Result<(), JsValue> {
        self.record("clear".into());
        Ok(())
    }
    fn render_rows(&self, start: f64, end: f64) -> Result<(), JsValue> {
        self.record(format!("rows:{}:{}", number(start), number(end)));
        Ok(())
    }
    fn visibility(&self) -> Option<&dyn VisibilityRenderer> {
        Some(self)
    }
}
impl VisibilityRenderer for Backend {
    fn handle_viewport_visibility_change(&self, visible: bool) -> Result<(), JsValue> {
        self.record(format!("visible:{visible}"));
        Ok(())
    }
}
