// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from browser/CoreBrowserTerminal.ts.

use super::{composition, listen, set_value, InputBindings, State};
use crate::input::keyboard_dispatch::{
    DispatchKeyEvent, KeyEventKind, KeyboardAction, KeyboardHost, KeyboardReadHost,
    KeyboardSnapshot, TextInputEvent,
};
use crate::utf16::from_js_string;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{InputEvent, KeyboardEvent};

pub(super) fn bind(bindings: &mut InputBindings) -> Result<(), JsValue> {
    let area = bindings.state.textarea.clone();
    for (name, kind) in [
        ("keydown", KeyEventKind::Down),
        ("keyup", KeyEventKind::Up),
        ("keypress", KeyEventKind::Press),
    ] {
        listen(bindings, area.as_ref(), name, true, move |state, event| {
            let event: KeyboardEvent = event.dyn_into()?;
            let key = js_string_property(event.as_ref(), "key")?;
            let code = js_string_property(event.as_ref(), "code")?;
            let dispatch = DispatchKeyEvent {
                key: xterm_core::input::keyboard::KeyboardEvent {
                    key_code: event.key_code(),
                    key: &key,
                    code: &code,
                    shift_key: event.shift_key(),
                    alt_key: event.alt_key(),
                    ctrl_key: event.ctrl_key(),
                    meta_key: event.meta_key(),
                },
                kind,
                repeat: event.repeat(),
                // Read getModifierState only when the source asks for AltGraph.
                alt_graph: false,
                char_code: Some(event.char_code()),
                which: Some(event.which()),
            };
            let adapter = Adapter {
                state,
                event: Some(&event),
            };
            match kind {
                KeyEventKind::Down => {
                    state.keyboard.key_down(&dispatch, &adapter)?;
                }
                KeyEventKind::Up => {
                    state.keyboard.key_up(&dispatch, &adapter)?;
                }
                KeyEventKind::Press => {
                    state.keyboard.key_press(&dispatch, &adapter)?;
                }
            }
            Ok(())
        })?;
    }
    listen(bindings, area.as_ref(), "input", true, |state, event| {
        let event: InputEvent = event.dyn_into()?;
        let data = js_sys::Reflect::get(event.as_ref(), &JsValue::from_str("data"))?;
        let data = if data.is_null() || data.is_undefined() {
            None
        } else {
            Some(from_js_string(&data.dyn_into::<js_sys::JsString>()?))
        };
        state.keyboard.input(
            TextInputEvent {
                data: data.as_deref(),
                insert_text: event.input_type() == "insertText",
                composed: event.composed(),
            },
            &Adapter { state, event: None },
        )?;
        Ok(())
    })?;
    Ok(())
}

fn js_string_property(object: &JsValue, name: &str) -> Result<Vec<u16>, JsValue> {
    let value =
        js_sys::Reflect::get(object, &JsValue::from_str(name))?.dyn_into::<js_sys::JsString>()?;
    Ok(from_js_string(&value))
}

struct Adapter<'a> {
    state: &'a Rc<State>,
    event: Option<&'a KeyboardEvent>,
}

impl KeyboardReadHost for Adapter<'_> {
    type Error = JsValue;
    fn snapshot(&self) -> Result<KeyboardSnapshot, JsValue> {
        self.state.host.snapshot()
    }
    fn is_mac(&self) -> Result<bool, JsValue> {
        self.state.host.is_mac()
    }
    fn is_windows(&self) -> Result<bool, JsValue> {
        self.state.host.is_windows()
    }
    fn mac_option_is_meta(&self) -> Result<bool, JsValue> {
        self.state.host.mac_option_is_meta()
    }
    fn scroll_on_user_input(&self) -> Result<bool, JsValue> {
        self.state.host.scroll_on_user_input()
    }
    fn scroll_position(&self) -> Result<(u32, u32), JsValue> {
        self.state.host.scroll_position()
    }
    fn rows(&self) -> Result<u32, JsValue> {
        self.state.host.rows()
    }
    fn screen_reader_mode(&self) -> Result<bool, JsValue> {
        self.state.host.screen_reader_mode()
    }
    fn use_kitty(&self) -> Result<bool, JsValue> {
        self.state.host.use_kitty()
    }
    fn use_win32(&self) -> Result<bool, JsValue> {
        self.state.host.use_win32()
    }
    fn evaluate_key_down(
        &self,
        event: &DispatchKeyEvent<'_>,
    ) -> Result<xterm_core::keyboard::KeyboardResult, JsValue> {
        self.state.host.evaluate_key_down(event)
    }
    fn evaluate_key_up(
        &self,
        event: &DispatchKeyEvent<'_>,
    ) -> Result<Option<xterm_core::keyboard::KeyboardResult>, JsValue> {
        self.state.host.evaluate_key_up(event)
    }
    fn alt_graph_state(&self, _: &DispatchKeyEvent<'_>) -> Result<bool, JsValue> {
        let event = self
            .event
            .ok_or_else(|| js_sys::Error::new("Missing keyboard event"))?;
        let method = js_sys::Reflect::get(event.as_ref(), &JsValue::from_str("getModifierState"))?
            .dyn_into::<js_sys::Function>()?;
        Ok(method
            .call1(event.as_ref(), &JsValue::from_str("AltGraph"))?
            .is_truthy())
    }
}

impl KeyboardHost for Adapter<'_> {
    fn custom_key_handler(&self, _: &DispatchKeyEvent<'_>) -> Result<bool, JsValue> {
        self.state.host.custom_key_handler(
            self.event
                .ok_or_else(|| js_sys::Error::new("Missing keyboard event"))?,
        )
    }

    fn composition_key_down(&self, event: &DispatchKeyEvent<'_>) -> Result<bool, JsValue> {
        composition::keydown(self.state, event.key.key_code)
    }

    fn perform(
        &self,
        _: Option<&DispatchKeyEvent<'_>>,
        action: KeyboardAction<'_>,
    ) -> Result<(), JsValue> {
        match action {
            KeyboardAction::Cancel => {
                let event = self
                    .event
                    .ok_or_else(|| js_sys::Error::new("Cannot cancel a missing keyboard event"))?;
                event.prevent_default();
                event.stop_propagation();
            }
            KeyboardAction::ClearTextarea => {
                set_value(&self.state.textarea, &[])?;
            }
            KeyboardAction::Focus => {
                let options = web_sys::FocusOptions::new();
                options.set_prevent_scroll(true);
                self.state.textarea.focus_with_options(&options)?;
            }
            KeyboardAction::Data { key, user_input } => self.state.host.data(key, user_input)?,
            effect => self.state.host.keyboard_effect(self.event, effect)?,
        }
        Ok(())
    }
}
