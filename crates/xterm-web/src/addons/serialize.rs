//! Rust serializer behind the small JS lifetime/activation facade.
#[path = "serialize_html.rs"]
mod html;
#[path = "serialize_ops.rs"]
mod ops;
#[path = "serialize_view.rs"]
mod view;
use crate::{
    js_error::to_js_error,
    terminal::{Shared, Terminal},
    utf16::to_js_string,
};
use js_sys::JsString;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{
    public_api::buffer::{BufferApiView, BufferType},
    serialize::{
        serialize_buffer, serialize_modes, CellRange, Position, SerializeError, SerializeModes,
    },
};

#[wasm_bindgen]
#[derive(Default)]
pub struct SerializeBackend {
    host: RefCell<Option<Rc<Shared>>>,
}
#[wasm_bindgen]
impl SerializeBackend {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        *self
            .host
            .try_borrow_mut()
            .map_err(|error| js_sys::Error::new(&error.to_string()))? =
            Some(Rc::clone(&terminal.shared));
        Ok(())
    }
    #[wasm_bindgen(js_name=clearHost)]
    pub fn clear_host(&self) -> Result<(), JsValue> {
        self.host
            .try_borrow_mut()
            .map_err(|error| js_sys::Error::new(&error.to_string()))?
            .take();
        Ok(())
    }
    pub fn serialize(&self, options: JsValue) -> Result<JsString, JsValue> {
        self.current()?;
        let range = ops::optional(&options, "range")?;
        let host = self.current()?;
        let namespace = host.native.buffer().map_err(|error| to_js_error(&error))?;
        let normal = namespace.normal();
        let mut output = if ops::serialTruthy(&range) {
            let range = ops::optional(&options, "range")?;
            let start = ops::serialStart(&range)?;
            let cols = host.native.cols().map_err(|error| to_js_error(&error))?;
            let end = ops::serialEnd(&range)?;
            render(&host, normal, start, end, cols, true)?
        } else {
            let count = ops::optional(&options, "scrollback")?;
            render_scrollback(&host, normal, &count)?
        };
        if !ops::serialTruthy(&ops::optional(&options, "excludeAltBuffer")?) {
            let host = self.current()?;
            let namespace = host.native.buffer().map_err(|error| to_js_error(&error))?;
            if namespace
                .active()
                .map_err(|error| to_js_error(&error))?
                .buffer_type()
                == BufferType::Alternate
            {
                output.extend("\x1b[?1049h\x1b[H".encode_utf16());
                output.extend(render_scrollback(
                    &host,
                    namespace.alternate(),
                    &JsValue::UNDEFINED,
                )?);
            }
        }
        if !ops::serialTruthy(&ops::optional(&options, "excludeModes")?) {
            let host = self.current()?;
            let modes = host.native.modes().map_err(|error| to_js_error(&error))?;
            let (rows, top, bottom) = host
                .native
                .with_state(|state| {
                    let active = state.buffers.active();
                    let buffer = active
                        .try_borrow()
                        .map_err(|error| js_sys::Error::new(&error.to_string()))?;
                    Ok::<_, JsValue>((state.rows(), buffer.scroll_top, buffer.scroll_bottom))
                })
                .map_err(|error| to_js_error(&error))??;
            output.extend(serialize_modes(
                &SerializeModes::from(&modes),
                rows,
                top,
                bottom,
            ));
        }
        Ok(to_js_string(&output))
    }
    #[wasm_bindgen(js_name=serializeAsHTML)]
    pub fn serialize_as_html(&self, options: JsValue) -> Result<JsString, JsValue> {
        html::serialize(self, options)
    }
}
impl SerializeBackend {
    fn current(&self) -> Result<Rc<Shared>, JsValue> {
        self.host
            .try_borrow()
            .map_err(|error| js_sys::Error::new(&error.to_string()))?
            .as_ref()
            .cloned()
            .ok_or_else(|| js_sys::Error::new("Cannot use addon until it has been loaded").into())
    }
}
fn render_scrollback(
    host: &Shared,
    source: Rc<BufferApiView>,
    count: &JsValue,
) -> Result<Vec<u16>, JsValue> {
    let length = source.len().map_err(|error| to_js_error(&error))?;
    let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
    let retained = ops::serialRetained(count, rows as f64, length as f64)?;
    let cols = host.native.cols().map_err(|error| to_js_error(&error))?;
    render(
        host,
        source,
        JsValue::from_f64(length as f64 - retained),
        JsValue::from_f64(length as f64 - 1.0),
        cols,
        false,
    )
}
fn render(
    host: &Shared,
    source: Rc<BufferApiView>,
    start: JsValue,
    end: JsValue,
    cols: usize,
    exclude: bool,
) -> Result<Vec<u16>, JsValue> {
    let span = ops::serialSpan(&start, &end)?;
    let first = ops::serialNumber(&start)?;
    let count = index(span)?;
    let origin = if first >= 0.0 && first.fract() == 0.0 && first + span <= u32::MAX as f64 {
        index(first)?
    } else {
        0
    };
    let end = origin
        .checked_add(count)
        .ok_or_else(|| js_sys::RangeError::new("Invalid array length"))?;
    let (rows, attrs) = host
        .native
        .with_state(|state| (state.rows(), state.attributes.clone()))
        .map_err(|error| to_js_error(&error))?;
    let view = view::NumberView::new(source, first, origin, rows, None)
        .map_err(|error| serialization_error(&error))?;
    serialize_buffer(
        &view,
        cols,
        rows,
        &attrs,
        CellRange {
            start: Position { x: 0, y: origin },
            end: Position { x: cols, y: end },
        },
        exclude,
    )
    .map_err(|error| serialization_error(&error))
}
fn index(value: f64) -> Result<usize, JsValue> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > u32::MAX as f64 {
        return Err(js_sys::RangeError::new("Invalid array length").into());
    }
    // The browser target is wasm32 and the checked Number lies in the u32 range.
    Ok(value as u32 as usize)
}
fn serialization_error(error: &SerializeError) -> JsValue {
    match error {
        SerializeError::InvalidRange => js_sys::RangeError::new(&error.to_string()).into(),
        SerializeError::MissingWrappedLine
        | SerializeError::MissingExtendedAttributes
        | SerializeError::MissingCombinedString => {
            js_sys::TypeError::new(&error.to_string()).into()
        }
        _ => to_js_error(error),
    }
}
