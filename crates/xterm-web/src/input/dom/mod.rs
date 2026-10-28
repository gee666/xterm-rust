//! DOM event ownership for browser input. The coordinator supplies live engine
//! state through InputHost; input is never echoed to the display by this module.

mod clipboard;
mod composition;
mod keyboard;
pub mod mouse;
mod timers;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{HtmlElement, HtmlTextAreaElement};

use super::composition::Composition;
use super::composition_geometry::CursorGeometry;
use super::keyboard_dispatch::{KeyboardAction, KeyboardDispatch, KeyboardReadHost};
use crate::dom::BrowserDom;
use crate::dom_listener::DomListener;
use crate::utf16::{from_js_string, to_js_string};
use timers::TimerQueue;

#[derive(Clone, Copy, Debug, Default)]
pub struct InputOptions {
    pub bracketed_paste: bool,
    pub ignore_bracketed_paste: bool,
    pub right_click_selects_word: bool,
    pub is_firefox: bool,
    pub is_linux: bool,
}

#[derive(Clone, Debug)]
pub struct OverlayLayout {
    pub cursor: CursorGeometry,
    pub font_family: Vec<u16>,
    pub font_size: f64,
}

/// Implemented by the browser terminal coordinator, using its real core and
/// selection/render services. Callbacks may reenter the terminal. No input
/// RefCell borrow is held across these calls. The core's data path enforces
/// disableStdin, user-input ordering and scroll-on-input as upstream does.
pub trait InputHost: mouse::MouseHost + KeyboardReadHost<Error = JsValue> {
    /// Statically owned hosts need no extra lease. Weak engine adapters retain
    /// the source method receiver through callbacks that may dispose terminal.
    fn retain_host(&self) -> Result<Option<Rc<dyn std::any::Any>>, JsValue> {
        Ok(None)
    }
    fn bracketed_paste(&self) -> Result<bool, JsValue>;
    fn ignore_bracketed_paste(&self) -> Result<bool, JsValue>;
    fn right_click_selects_word(&self) -> Result<bool, JsValue>;
    fn is_firefox(&self) -> Result<bool, JsValue>;
    fn is_linux(&self) -> Result<bool, JsValue>;
    fn custom_key_handler(&self, event: &web_sys::KeyboardEvent) -> Result<bool, JsValue>;
    /// Handles selection/scroll/cursor/onKey effects, not local DOM cancellation
    /// or engine data. InputBindings handles those before calling this method.
    fn keyboard_effect(
        &self,
        event: Option<&web_sys::KeyboardEvent>,
        action: KeyboardAction<'_>,
    ) -> Result<(), JsValue>;
    fn overlay_layout(&self) -> Result<OverlayLayout, JsValue>;
    fn data(&self, data: &[u16], user_input: bool) -> Result<(), JsValue>;
    fn has_selection(&self) -> Result<bool, JsValue>;
    fn selection_text(&self) -> Result<Vec<u16>, JsValue>;
    fn sync_textarea(&self) -> Result<(), JsValue>;
    fn right_click_select(&self, event: &web_sys::MouseEvent) -> Result<(), JsValue>;
}

struct State {
    textarea: HtmlTextAreaElement,
    overlay: HtmlElement,
    screen: HtmlElement,
    host: Rc<dyn InputHost>,
    composition: RefCell<Composition>,
    keyboard: KeyboardDispatch,
    timers: TimerQueue,
    disposed: Cell<bool>,
}

/// Own this alongside BrowserDom and dispose it before detaching DOM nodes.
/// Construction either installs every requested listener or cleans up all of
/// the listeners already installed, through RAII.
pub struct InputBindings {
    state: Rc<State>,
    listeners: RefCell<Vec<DomListener>>,
    mouse: RefCell<Option<mouse::MouseBindings>>,
}

impl InputBindings {
    pub fn new(dom: &BrowserDom, host: Rc<dyn InputHost>) -> Result<Self, JsValue> {
        let bindings = Self::new_without_mouse(dom, host)?;
        bindings.bind_mouse(dom)?;
        Ok(bindings)
    }

    /// The terminal stores this owner before binding mouse. Protocol logging
    /// can invoke user callbacks during bind_mouse, including terminal.dispose.
    pub(crate) fn new_without_mouse(
        dom: &BrowserDom,
        host: Rc<dyn InputHost>,
    ) -> Result<Self, JsValue> {
        let window = dom
            .document
            .default_view()
            .ok_or_else(|| js_sys::Error::new("Input requires a document window"))?;
        let state = Rc::new(State {
            textarea: dom.textarea.clone(),
            overlay: dom.composition.clone(),
            screen: dom.screen.clone(),
            host,
            composition: RefCell::new(Composition::default()),
            keyboard: KeyboardDispatch::default(),
            timers: TimerQueue::new(window),
            disposed: Cell::new(false),
        });
        let mut bindings = Self {
            state,
            listeners: RefCell::new(Vec::new()),
            mouse: RefCell::new(None),
        };
        composition::bind(&mut bindings)?;
        keyboard::bind(&mut bindings)?;
        clipboard::bind(&mut bindings, dom)?;
        Ok(bindings)
    }

    pub(crate) fn bind_mouse(&self, dom: &BrowserDom) -> Result<(), JsValue> {
        self.check_live()?;
        if self.mouse.borrow().is_some() {
            return Err(js_sys::Error::new("Mouse input is already bound").into());
        }
        let mouse = mouse::MouseBindings::new(dom, self.state.host.clone())?;
        if self.state.disposed.get() {
            // Match registration in an already-disposed source DisposableStore.
            mouse.dispose()?;
        } else {
            *self.mouse.borrow_mut() = Some(mouse);
        }
        Ok(())
    }

    /// Programmatic paste follows the same engine path as a browser paste.
    pub fn paste(&self, text: &[u16]) -> Result<(), JsValue> {
        self.check_live()?;
        let _host = self.state.host.retain_host()?;
        clipboard::paste(&self.state, text)
    }

    /// Called after a render/layout change while composition is active.
    pub fn update_composition_elements(&self) -> Result<(), JsValue> {
        self.check_live()?;
        let _host = self.state.host.retain_host()?;
        composition::update_elements(&self.state, true)
    }

    pub fn is_composing(&self) -> bool {
        self.state.composition.borrow().is_composing
    }

    pub fn reset_mouse(&self) {
        if let Some(mouse) = self.mouse.borrow().as_ref() {
            mouse.reset();
        }
    }

    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.disposed.set(true);
        let mouse = self.mouse.borrow_mut().take();
        if let Some(mouse) = mouse {
            mouse.dispose()?;
        }
        self.state.timers.clear();
        let listeners = std::mem::take(&mut *self.listeners.borrow_mut());
        for mut listener in listeners {
            listener.dispose()?;
        }
        Ok(())
    }

    fn check_live(&self) -> Result<(), JsValue> {
        if self.state.disposed.get() {
            Err(js_sys::Error::new("Input component is disposed").into())
        } else {
            Ok(())
        }
    }
}

impl Drop for InputBindings {
    fn drop(&mut self) {
        self.state.disposed.set(true);
        self.state.timers.clear();
        // DomListener removes its validated built-in listener during Drop.
    }
}

fn read_value(area: &HtmlTextAreaElement) -> Result<Vec<u16>, JsValue> {
    let value = js_sys::Reflect::get(area.as_ref(), &JsValue::from_str("value"))?;
    Ok(from_js_string(&value.dyn_into::<js_sys::JsString>()?))
}

fn set_value(area: &HtmlTextAreaElement, value: &[u16]) -> Result<(), JsValue> {
    let success = js_sys::Reflect::set(
        area.as_ref(),
        &JsValue::from_str("value"),
        &to_js_string(value),
    )?;
    if !success {
        return Err(js_sys::TypeError::new("Cannot set input textarea value").into());
    }
    Ok(())
}

fn listen(
    bindings: &mut InputBindings,
    target: &web_sys::EventTarget,
    name: &str,
    capture: bool,
    callback: impl Fn(&Rc<State>, web_sys::Event) -> Result<(), JsValue> + 'static,
) -> Result<(), JsValue> {
    let state = Rc::downgrade(&bindings.state);
    bindings
        .listeners
        .borrow_mut()
        .push(DomListener::new_reentrant_fallible(
            target,
            name,
            capture,
            None,
            move |event| {
                if let Some(state) = state.upgrade() {
                    if !state.disposed.get() {
                        let _host = state.host.retain_host()?;
                        return callback(&state, event);
                    }
                }
                Ok(())
            },
        )?);
    Ok(())
}
