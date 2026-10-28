#[path = "input_mouse_host.rs"]
mod mouse_host;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Event, HtmlElement, KeyboardEvent, MouseEvent};
use xterm_web::{
    dom::{BrowserDom, DomOptions},
    input::{
        composition_geometry::CursorGeometry,
        dom::{InputBindings, InputHost, InputOptions, OverlayLayout},
        keyboard_dispatch::{KeyboardAction, KeyboardReadHost, KeyboardSnapshot},
    },
};

pub type CustomKeyCallback = Rc<dyn Fn(&KeyboardEvent) -> Result<bool, JsValue>>;

#[derive(Default)]
pub struct Host {
    pub options: Cell<InputOptions>,
    pub keyboard: Cell<KeyboardSnapshot>,
    pub data: RefCell<Vec<(Vec<u16>, bool)>>,
    pub effects: RefCell<Vec<String>>,
    pub selection: RefCell<Vec<u16>>,
    pub veto: Cell<bool>,
    pub custom_key: RefCell<Option<CustomKeyCallback>>,
    pub disabled: Cell<bool>,
    pub mouse: Rc<xterm_core::mouse_state::MouseStateService>,
    pub on_key: RefCell<Option<Rc<dyn Fn()>>>,
    pub data_error: RefCell<Option<JsValue>>,
    pub option_changes: xterm_core::event::Emitter<&'static str>,
}

impl KeyboardReadHost for Host {
    type Error = JsValue;
    fn snapshot(&self) -> Result<KeyboardSnapshot, JsValue> {
        Ok(self.keyboard.get())
    }
}

impl InputHost for Host {
    fn options(&self) -> Result<InputOptions, JsValue> {
        Ok(self.options.get())
    }
    fn custom_key_handler(&self, event: &KeyboardEvent) -> Result<bool, JsValue> {
        let callback = self.custom_key.borrow().clone();
        if let Some(callback) = callback {
            callback(event)
        } else {
            Ok(!self.veto.get())
        }
    }
    fn keyboard_effect(
        &self,
        _: Option<&KeyboardEvent>,
        action: KeyboardAction<'_>,
    ) -> Result<(), JsValue> {
        self.effects.borrow_mut().push(format!("{action:?}"));
        if matches!(action, KeyboardAction::OnKey(_)) {
            let callback = self.on_key.borrow().clone();
            if let Some(callback) = callback {
                callback();
            }
        }
        Ok(())
    }
    fn overlay_layout(&self) -> Result<OverlayLayout, JsValue> {
        Ok(OverlayLayout {
            cursor: CursorGeometry {
                in_viewport: true,
                x: 80,
                y: 2,
                cols: 80,
                cell_width: 9.0,
                cell_height: 18.0,
                overlay_width: 0.0,
                overlay_height: 0.0,
            },
            font_family: "monospace".encode_utf16().collect(),
            font_size: 15.0,
        })
    }
    fn data(&self, data: &[u16], user_input: bool) -> Result<(), JsValue> {
        if let Some(error) = self.data_error.borrow().clone() {
            return Err(error);
        }
        if !self.disabled.get() {
            self.data.borrow_mut().push((data.to_vec(), user_input));
        }
        Ok(())
    }
    fn has_selection(&self) -> Result<bool, JsValue> {
        Ok(!self.selection.borrow().is_empty())
    }
    fn selection_text(&self) -> Result<Vec<u16>, JsValue> {
        Ok(self.selection.borrow().clone())
    }
    fn right_click_select(&self, _: &MouseEvent) -> Result<(), JsValue> {
        self.effects.borrow_mut().push("right-click-select".into());
        Ok(())
    }
    fn sync_textarea(&self) -> Result<(), JsValue> {
        self.effects.borrow_mut().push("sync-textarea".into());
        Ok(())
    }
}

pub struct Setup {
    pub parent: HtmlElement,
    pub dom: BrowserDom,
    pub host: Rc<Host>,
    pub input: InputBindings,
}
impl Setup {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
        let host = Rc::new(Host::default());
        let input = InputBindings::new(&dom, host.clone()).unwrap();
        Self {
            parent,
            dom,
            host,
            input,
        }
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        self.input.dispose().unwrap();
        self.parent.remove();
    }
}

pub fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

pub fn event(class: &str, name: &str, fields: &[(&str, JsValue)]) -> Event {
    let init = js_sys::Object::new();
    js_sys::Reflect::set(&init, &"bubbles".into(), &true.into()).unwrap();
    js_sys::Reflect::set(&init, &"cancelable".into(), &true.into()).unwrap();
    for (key, value) in fields {
        js_sys::Reflect::set(&init, &(*key).into(), value).unwrap();
    }
    let constructor: js_sys::Function = js_sys::Reflect::get(&js_sys::global(), &class.into())
        .unwrap()
        .dyn_into()
        .unwrap();
    let args = js_sys::Array::new();
    args.push(&name.into());
    args.push(&init);
    js_sys::Reflect::construct(&constructor, &args)
        .unwrap()
        .dyn_into()
        .unwrap()
}

pub async fn next_turn() {
    let promise: js_sys::Promise =
        js_sys::Function::new_no_args("return new Promise(resolve => setTimeout(resolve, 0))")
            .call0(&JsValue::NULL)
            .unwrap()
            .dyn_into()
            .unwrap();
    wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
}
