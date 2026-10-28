//! Public mode reads use the same owned snapshot as the headless API.

use super::Terminal;
use crate::js_error::to_js_error;
use js_sys::{Array, Object};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(getter)]
    pub fn modes(&self) -> Result<JsValue, JsValue> {
        let modes = self
            .shared
            .native
            .modes()
            .map_err(|error| to_js_error(&error))?;
        let entries = [
            (
                "applicationCursorKeysMode",
                JsValue::from(modes.application_cursor_keys_mode),
            ),
            (
                "applicationKeypadMode",
                modes.application_keypad_mode.into(),
            ),
            ("bracketedPasteMode", modes.bracketed_paste_mode.into()),
            ("insertMode", modes.insert_mode.into()),
            (
                "mouseTrackingMode",
                modes.mouse_tracking_mode.as_str().into(),
            ),
            ("originMode", modes.origin_mode.into()),
            (
                "reverseWraparoundMode",
                modes.reverse_wraparound_mode.into(),
            ),
            ("sendFocusMode", modes.send_focus_mode.into()),
            ("showCursor", modes.show_cursor.into()),
            (
                "synchronizedOutputMode",
                modes.synchronized_output_mode.into(),
            ),
            ("win32InputMode", modes.win32_input_mode.into()),
            ("wraparoundMode", modes.wraparound_mode.into()),
        ]
        .into_iter()
        .map(|(key, value)| {
            let pair = Array::new();
            pair.push(&JsValue::from_str(key));
            pair.push(&value);
            pair
        })
        .collect::<Array>();
        // fromEntries defines own data properties, unlike ordinary assignment
        // which could invoke a setter installed on Object.prototype.
        Object::from_entries(&entries).map(Into::into)
    }
}
