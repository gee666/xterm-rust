// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/Clipboard.ts.

use super::{listen, set_value, InputBindings, State};
use crate::{
    dom::BrowserDom,
    utf16::{from_js_string, to_js_string},
};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, MouseEvent};

// Method application and primitive coercion belong at the JS boundary. Paste
// normalization/bracketing and context-menu geometry remain implemented in Rust.
#[wasm_bindgen(inline_js = "
const clipboardApply = Reflect.apply;
export function clipboardProperty(value, key) { return value[key]; }
export function clipboardInvoke1(fn, receiver, a) { return clipboardApply(fn, receiver, [a]); }
export function clipboardInvoke2(fn, receiver, a, b) { return clipboardApply(fn, receiver, [a,b]); }
export function clipboardSubtract(value, rhs) { return value - rhs; }
export function clipboardPrevent(event) { event.preventDefault(); }
export function clipboardStop(event) { event.stopPropagation(); }
")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn clipboardProperty(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn clipboardInvoke1(
        function: &JsValue,
        receiver: &JsValue,
        a: &JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn clipboardInvoke2(
        function: &JsValue,
        receiver: &JsValue,
        a: &JsValue,
        b: &JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn clipboardSubtract(value: &JsValue, rhs: f64) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch)]
    fn clipboardPrevent(event: &Event) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn clipboardStop(event: &Event) -> Result<(), JsValue>;
}

pub(super) fn bind(bindings: &mut InputBindings, dom: &BrowserDom) -> Result<(), JsValue> {
    listen(
        bindings,
        dom.element.as_ref(),
        "copy",
        false,
        |state, event| {
            if !state.host.has_selection()? {
                return Ok(());
            }
            if clipboardProperty(event.as_ref(), "clipboardData")?.is_truthy() {
                let data = clipboardProperty(event.as_ref(), "clipboardData")?;
                let method = clipboardProperty(&data, "setData")?;
                // Source resolves setData before evaluating the selection argument,
                // but does not check callability until that argument is evaluated.
                let text = to_js_string(&state.host.selection_text()?);
                clipboardInvoke2(&method, &data, &JsValue::from_str("text/plain"), &text)?;
            }
            clipboardPrevent(&event)
        },
    )?;
    for target in [dom.textarea.as_ref(), dom.element.as_ref()] {
        listen(bindings, target, "paste", false, |state, event| {
            clipboardStop(&event)?;
            if clipboardProperty(event.as_ref(), "clipboardData")?.is_truthy() {
                let data = clipboardProperty(event.as_ref(), "clipboardData")?;
                let text = get_clipboard_data(&data)?;
                paste(state, &text)?;
            }
            Ok(())
        })?;
    }
    let firefox = bindings.state.host.is_firefox()?;
    listen(
        bindings,
        dom.element.as_ref(),
        if firefox { "mousedown" } else { "contextmenu" },
        false,
        move |state, event| {
            if firefox && clipboardProperty(event.as_ref(), "button")? != JsValue::from_f64(2.0) {
                return Ok(());
            }
            // The option is an argument to rightClickHandler, so focus callbacks
            // inside move_textarea must not change it for this invocation.
            let select_word = state.host.right_click_selects_word()?;
            move_textarea(state, &event)?;
            if select_word {
                // Registered DOM events do not need a realm-specific instanceof
                // check. The source's TS MouseEvent assertion is also erased.
                state
                    .host
                    .right_click_select(event.unchecked_ref::<MouseEvent>())?;
            }
            let selection = state.host.selection_text()?;
            set_value(&state.textarea, &selection)?;
            state.textarea.select();
            Ok(())
        },
    )?;
    if bindings.state.host.is_linux()? {
        listen(
            bindings,
            dom.element.as_ref(),
            "auxclick",
            false,
            |state, event| {
                if clipboardProperty(event.as_ref(), "button")? == JsValue::from_f64(1.0) {
                    move_textarea(state, &event)?;
                }
                Ok(())
            },
        )?;
    }
    Ok(())
}

fn move_textarea(state: &State, event: &Event) -> Result<(), JsValue> {
    let rect = state.screen.get_bounding_client_rect();
    let client_x = clipboardProperty(event.as_ref(), "clientX")?;
    let left = subtract(&client_x, rect.left())?;
    let client_y = clipboardProperty(event.as_ref(), "clientY")?;
    let top = subtract(&client_y, rect.top())?;
    let position = super::super::clipboard::context_menu_position(left, top, 0.0, 0.0);
    let style = state.textarea.style();
    for (key, value) in [
        ("width", format!("{}px", position.width)),
        ("height", format!("{}px", position.height)),
        ("left", format!("{}px", position.left)),
        ("top", format!("{}px", position.top)),
        ("z-index", position.z_index.to_string()),
    ] {
        style.set_property(key, &value)?;
    }
    state.textarea.focus()
}
fn subtract(value: &JsValue, rhs: f64) -> Result<f64, JsValue> {
    match value.as_f64() {
        Some(value) => Ok(value - rhs),
        None => clipboardSubtract(value, rhs),
    }
}

pub(super) fn paste(state: &Rc<State>, text: &[u16]) -> Result<(), JsValue> {
    let prepared = super::super::clipboard::prepare_text_for_terminal(text);
    let bracketed = state.host.bracketed_paste()? && !state.host.ignore_bracketed_paste()?;
    let text = if bracketed {
        super::super::clipboard::bracket_text_for_paste(&prepared, true)
    } else {
        prepared
    };
    state.host.data(&text, true)?;
    set_value(&state.textarea, &[])
}

fn get_clipboard_data(data: &JsValue) -> Result<Vec<u16>, JsValue> {
    let method = clipboardProperty(data, "getData")?;
    let value = clipboardInvoke1(&method, data, &JsValue::from_str("text/plain"))?;
    if !value.is_string() {
        return Err(js_sys::TypeError::new("Clipboard text must be a string").into());
    }
    Ok(from_js_string(&js_sys::JsString::from(value)))
}
