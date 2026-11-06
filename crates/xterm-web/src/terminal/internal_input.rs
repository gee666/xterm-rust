//! Internal input operations use the same state as parsing and image handlers.
use super::internal_parser::InternalParser;
use crate::js_error::to_js_error;
use js_sys::Function;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::{
    headless,
    terminal::{AttributeView, TerminalState},
};

#[wasm_bindgen]
pub struct InternalInputHandler {
    native: Rc<headless::Terminal>,
    parser: JsValue,
    dirty: JsValue,
    attributes: RefCell<Option<(AttributeView, JsValue)>>,
    reset: Function,
}
impl InternalInputHandler {
    pub(super) fn new(native: Rc<headless::Terminal>) -> Result<Self, JsValue> {
        let registrations = native
            .with_parser(|parser| parser.registrations())
            .map_err(|error| to_js_error(&error))?;
        let event = native.on_event();
        let reset = Closure::wrap(Box::new(move |listener: Function| {
            let handle = event.try_subscribe(move |value| {
                if *value == xterm_core::terminal::TerminalEvent::RequestReset {
                    listener
                        .call0(&JsValue::UNDEFINED)
                        .map_err(crate::js_error::callback_error)?;
                }
                Ok(())
            });
            xterm_wasm_support::events::disposable(handle)
        }) as Box<dyn Fn(Function) -> JsValue>)
        .into_js_value()
        .unchecked_into();
        Ok(Self {
            parser: InternalParser::new(registrations).into(),
            dirty: InternalDirtyRows {
                state: native.state_handle_for_addon(),
            }
            .into(),
            native,
            attributes: RefCell::new(None),
            reset,
        })
    }
}
#[wasm_bindgen]
impl InternalInputHandler {
    #[wasm_bindgen(getter, js_name = _parser)]
    pub fn parser(&self) -> JsValue {
        self.parser.clone()
    }
    #[wasm_bindgen(getter, js_name = _dirtyRowTracker)]
    pub fn dirty(&self) -> JsValue {
        self.dirty.clone()
    }
    #[wasm_bindgen(getter, js_name = _curAttrData)]
    pub fn attributes(&self) -> Result<JsValue, JsValue> {
        let view = AttributeView::capture(self.native.state_handle_for_addon())
            .map_err(|error| to_js_error(&error))?;
        if let Some((previous, object)) = self.attributes.borrow().as_ref() {
            if previous.same_object(&view) {
                return Ok(object.clone());
            }
        }
        let object: JsValue = InternalAttributes { view: view.clone() }.into();
        let previous = self.attributes.replace(Some((view, object.clone())));
        drop(previous);
        Ok(object)
    }
    #[wasm_bindgen(js_name = getAttrData)]
    pub fn get_attributes(&self) -> Result<JsValue, JsValue> {
        self.attributes()
    }
    #[wasm_bindgen(getter, js_name = onRequestReset)]
    pub fn on_request_reset(&self) -> Function {
        self.reset.clone()
    }
    #[wasm_bindgen(js_name = lineFeed)]
    pub fn line_feed(&self) -> Result<bool, JsValue> {
        xterm_core::terminal::line_feed_state(&self.native.state_handle_for_addon())
            .map_err(|error| to_js_error(&error))?;
        Ok(true)
    }
}

/// Read-only retained AttributeData identity. Writable raw fields remain separate.
#[wasm_bindgen(js_name = _AttributeData)]
pub struct InternalAttributes {
    view: AttributeView,
}
#[wasm_bindgen(js_class = _AttributeData)]
impl InternalAttributes {
    #[wasm_bindgen(getter)]
    pub fn fg(&self) -> Result<u32, JsValue> {
        Ok(self.view.read().map_err(|e| to_js_error(&e))?.fg)
    }
    #[wasm_bindgen(getter)]
    pub fn bg(&self) -> Result<u32, JsValue> {
        Ok(self.view.read().map_err(|e| to_js_error(&e))?.bg)
    }
    #[wasm_bindgen(js_name = isInverse)]
    pub fn is_inverse(&self) -> Result<u32, JsValue> {
        Ok(self.view.read().map_err(|e| to_js_error(&e))?.is_inverse())
    }
    #[wasm_bindgen(js_name = getFgColorMode)]
    pub fn fg_mode(&self) -> Result<u32, JsValue> {
        Ok(self
            .view
            .read()
            .map_err(|e| to_js_error(&e))?
            .get_fg_color_mode())
    }
    #[wasm_bindgen(js_name = getBgColorMode)]
    pub fn bg_mode(&self) -> Result<u32, JsValue> {
        Ok(self
            .view
            .read()
            .map_err(|e| to_js_error(&e))?
            .get_bg_color_mode())
    }
    #[wasm_bindgen(js_name = getFgColor)]
    pub fn fg_color(&self) -> Result<i32, JsValue> {
        Ok(self
            .view
            .read()
            .map_err(|e| to_js_error(&e))?
            .get_fg_color())
    }
    #[wasm_bindgen(js_name = getBgColor)]
    pub fn bg_color(&self) -> Result<i32, JsValue> {
        Ok(self
            .view
            .read()
            .map_err(|e| to_js_error(&e))?
            .get_bg_color())
    }
}

#[wasm_bindgen]
pub struct InternalDirtyRows {
    state: Rc<RefCell<TerminalState>>,
}
#[wasm_bindgen]
impl InternalDirtyRows {
    #[wasm_bindgen(js_name = markDirty)]
    pub fn mark_dirty(&self, row: usize) -> Result<(), JsValue> {
        xterm_core::terminal::mark_dirty_state(&self.state, row).map_err(|e| to_js_error(&e))
    }
    #[wasm_bindgen(js_name = markRangeDirty)]
    pub fn mark_range(&self, first: usize, last: usize) -> Result<(), JsValue> {
        self.mark_dirty(first)?;
        self.mark_dirty(last)
    }
    #[wasm_bindgen(js_name = markAllDirty)]
    pub fn mark_all(&self) -> Result<(), JsValue> {
        let last = self.state.try_borrow().map_err(|e| to_js_error(&e))?.rows() - 1;
        self.mark_range(0, last)
    }
    #[wasm_bindgen(js_name = clearRange)]
    pub fn clear(&self) -> Result<(), JsValue> {
        self.state
            .try_borrow_mut()
            .map_err(|e| to_js_error(&e))?
            .clear_pending_dirty_range()
            .map_err(|e| to_js_error(&e))?;
        Ok(())
    }
    #[wasm_bindgen(getter)]
    pub fn start(&self) -> Result<usize, JsValue> {
        Ok(self
            .state
            .try_borrow()
            .map_err(|e| to_js_error(&e))?
            .pending_dirty_range()
            .map_err(|e| to_js_error(&e))?
            .0)
    }
    #[wasm_bindgen(getter)]
    pub fn end(&self) -> Result<usize, JsValue> {
        Ok(self
            .state
            .try_borrow()
            .map_err(|e| to_js_error(&e))?
            .pending_dirty_range()
            .map_err(|e| to_js_error(&e))?
            .1)
    }
}
