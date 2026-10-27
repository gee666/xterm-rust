// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/Clipboard.ts.

use super::{listen, set_value, InputBindings, State};
use crate::dom::BrowserDom;
use crate::utf16::to_js_string;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{ClipboardEvent, MouseEvent};

pub(super) fn bind(bindings: &mut InputBindings, dom: &BrowserDom) -> Result<(), JsValue> {
    listen(
        bindings,
        dom.element.as_ref(),
        "copy",
        false,
        |state, event| {
            if !state.host.has_selection() {
                return Ok(());
            }
            let event: ClipboardEvent = event.dyn_into()?;
            // Upstream always prevents default, even when clipboardData is absent.
            if let Some(data) = event.clipboard_data() {
                let text = to_js_string(&state.host.selection_text());
                set_clipboard_data(&data, &text)?;
            }
            event.prevent_default();
            Ok(())
        },
    )?;
    for target in [dom.textarea.as_ref(), dom.element.as_ref()] {
        listen(bindings, target, "paste", false, |state, event| {
            let event: ClipboardEvent = event.dyn_into()?;
            event.stop_propagation();
            if let Some(data) = event.clipboard_data() {
                let text = get_clipboard_data(&data)?;
                paste(state, &text)?;
            }
            Ok(())
        })?;
    }
    let firefox = bindings.state.host.options().is_firefox;
    listen(
        bindings,
        dom.element.as_ref(),
        if firefox { "mousedown" } else { "contextmenu" },
        false,
        move |state, event| {
            let event: MouseEvent = event.dyn_into()?;
            if firefox && event.button() != 2 {
                return Ok(());
            }
            move_textarea(state, &event)?;
            if state.host.options().right_click_selects_word {
                state.host.right_click_select(&event);
            }
            let selection = state.host.selection_text();
            set_value(&state.textarea, &selection)?;
            state.textarea.select();
            Ok(())
        },
    )?;
    if bindings.state.host.options().is_linux {
        listen(
            bindings,
            dom.element.as_ref(),
            "auxclick",
            false,
            |state, event| {
                let event: MouseEvent = event.dyn_into()?;
                if event.button() == 1 {
                    move_textarea(state, &event)?;
                }
                Ok(())
            },
        )?;
    }
    Ok(())
}

fn move_textarea(state: &State, event: &MouseEvent) -> Result<(), JsValue> {
    let rect = state.screen.get_bounding_client_rect();
    let position = super::super::clipboard::context_menu_position(
        f64::from(event.client_x()),
        f64::from(event.client_y()),
        rect.left(),
        rect.top(),
    );
    let style = state.textarea.style();
    for (key, value) in [
        ("left", format!("{}px", position.left)),
        ("top", format!("{}px", position.top)),
        ("width", format!("{}px", position.width)),
        ("height", format!("{}px", position.height)),
        ("z-index", position.z_index.to_string()),
    ] {
        style.set_property(key, &value)?;
    }
    state.textarea.focus()
}

pub(super) fn paste(state: &Rc<State>, text: &[u16]) -> Result<(), JsValue> {
    let options = state.host.options();
    let text = super::super::clipboard::paste(
        text,
        options.bracketed_paste,
        options.ignore_bracketed_paste,
    );
    state.host.data(&text, true);
    set_value(&state.textarea, &[])
}

// web-sys string conversions are lossy for lone surrogates. Call the real
// DataTransfer methods with JsString values instead of Rust String bindings.
fn get_clipboard_data(data: &web_sys::DataTransfer) -> Result<Vec<u16>, JsValue> {
    let method = js_sys::Reflect::get(data.as_ref(), &JsValue::from_str("getData"))?
        .dyn_into::<js_sys::Function>()?;
    let value = method
        .call1(data.as_ref(), &JsValue::from_str("text/plain"))?
        .dyn_into::<js_sys::JsString>()?;
    Ok(crate::utf16::from_js_string(&value))
}

fn set_clipboard_data(
    data: &web_sys::DataTransfer,
    text: &js_sys::JsString,
) -> Result<(), JsValue> {
    let method = js_sys::Reflect::get(data.as_ref(), &JsValue::from_str("setData"))?
        .dyn_into::<js_sys::Function>()?;
    method.call2(data.as_ref(), &JsValue::from_str("text/plain"), text)?;
    Ok(())
}
