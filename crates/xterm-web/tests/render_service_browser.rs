#![cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::{
    event::{Emitter, Event},
    headless::Terminal,
    lifecycle::Disposable,
};
use xterm_web::{
    api::{CharacterDimensions, CssDimensions, DeviceDimensions, RenderDimensions, Size},
    char_size_browser::CharSizeService,
    renderer::{
        service::RenderService,
        service_api::{BrowserService, RedrawRequest, Renderer, VisibilityRenderer},
        width_cache::Font,
    },
    theme::ThemeService,
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/render_service_host.mjs")]
extern "C" {
    #[wasm_bindgen(catch,js_name=withRenderHarness)]
    fn with_harness(callback: &js_sys::Function) -> Result<JsValue, JsValue>;
}
type Log = Rc<RefCell<Vec<String>>>;
fn number(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}
fn position(value: Option<[i64; 2]>) -> String {
    value
        .map(|p| format!("{},{}", p[0], p[1]))
        .unwrap_or_else(|| "-".into())
}
fn invoke(host: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let method: js_sys::Function = js_sys::Reflect::get(host, &name.into())?.dyn_into()?;
    let values = js_sys::Array::new();
    for arg in args {
        values.push(arg);
    }
    method.apply(host, &values)
}
fn pending(host: &JsValue) -> Result<String, JsValue> {
    Ok(js_sys::JSON::stringify(&invoke(host, "pending", &[])?)?
        .as_string()
        .unwrap())
}
struct Browser {
    window: web_sys::Window,
    dpr: Emitter<f64>,
    changed: Emitter<web_sys::Window>,
}
impl BrowserService for Browser {
    fn window(&self) -> web_sys::Window {
        self.window.clone()
    }
    fn dpr(&self) -> Result<f64, JsValue> {
        Ok(1.0)
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
struct Backend {
    name: &'static str,
    log: Log,
    redraw: Emitter<RedrawRequest>,
    dimensions: Cell<RenderDimensions>,
    fail_dispose: Cell<bool>,
    failure: JsValue,
}
impl Backend {
    fn new(name: &'static str, log: &Log) -> Rc<Self> {
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
    fn request(&self, start: f64, end: f64, sync: bool) -> Result<(), JsValue> {
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
fn run_trace(host: JsValue) -> Result<JsValue, JsValue> {
    let log: Log = Rc::default();
    let owner = js_sys::Reflect::get(&host, &"window".into())?.unchecked_into();
    let browser = Rc::new(Browser {
        window: owner,
        dpr: Emitter::new(),
        changed: Emitter::new(),
    });
    let terminal = Rc::new(Terminal::new(4, 3, 0).map_err(|e| js_sys::Error::new(&e.to_string()))?);
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: web_sys::HtmlElement = document.create_element("div")?.unchecked_into();
    let metrics = Rc::new(CharSizeService::new(
        &document,
        &parent,
        Font {
            family: "monospace".into(),
            size: 15.0,
            ..Font::default()
        },
    )?);
    metrics
        .state
        .update_measurement(Some(5.0), Some(10.0))
        .unwrap();
    let decorations = terminal
        .with_state(|state| xterm_core::decoration::DecorationService::with_buffers(&state.buffers))
        .unwrap()
        .unwrap();
    let service = RenderService::new(
        browser,
        parent,
        terminal.clone(),
        Rc::new(ThemeService::default()),
        metrics,
        decorations,
    )?;
    let viewport = log.clone();
    let _viewport = service.on_rendered_viewport_change().subscribe(move |e| {
        viewport
            .borrow_mut()
            .push(format!("viewport:{}:{}", number(e.start), number(e.end)))
    });
    let rendered = log.clone();
    let _rendered = service.on_render().subscribe(move |e| {
        rendered
            .borrow_mut()
            .push(format!("render:{}:{}", number(e.start), number(e.end)))
    });
    let dimensions = log.clone();
    let _dimensions = service.on_dimensions_change().subscribe(move |d| {
        dimensions.borrow_mut().push(format!(
            "dimensions:{}:{}",
            number(d.css.canvas.width),
            number(d.css.canvas.height)
        ))
    });
    let a = Backend::new("A", &log);
    let b = Backend::new("B", &log);
    let c = Backend::new("C", &log);
    service.set_renderer(a.clone())?;
    invoke(&host, "frame", &[])?;
    let frames = log.clone();
    service.add_refresh_callback(move |time| {
        frames.borrow_mut().push(format!("frame:{}", number(time)));
        Ok(())
    })?;
    service.refresh_rows(2.0, 2.0, false, true)?;
    service.refresh_rows(1.0, 1.0, false, false)?;
    invoke(&host, "frame", &[])?;
    a.request(0.0, 0.0, false)?;
    invoke(&host, "frame", &[])?;
    service.set_renderer(a.clone())?;
    invoke(&host, "frame", &[])?;
    a.request(1.0, 1.0, true)?;
    let core = terminal.with_state(|s| s.core.clone()).unwrap();
    core.dec_private_modes.borrow_mut().synchronized_output = true;
    service.refresh_rows(2.0, 2.0, false, false)?;
    invoke(&host, "frame", &[])?;
    core.dec_private_modes.borrow_mut().synchronized_output = false;
    service.refresh_rows(1.0, 1.0, false, false)?;
    core.dec_private_modes.borrow_mut().synchronized_output = true;
    invoke(&host, "frame", &[])?;
    core.dec_private_modes.borrow_mut().synchronized_output = false;
    service.refresh_rows(0.0, 0.0, true, true)?;
    log.borrow_mut()
        .push(format!("pending:{}", pending(&host)?));
    invoke(&host, "visibility", &[false.into()])?;
    service.resize(6, 4)?;
    service.handle_resize(6, 4)?;
    service.resize(7, 5)?;
    service.handle_resize(7, 5)?;
    service.refresh_rows(1.0, 2.0, false, false)?;
    invoke(&host, "frame", &[])?;
    invoke(&host, "ratio", &[1.0.into()])?;
    invoke(&host, "frame", &[])?;
    service.resize(7, 5)?;
    service.resize(7, 5)?;
    service.handle_selection_changed(Some([1, 2]), Some([3, 4]), true)?;
    service.handle_focus()?;
    service.handle_blur()?;
    service.handle_cursor_move()?;
    service.refresh_rows(-2.0, 99.0, true, false)?;
    service.refresh_rows(0.5, 1.5, true, false)?;
    service.refresh_rows(f64::NAN, f64::NAN, true, false)?;
    service.set_renderer(b.clone())?;
    invoke(&host, "frame", &[])?;
    b.fail_dispose.set(true);
    let error = service.set_renderer(c.clone()).unwrap_err();
    assert!(js_sys::Object::is(&error, &b.failure));
    log.borrow_mut().push("dispose-error-same:true".into());
    b.request(2.0, 2.0, true)?;
    b.fail_dispose.set(false);
    core.dec_private_modes.borrow_mut().synchronized_output = true;
    service.refresh_rows(3.0, 3.0, false, false)?;
    invoke(&host, "safety", &[])?;
    invoke(&host, "frame", &[])?;
    assert!(!core.dec_private_modes.borrow().synchronized_output);
    service.refresh_rows(0.0, 0.0, false, false)?;
    core.dec_private_modes.borrow_mut().synchronized_output = true;
    service.refresh_rows(2.0, 2.0, false, false)?;
    service.dispose()?;
    service.set_renderer(c)?;
    log.borrow_mut()
        .push(format!("has-renderer:{}", service.has_renderer()));
    log.borrow_mut()
        .push(format!("pending:{}", pending(&host)?));
    let output = js_sys::Array::new();
    for entry in log.borrow().iter() {
        output.push(&entry.into());
    }
    Ok(output.into())
}
#[wasm_bindgen_test]
fn exact_source_trace_covers_frames_sync_visibility_replacement_errors_and_disposal() {
    let callback =
        Closure::wrap(Box::new(run_trace) as Box<dyn Fn(JsValue) -> Result<JsValue, JsValue>>);
    let actual = with_harness(callback.as_ref().unchecked_ref()).unwrap();
    let actual: Vec<String> = js_sys::Array::from(&actual)
        .iter()
        .map(|v| v.as_string().unwrap())
        .collect();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/render-service.json")).unwrap();
    let expected: Vec<String> = fixture["log"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(actual, expected);
}
