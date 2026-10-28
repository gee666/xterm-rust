//! Reuses the input test host for keyboard/clipboard contracts. Mouse state is
//! controlled separately so DOM listener, veto and disposal effects are asserted.
#[path = "input_dom.rs"]
pub mod input;
use input::Host as InputTestHost;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsValue;
use web_sys::{HtmlElement, KeyboardEvent, MouseEvent, WheelEvent};
use xterm_core::{
    lifecycle::{to_disposable, DisposableHandle},
    mouse_state::MouseStateService,
};
use xterm_web::input::{
    dom::{
        mouse::{MouseHost, MouseReadHost, MouseSnapshot},
        InputHost, InputOptions, OverlayLayout,
    },
    keyboard_dispatch::{KeyboardAction, KeyboardReadHost, KeyboardSnapshot},
    mouse_policy::{MouseOutput, MouseRouting},
    mouse_wheel::WheelOptions,
};

type WheelHook = Rc<dyn Fn(&WheelEvent)>;

pub struct Host {
    pub input: InputTestHost,
    pub option_changes: xterm_core::event::Emitter<&'static str>,
    pub snapshot: Cell<MouseSnapshot>,
    pub fail_snapshot: Cell<bool>,
    pub fail_read: Cell<Option<&'static str>>,
    pub reads: RefCell<Vec<&'static str>>,
    pub trace: Rc<RefCell<Vec<String>>>,
    pub veto: Cell<bool>,
    pub force_selection: Cell<bool>,
    pub selection_enabled: Cell<bool>,
    pub wheel_calls: Cell<usize>,
    pub wheel_hook: RefCell<Option<WheelHook>>,
    pub focus_calls: Cell<usize>,
    pub touch_scroll: RefCell<Vec<f64>>,
    pub gesture_disposals: Rc<Cell<usize>>,
}
impl Host {
    pub fn new() -> Self {
        Self {
            input: InputTestHost::default(),
            option_changes: xterm_core::event::Emitter::new(),
            snapshot: Cell::new(MouseSnapshot {
                routing: MouseRouting {
                    cols: 80.0,
                    rows: 24.0,
                    require_alt: false,
                    device_cell_height: Some(18.0),
                    dpr: Some(1.0),
                    wheel_options: WheelOptions::default(),
                },
                valid_char_size: true,
                css_cell_width: 9.0,
                css_cell_height: 18.0,
                css_canvas_width: 720.0,
                css_canvas_height: 432.0,
                has_scrollback: true,
                application_cursor: false,
            }),
            fail_snapshot: Cell::new(false),
            fail_read: Cell::new(None),
            reads: RefCell::new(Vec::new()),
            trace: Rc::new(RefCell::new(Vec::new())),
            veto: Cell::new(false),
            force_selection: Cell::new(false),
            selection_enabled: Cell::new(true),
            wheel_calls: Cell::new(0),
            wheel_hook: RefCell::new(None),
            focus_calls: Cell::new(0),
            touch_scroll: RefCell::new(Vec::new()),
            gesture_disposals: Rc::new(Cell::new(0)),
        }
    }
    fn read(&self, key: &'static str) -> Result<MouseSnapshot, JsValue> {
        self.reads.borrow_mut().push(key);
        self.trace.borrow_mut().push(format!("host:{key}"));
        if self.fail_snapshot.get() {
            return Err(js_sys::Error::new("snapshot getter failed").into());
        }
        if self.fail_read.get() == Some(key) {
            return Err(js_sys::Error::new(&format!("unexpected mouse read: {key}")).into());
        }
        Ok(self.snapshot.get())
    }
}
impl MouseReadHost for Host {
    fn mouse_snapshot(&self) -> Result<MouseSnapshot, JsValue> {
        Err(js_sys::Error::new("observer forbids aggregate mouse snapshot").into())
    }
    fn mouse_require_alt(&self) -> Result<bool, JsValue> {
        Ok(self.read("require-alt")?.routing.require_alt)
    }
    fn mouse_has_scrollback(&self) -> Result<bool, JsValue> {
        Ok(self.read("has-scrollback")?.has_scrollback)
    }
    fn mouse_application_cursor(&self) -> Result<bool, JsValue> {
        Ok(self.read("application-cursor")?.application_cursor)
    }
    fn mouse_cols(&self) -> Result<f64, JsValue> {
        Ok(self.read("cols")?.routing.cols)
    }
    fn mouse_rows(&self) -> Result<f64, JsValue> {
        Ok(self.read("rows")?.routing.rows)
    }
    fn mouse_scroll_sensitivity(&self) -> Result<f64, JsValue> {
        Ok(self.read("slow")?.routing.wheel_options.scroll_sensitivity)
    }
    fn mouse_fast_scroll_sensitivity(&self) -> Result<f64, JsValue> {
        Ok(self
            .read("fast")?
            .routing
            .wheel_options
            .fast_scroll_sensitivity)
    }
    fn mouse_device_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(self.read("device-height")?.routing.device_cell_height)
    }
    fn mouse_dpr(&self) -> Result<Option<f64>, JsValue> {
        Ok(self.read("dpr")?.routing.dpr)
    }
    fn mouse_valid_char_size(&self) -> Result<bool, JsValue> {
        Ok(self.read("valid")?.valid_char_size)
    }
    fn mouse_canvas_size(&self) -> Result<[f64; 2], JsValue> {
        let s = self.read("canvas")?;
        Ok([s.css_canvas_width, s.css_canvas_height])
    }
    fn mouse_cell_size(&self) -> Result<[f64; 2], JsValue> {
        let s = self.read("cell")?;
        Ok([s.css_cell_width, s.css_cell_height])
    }
    fn mouse_css_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(self.read("css-height")?.css_cell_height))
    }
}

impl MouseHost for Host {
    fn mouse_service(&self) -> Result<Rc<MouseStateService>, JsValue> {
        Ok(self.input.mouse.clone())
    }
    fn mouse_protocol_log(
        &self,
        events: xterm_core::mouse_state::MouseEventTypes,
    ) -> Result<(), JsValue> {
        self.input
            .effects
            .borrow_mut()
            .push(format!("mouse-protocol:{}", events.0));
        Ok(())
    }
    fn mouse_option_changes(&self) -> xterm_core::event::Event<&'static str> {
        self.option_changes.event()
    }
    fn allow_custom_wheel(&self, event: &WheelEvent) -> Result<bool, JsValue> {
        self.wheel_calls.set(self.wheel_calls.get() + 1);
        self.trace.borrow_mut().push("veto".into());
        let hook = self.wheel_hook.borrow().clone();
        if let Some(hook) = hook {
            hook(event);
        }
        Ok(!self.veto.get())
    }
    fn mouse_focus(&self) -> Result<(), JsValue> {
        self.focus_calls.set(self.focus_calls.get() + 1);
        Ok(())
    }
    fn should_force_selection(&self, _: &MouseEvent) -> Result<bool, JsValue> {
        Ok(self.force_selection.get())
    }
    fn set_mouse_selection_enabled(&self, enabled: bool) -> Result<(), JsValue> {
        self.selection_enabled.set(enabled);
        Ok(())
    }
    fn mouse_output(&self, output: MouseOutput) -> Result<(), JsValue> {
        self.trace.borrow_mut().push(
            match &output {
                MouseOutput::Binary(_) => "binary",
                MouseOutput::UserData(_) => "data",
            }
            .into(),
        );
        match output {
            MouseOutput::Binary(data) => self.input.data.borrow_mut().push((data, false)),
            MouseOutput::UserData(data) => self.input.data.borrow_mut().push((data, true)),
        }
        Ok(())
    }
    fn mouse_touch_scroll(&self, y: f64) -> Result<(), JsValue> {
        self.touch_scroll.borrow_mut().push(y);
        Ok(())
    }
    fn mouse_gesture_target(&self, _: &HtmlElement) -> Result<DisposableHandle, JsValue> {
        // This collaborator checks the consumer's ownership, not touch recognition.
        let count = self.gesture_disposals.clone();
        Ok(to_disposable(move || count.set(count.get() + 1)))
    }
}
impl KeyboardReadHost for Host {
    type Error = JsValue;
    fn snapshot(&self) -> Result<KeyboardSnapshot, JsValue> {
        self.input.snapshot()
    }
}

impl InputHost for Host {
    fn options(&self) -> Result<InputOptions, JsValue> {
        self.input.options()
    }
    fn custom_key_handler(&self, event: &KeyboardEvent) -> Result<bool, JsValue> {
        self.input.custom_key_handler(event)
    }
    fn keyboard_effect(
        &self,
        event: Option<&KeyboardEvent>,
        action: KeyboardAction<'_>,
    ) -> Result<(), JsValue> {
        self.input.keyboard_effect(event, action)
    }
    fn overlay_layout(&self) -> Result<OverlayLayout, JsValue> {
        self.input.overlay_layout()
    }
    fn data(&self, data: &[u16], user_input: bool) -> Result<(), JsValue> {
        self.input.data(data, user_input)
    }
    fn has_selection(&self) -> Result<bool, JsValue> {
        self.input.has_selection()
    }
    fn selection_text(&self) -> Result<Vec<u16>, JsValue> {
        self.input.selection_text()
    }
    fn sync_textarea(&self) -> Result<(), JsValue> {
        self.input.sync_textarea()
    }
    fn right_click_select(&self, event: &MouseEvent) -> Result<(), JsValue> {
        self.input.right_click_select(event)
    }
}
