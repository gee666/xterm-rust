//! Public mode snapshots read the real engine state and are independent objects.
use super::Terminal;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function modesObject(cursor, keypad, paste, insert, mouse, origin, reverse, focus, show, sync, win32, wrap) {
  return { applicationCursorKeysMode: cursor, applicationKeypadMode: keypad,
    bracketedPasteMode: paste, insertMode: insert, mouseTrackingMode: mouse,
    originMode: origin, reverseWraparoundMode: reverse, sendFocusMode: focus,
    showCursor: show, synchronizedOutputMode: sync, win32InputMode: win32,
    wraparoundMode: wrap };
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = modesObject)]
    fn modes_object(
        cursor: bool,
        keypad: bool,
        paste: bool,
        insert: bool,
        mouse: &str,
        origin: bool,
        reverse: bool,
        focus: bool,
        show: bool,
        sync: bool,
        win32: bool,
        wrap: bool,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(getter)]
    pub fn modes(&self) -> Result<JsValue, JsValue> {
        let modes = self
            .shared
            .native
            .modes()
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        modes_object(
            modes.application_cursor_keys_mode,
            modes.application_keypad_mode,
            modes.bracketed_paste_mode,
            modes.insert_mode,
            modes.mouse_tracking_mode.as_str(),
            modes.origin_mode,
            modes.reverse_wraparound_mode,
            modes.send_focus_mode,
            modes.show_cursor,
            modes.synchronized_output_mode,
            modes.win32_input_mode,
            modes.wraparound_mode,
        )
    }
}
