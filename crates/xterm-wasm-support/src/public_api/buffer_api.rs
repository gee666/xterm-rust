// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! Live JavaScript buffer views over the Rust buffer namespace.
use super::buffer_cell::WebBufferCell;
use crate::{js_error::to_js_error, utf16::to_js_string};
use js_sys::{Function, JsString, Reflect};
use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use xterm_core::{
    buffer::{BufferLine, LineRef},
    headless::Terminal,
    public_api::{
        line_numbers::{
            combined_at_number, extended_at_number, read_cell_number, read_cell_words_number,
        },
        line_translation::{translate_to_string_number, NumericTranslationError},
        BufferApiView, BufferNamespaceApi, BufferType,
    },
};

// Strict assignment belongs at the JS boundary so proxy/readonly/primitive
// failures retain the engine's actual exception rather than a Rust approximation.
#[wasm_bindgen(
    inline_js = "export function bufferAssignField(target, field, value) { target[field] = value; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = bufferAssignField)]
    fn assign_cell_field(target: &JsValue, field: &str, value: &JsValue) -> Result<(), JsValue>;
}

struct CachedLine {
    native: Weak<RefCell<BufferLine>>,
    object: JsValue,
}

#[wasm_bindgen(js_name = BufferNamespace)]
pub struct WebBufferNamespace {
    native: Rc<BufferNamespaceApi>,
    normal: JsValue,
    alternate: JsValue,
    on_change: Function,
    // Namespace ownership keeps the terminal engine alive, as source _core does.
    _terminal: Rc<Terminal>,
}
impl WebBufferNamespace {
    pub fn new(terminal: Rc<Terminal>) -> Result<Self, JsValue> {
        let native = terminal.buffer().map_err(|error| to_js_error(&error))?;
        let normal: JsValue = WebBufferView::new(native.normal()).into();
        let alternate: JsValue = WebBufferView::new(native.alternate()).into();
        let normal_event = normal.clone();
        let alternate_event = alternate.clone();
        let on_change = super::events::callable(native.on_buffer_change(), move |view| {
            Ok(match view.buffer_type() {
                BufferType::Normal => normal_event.clone(),
                BufferType::Alternate => alternate_event.clone(),
            })
        });
        Ok(Self {
            normal,
            alternate,
            native,
            on_change,
            _terminal: terminal,
        })
    }
}
#[wasm_bindgen(js_class = BufferNamespace)]
impl WebBufferNamespace {
    #[wasm_bindgen(getter)]
    pub fn normal(&self) -> JsValue {
        self.native.normal();
        self.normal.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn alternate(&self) -> JsValue {
        self.native.alternate();
        self.alternate.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn active(&self) -> Result<JsValue, JsValue> {
        let view = self.native.active().map_err(|error| to_js_error(&error))?;
        Ok(match view.buffer_type() {
            BufferType::Normal => self.normal.clone(),
            BufferType::Alternate => self.alternate.clone(),
        })
    }
    #[wasm_bindgen(getter, js_name = onBufferChange)]
    pub fn on_buffer_change_function(&self) -> Function {
        self.on_change.clone()
    }
}

#[wasm_bindgen(js_name = BufferView)]
pub struct WebBufferView {
    native: Rc<BufferApiView>,
    lines: RefCell<HashMap<usize, CachedLine>>,
}
impl WebBufferView {
    fn new(native: Rc<BufferApiView>) -> Self {
        Self {
            native,
            lines: RefCell::new(HashMap::new()),
        }
    }
}
#[wasm_bindgen(js_class = BufferView)]
impl WebBufferView {
    #[wasm_bindgen(getter, js_name = type)]
    pub fn buffer_type(&self) -> String {
        match self.native.buffer_type() {
            BufferType::Normal => "normal",
            BufferType::Alternate => "alternate",
        }
        .into()
    }
    #[wasm_bindgen(getter, js_name = cursorY)]
    pub fn cursor_y(&self) -> Result<usize, JsValue> {
        self.native.cursor_y().map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(getter, js_name = cursorX)]
    pub fn cursor_x(&self) -> Result<usize, JsValue> {
        self.native.cursor_x().map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(getter, js_name = viewportY)]
    pub fn viewport_y(&self) -> Result<usize, JsValue> {
        self.native
            .viewport_y()
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(getter, js_name = baseY)]
    pub fn base_y(&self) -> Result<usize, JsValue> {
        self.native.base_y().map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> Result<usize, JsValue> {
        self.native.len().map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = getNullCell)]
    pub fn get_null_cell(&self) -> WebBufferCell {
        WebBufferCell::new(self.native.get_null_cell())
    }
    #[wasm_bindgen(js_name = getLine)]
    pub fn get_line(&self, index: f64) -> Result<JsValue, JsValue> {
        let Some(line) = self
            .native
            .get_line_number(index)
            .map_err(|error| to_js_error(&error))?
        else {
            return Ok(JsValue::UNDEFINED);
        };
        let key = Rc::as_ptr(&line) as usize;
        let mut cache = self.lines.borrow_mut();
        if let Some(cached) = cache.get(&key) {
            return Ok(cached.object.clone());
        }
        // Keep JS objects alive while their native row is still owned by core,
        // as JS WeakRef users can observe that lifetime. Each wrapper owns one
        // native Rc; a count of one means only the cached wrapper remains.
        // Release those old wrappers periodically instead of retaining history.
        let live = self.native.len().map_err(|error| to_js_error(&error))?;
        if cache.len() > live.saturating_mul(2) {
            cache.retain(|_, entry| entry.native.strong_count() > 1);
        }
        let weak = Rc::downgrade(&line);
        let object: JsValue = WebBufferLine { native: line }.into();
        cache.insert(
            key,
            CachedLine {
                native: weak,
                object: object.clone(),
            },
        );
        Ok(object)
    }
}

#[wasm_bindgen(js_name = BufferLine)]
pub struct WebBufferLine {
    native: LineRef,
}
#[wasm_bindgen(js_class = BufferLine)]
impl WebBufferLine {
    #[wasm_bindgen(getter, js_name = isWrapped)]
    pub fn is_wrapped(&self) -> Result<bool, JsValue> {
        Ok(self
            .native
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .is_wrapped)
    }
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> Result<usize, JsValue> {
        Ok(self
            .native
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .len())
    }
    #[wasm_bindgen(js_name = getCell)]
    pub fn get_cell(&self, index: f64, target: JsValue) -> Result<JsValue, JsValue> {
        if index < 0.0 || index >= self.length()? as f64 {
            return Ok(JsValue::UNDEFINED);
        }
        let read = || {
            let line = self
                .native
                .try_borrow()
                .map_err(|error| to_js_error(&error))?;
            Ok::<_, JsValue>(read_cell_number(&line, index))
        };
        if !target.is_truthy() {
            return Ok(WebBufferCell::numeric(read()?).into());
        }
        let assign = |field: &str, property: &JsValue| assign_cell_field(&target, field, property);
        // Each setter can mutate or resize the row. Later source fields are
        // read afterward, without retaining a native borrow across JS callbacks.
        let read_words = || {
            let line = self
                .native
                .try_borrow()
                .map_err(|error| to_js_error(&error))?;
            Ok::<_, JsValue>(read_cell_words_number(&line, index))
        };
        let word = |value: Option<u32>| value.map_or(JsValue::UNDEFINED, JsValue::from);
        assign("content", &word(read_words()?.content))?;
        assign("fg", &word(read_words()?.fg))?;
        assign("bg", &word(read_words()?.bg))?;
        let destination = Reflect::get(&target, &"content".into())?;
        let combined = if WebBufferCell::coerce_word(&destination)?
            & xterm_core::buffer::constants::content::IS_COMBINED_MASK
            != 0
        {
            let chars = {
                let line = self
                    .native
                    .try_borrow()
                    .map_err(|error| to_js_error(&error))?;
                combined_at_number(&line, index)
            };
            chars.map_or(JsValue::UNDEFINED, |chars| to_js_string(&chars).into())
        } else {
            to_js_string(&[]).into()
        };
        assign("combinedData", &combined)?;
        let extended = {
            let line = self
                .native
                .try_borrow()
                .map_err(|error| to_js_error(&error))?;
            extended_at_number(&line, index)
        };
        assign("extended", &WebBufferCell::extended_value(extended))?;
        Ok(target)
    }
    #[wasm_bindgen(js_name = translateToString)]
    pub fn translate_to_string(
        &self,
        trim_right: Option<bool>,
        start: Option<f64>,
        end: Option<f64>,
    ) -> Result<JsString, JsValue> {
        let translated = {
            let line = self
                .native
                .try_borrow()
                .map_err(|error| to_js_error(&error))?;
            translate_to_string_number(&line, trim_right.unwrap_or(false), start, end, false)
        };
        // Error/string conversion crosses the host boundary only after the row
        // borrow ends, so host overrides cannot reenter a borrowed line.
        let translated = translated.map_err(|error| match error {
            NumericTranslationError::MissingCombinedString => {
                JsValue::from(js_sys::TypeError::new(&error.to_string()))
            }
            NumericTranslationError::NonProgressingRange | NumericTranslationError::Allocation => {
                JsValue::from(js_sys::RangeError::new(&error.to_string()))
            }
        })?;
        Ok(to_js_string(&translated.text))
    }
}
