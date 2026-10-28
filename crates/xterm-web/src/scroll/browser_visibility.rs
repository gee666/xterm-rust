// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Browser host for scrollbarVisibilityController.ts.

use super::fast_dom_node::FastDomNode;
use super::visibility::{VisibilityController, VisibilityError};
use super::ScrollbarVisibility;
use crate::links::browser_debounce::BrowserTimerScheduler;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;

type Dispatch = Rc<dyn Fn() -> Result<(), JsValue>>;
type TimerCallback = Closure<dyn Fn() -> Result<(), JsValue>>;
struct State {
    controller: VisibilityController<BrowserTimerScheduler>,
    _callback: TimerCallback,
}
impl Drop for State {
    fn drop(&mut self) {
        if let Err(error) = self.controller.dispose() {
            web_sys::console::error_1(&to_js(error));
        }
    }
}

/// Owns the callback allocation as long as its timer host can reference it.
/// The timeout Window is the source module's ambient window, not necessarily the
/// node's ownerDocument window. The caller supplies the correct module realm.
#[derive(Clone)]
pub struct BrowserVisibilityController {
    state: Rc<State>,
}
pub type BrowserVisibility = BrowserVisibilityController;

pub fn new_visibility(
    window: Window,
    visibility: ScrollbarVisibility,
    visible_class: impl Into<String>,
    invisible_class: impl Into<String>,
) -> Result<BrowserVisibility, JsValue> {
    BrowserVisibilityController::new(window, visibility, visible_class, invisible_class)
}

impl BrowserVisibilityController {
    pub fn new(
        window: Window,
        visibility: ScrollbarVisibility,
        visible_class: impl Into<String>,
        invisible_class: impl Into<String>,
    ) -> Result<Self, JsValue> {
        let slot: Rc<RefCell<Option<Dispatch>>> = Rc::new(RefCell::new(None));
        let active = Rc::clone(&slot);
        let callback: TimerCallback = Closure::wrap(Box::new(move || {
            let dispatch = active.borrow().clone();
            if let Some(dispatch) = dispatch {
                dispatch()?;
            }
            Ok(())
        }));
        let scheduler = BrowserTimerScheduler::new(
            window,
            callback
                .as_ref()
                .unchecked_ref::<js_sys::Function>()
                .clone(),
        )?;
        let controller = VisibilityController::new(
            scheduler,
            visibility,
            visible_class.into(),
            invisible_class.into(),
        );
        let dispatch = controller.weak_dispatcher();
        *slot.borrow_mut() = Some(Rc::new(move || dispatch().map_err(to_js)));
        Ok(Self {
            state: Rc::new(State {
                controller,
                _callback: callback,
            }),
        })
    }
    pub fn set_dom_node(&self, node: FastDomNode) -> Result<(), JsValue> {
        self.state
            .controller
            .set_dom_node(move |name| node.set_class_name(name))
            .map_err(to_js)
    }
    pub fn set_visibility(&self, visibility: ScrollbarVisibility) -> Result<(), JsValue> {
        self.state
            .controller
            .set_visibility(visibility)
            .map_err(to_js)
    }
    pub fn set_should_be_visible(&self, visible: bool) -> Result<(), JsValue> {
        self.state
            .controller
            .set_should_be_visible(visible)
            .map_err(to_js)
    }
    pub fn set_is_needed(&self, needed: bool) -> Result<(), JsValue> {
        self.state.controller.set_is_needed(needed).map_err(to_js)
    }
    pub fn ensure_visibility(&self) -> Result<(), JsValue> {
        self.state.controller.ensure_visibility().map_err(to_js)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.controller.dispose().map_err(to_js)
    }
}
fn to_js(error: VisibilityError<JsValue>) -> JsValue {
    match error {
        VisibilityError::Host(error) => error,
        VisibilityError::DisposedTimer => {
            js_sys::Error::new("Calling setIfNotSet on a disposed TimeoutTimer").into()
        }
    }
}
