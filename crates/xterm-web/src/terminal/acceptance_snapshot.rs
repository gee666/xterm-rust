//! Read-only diagnostic data from the live Rust engine. Enabled only by the
//! default-off acceptance-hooks feature, never required by the production ABI.

use super::Terminal;
use crate::js_error::to_js_error;
use serde_json::{json, Value};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{
    buffer::{AttributeData, Buffer, BufferLine},
    charset::{builtin_charset, Charset, CharsetValue},
    core_service::CursorStyle,
};

fn attributes(value: &AttributeData) -> Value {
    json!({"fg": value.fg, "bg": value.bg, "ext": value.extended.ext(), "urlId": value.extended.url_id(), "hasPayload": value.extended.payload().is_some()})
}
fn charset(value: &Option<Charset>) -> Result<Value, JsValue> {
    let Some(value) = value else {
        return Ok(Value::Null);
    };
    let builtin = "04567=ACEHKQRYZ"
        .chars()
        .find(|&id| builtin_charset(id).is_some_and(|other| Charset::ptr_eq(value, &other)));
    let entries: Vec<_> = value
        .snapshot()
        .map_err(|error| to_js_error(&error))?
        .into_iter()
        .map(|(key, value)| {
            let value = match value {
                CharsetValue::Text(text) => json!(text),
                CharsetValue::Undefined => Value::Null,
            };
            json!([key, value])
        })
        .collect();
    Ok(json!({"builtin": builtin.map(|id| id.to_string()), "entries": entries}))
}
fn line(value: &BufferLine) -> Value {
    let cells: Vec<_> = (0..value.len()).map(|x| {
        value.get_cell(x).map_or(Value::Null, |cell| {
            json!({"content": cell.content, "fg": cell.fg, "bg": cell.bg,
                "ext": cell.extended.ext(), "urlId": cell.extended.url_id(), "hasPayload": cell.extended.payload().is_some(),
                "chars": cell.get_chars(), "width": cell.get_width(), "code": cell.get_code().unwrap_or(0)})
        })
    }).collect();
    json!({"isWrapped": value.is_wrapped, "cells": cells})
}
fn buffer(value: &Rc<RefCell<Buffer>>) -> Result<Value, JsValue> {
    let value = value.try_borrow().map_err(|error| to_js_error(&error))?;
    let mut lines = Vec::with_capacity(value.lines.len());
    for index in 0..value.lines.len() {
        lines.push(match value.lines.get(index) {
            Some(row) => {
                let row = row.try_borrow().map_err(|error| to_js_error(&error))?;
                line(&row)
            }
            None => Value::Null,
        });
    }
    let markers: Vec<_> = value.markers.try_borrow().map_err(|error| to_js_error(&error))?.iter()
        .map(|marker| json!({"id": marker.id(), "line": marker.line(), "isDisposed": marker.is_disposed()})).collect();
    Ok(json!({
        "cursorX": value.x, "cursorY": value.y, "baseY": value.ybase, "viewportY": value.ydisp,
        "scrollTop": value.scroll_top, "scrollBottom": value.scroll_bottom,
        "maxLength": value.lines.max_length(), "tabs": value.tabs,
        "saved": {"x": value.saved_x, "y": value.saved_y, "attributes": attributes(&value.saved_cur_attr_data),
            "charset": charset(&value.saved_charset)?, "charsets": value.saved_charsets.iter().map(charset).collect::<Result<Vec<_>, _>>()?,
            "glevel": value.saved_glevel, "origin": value.saved_origin_mode, "wraparound": value.saved_wraparound_mode},
        "markers": markers, "lines": lines,
    }))
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(js_name = _acceptanceOnInternalEvent)]
    pub fn acceptance_on_internal_event(
        &self,
        listener: js_sys::Function,
    ) -> Result<JsValue, JsValue> {
        use xterm_core::terminal::TerminalEvent as E;
        let event = self
            .shared
            .native
            .on_event()
            .filter_map(|event| match event {
                E::Render { .. }
                | E::SyncScrollBar
                | E::SendFocus
                | E::RequestReset
                | E::InputScroll(_) => Some(event.clone()),
                E::Bell
                | E::ColorSchemeQuery
                | E::Color(_)
                | E::WindowReport(_)
                | E::LineFeed
                | E::CursorMove
                | E::Scroll(_)
                | E::Resize { .. }
                | E::WriteParsed
                | E::Data(_)
                | E::Binary(_)
                | E::TitleChange(_)
                | E::AccessibilityChar(_)
                | E::AccessibilityTab(_) => None,
            });
        let register = super::events::callable(event, |event| {
            let value = match event {
                E::Render { start, end } => json!({"name": "refresh", "start": start, "end": end}),
                E::SyncScrollBar => json!({"name": "syncScrollBar"}),
                E::SendFocus => json!({"name": "sendFocus"}),
                E::RequestReset => json!({"name": "requestReset"}),
                E::InputScroll(position) => json!({"name": "inputScroll", "position": position}),
                _ => return Err(js_sys::Error::new("Unfiltered acceptance event").into()),
            };
            let text = serde_json::to_string(&value).map_err(|error| to_js_error(&error))?;
            js_sys::JSON::parse(&text)
        });
        register.call1(&JsValue::UNDEFINED, &listener)
    }

    #[wasm_bindgen(js_name = _acceptanceSnapshot)]
    pub fn acceptance_snapshot(&self) -> Result<JsValue, JsValue> {
        let disposed = self
            .shared
            .native
            .is_disposed()
            .map_err(|error| to_js_error(&error))?;
        let snapshot = self.shared.native.with_state(|state| {
            let normal = state.buffers.normal();
            let alternate = state.buffers.alt();
            let private = state.core.dec_private_modes.try_borrow().map_err(|error| to_js_error(&error))?;
            let kitty = state.core.kitty_keyboard.try_borrow().map_err(|error| to_js_error(&error))?;
            let cursor_style = private.cursor_style.map(|style| match style {
                CursorStyle::Block => "block", CursorStyle::Underline => "underline", CursorStyle::Bar => "bar",
            });
            Ok::<_, JsValue>(json!({
                "schema": 1, "cols": state.cols(), "rows": state.rows(), "disposed": disposed,
                "active": if Rc::ptr_eq(&state.active_buffer(), &alternate) { "alternate" } else { "normal" },
                "normal": buffer(&normal)?, "alternate": buffer(&alternate)?,
                "attributes": attributes(&state.attributes), "title": state.title, "iconName": state.icon_name,
                "charset": charset(&state.charset.charset)?, "charsets": state.charset.charsets().iter().map(charset).collect::<Result<Vec<_>, _>>()?,
                "glevel": state.charset.glevel, "unicodeVersion": state.unicode.active_version(),
                "modes": {"insert": state.core.modes.try_borrow().map_err(|error| to_js_error(&error))?.insert_mode,
                    "applicationCursorKeys": private.application_cursor_keys, "applicationKeypad": private.application_keypad,
                    "bracketedPaste": private.bracketed_paste_mode, "colorSchemeUpdates": private.color_scheme_updates,
                    "cursorBlink": private.cursor_blink, "cursorStyle": cursor_style,
                    "origin": private.origin, "reverseWraparound": private.reverse_wraparound, "sendFocus": private.send_focus,
                    "synchronizedOutput": private.synchronized_output, "win32Input": private.win32_input_mode,
                    "wraparound": private.wraparound, "cursorInitialized": state.core.is_cursor_initialized.get(),
                    "cursorHidden": state.core.is_cursor_hidden.get(), "mouseProtocol": state.mouse.active_protocol(),
                    "mouseEncoding": state.mouse.active_encoding()},
                "kitty": {"flags": kitty.flags, "mainFlags": kitty.main_flags, "altFlags": kitty.alt_flags,
                    "mainStack": kitty.main_stack, "altStack": kitty.alt_stack},
            }))
        }).map_err(|error| to_js_error(&error))??;
        let text = serde_json::to_string(&snapshot).map_err(|error| to_js_error(&error))?;
        js_sys::JSON::parse(&text)
    }
}
