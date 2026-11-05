// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Browser serialization uses the existing Rust serializers and public views.
mod html;
mod options;
mod view;
use crate::{
    js_error::to_js_error,
    terminal::{Shared, Terminal},
    utf16::to_js_string,
};
use js_sys::JsString;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use xterm_core::{
    public_api::BufferApiView,
    serialize::{serialize_buffer, serialize_live_modes, CellRange, Position},
};

#[wasm_bindgen]
extern "C" {
    /// An optional raw JS argument, including boxed primitives and symbols.
    #[wasm_bindgen(typescript_type = "any")]
    pub type SerializeArgument;
}

/// The compatible JS facade retains its host in a JS field. Rust stores only a
/// weak link, so an unreachable manager/addon/Terminal cycle is collectible.
#[wasm_bindgen]
#[derive(Default)]
pub struct SerializeAddon {
    terminal: RefCell<Weak<Shared>>,
}
#[wasm_bindgen]
impl SerializeAddon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn activate(&self, terminal: &Terminal) {
        *self.terminal.borrow_mut() = Rc::downgrade(&terminal.shared);
    }
    pub fn serialize(&self, options: Option<SerializeArgument>) -> Result<JsString, JsValue> {
        let options = options.map(Into::into).unwrap_or(JsValue::UNDEFINED);
        self.host()?;
        let has_range = options::property(&options, "range")?.is_truthy();
        let host = self.host()?;
        let namespace = host.native.buffer().map_err(|error| to_js_error(&error))?;
        let normal = namespace.normal();
        let mut output = if has_range {
            let range = options::property(&options, "range")?;
            let start = options::marker(&range, "start")?;
            let cols = columns(&host)?;
            let end = options::marker(&range, "end")?;
            buffer(
                &host,
                normal,
                CellRange {
                    start: Position { x: 0, y: start },
                    end: Position { x: cols, y: end },
                },
                true,
            )?
        } else {
            let scrollback = options::property(&options, "scrollback")?;
            scrollback_buffer(&host, normal, scrollback)?
        };
        if !options::property(&options, "excludeAltBuffer")?.is_truthy() {
            let host = self.host()?;
            let namespace = host.native.buffer().map_err(|error| to_js_error(&error))?;
            let active = namespace.active().map_err(|error| to_js_error(&error))?;
            if active.buffer_type() == xterm_core::public_api::BufferType::Alternate {
                output.extend("\x1b[?1049h\x1b[H".encode_utf16());
                output.extend(scrollback_buffer(
                    &host,
                    namespace.alternate(),
                    JsValue::UNDEFINED,
                )?);
            }
        }
        if !options::property(&options, "excludeModes")?.is_truthy() {
            let host = self.host()?;
            output.extend(
                host.native
                    .with_state(serialize_live_modes)
                    .map_err(|error| to_js_error(&error))?
                    .map_err(|error| view::error(&error))?,
            );
        }
        Ok(to_js_string(&output))
    }
    #[wasm_bindgen(js_name = serializeAsHTML)]
    pub fn serialize_as_html(
        &self,
        options: Option<SerializeArgument>,
    ) -> Result<JsString, JsValue> {
        html::serialize(self, options.map(Into::into).unwrap_or(JsValue::UNDEFINED))
            .map(|output| to_js_string(&output))
    }
    // Source disposal has no registrations and intentionally retains the host.
    pub fn dispose(&self) {}
}
impl SerializeAddon {
    fn host(&self) -> Result<Rc<Shared>, JsValue> {
        self.terminal
            .borrow()
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("Cannot use addon until it has been loaded").into())
    }
}
fn columns(host: &Shared) -> Result<usize, JsValue> {
    host.native.cols().map_err(|error| to_js_error(&error))
}
fn scrollback_buffer(
    host: &Shared,
    view: Rc<BufferApiView>,
    scrollback: JsValue,
) -> Result<Vec<u16>, JsValue> {
    let max = view.len().map_err(|error| to_js_error(&error))?;
    let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
    let start = options::scrollback_start(max, rows, scrollback)?;
    let cols = columns(host)?;
    buffer(
        host,
        view,
        CellRange {
            start: Position { x: 0, y: start },
            end: Position {
                x: cols,
                y: max.saturating_sub(1),
            },
        },
        false,
    )
}
fn buffer(
    host: &Shared,
    view: Rc<BufferApiView>,
    range: CellRange,
    exclude_cursor: bool,
) -> Result<Vec<u16>, JsValue> {
    let rows = host.native.rows().map_err(|error| to_js_error(&error))?;
    let attrs = host
        .native
        .with_state(|state| state.attributes.clone())
        .map_err(|error| to_js_error(&error))?;
    let view = view::BrowserView::new(view, host)?;
    serialize_buffer(&view, columns(host)?, rows, &attrs, range, exclude_cursor)
        .map_err(|error| view::error(&error))
}
