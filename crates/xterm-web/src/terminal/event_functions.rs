use super::events::{callable, empty, pair, pair_numbers, text};
use js_sys::Function;
use wasm_bindgen::JsValue;
use super::browser_services::BrowserEvents;
use xterm_core::headless;

pub struct EventFunctions {
    pub bell: Function,
    pub binary: Function,
    pub cursor: Function,
    pub data: Function,
    pub line_feed: Function,
    pub resize: Function,
    pub scroll: Function,
    pub title: Function,
    pub parsed: Function,
    pub key: Function,
    pub render: Function,
    pub selection: Function,
    pub dimensions: Function,
}
impl EventFunctions {
    pub fn new(native: &headless::Terminal, browser: &BrowserEvents) -> Self {
        Self {
            bell: callable(native.on_bell(), empty),
            binary: callable(native.on_binary(), |value| text(value)),
            cursor: callable(native.on_cursor_move(), empty),
            data: callable(native.on_data(), |value| text(value)),
            line_feed: callable(native.on_line_feed(), empty),
            resize: callable(native.on_resize(), |value| pair(("cols", value.cols), ("rows", value.rows))),
            scroll: callable(native.on_scroll(), |value| Ok(JsValue::from_f64(*value as f64))),
            title: callable(native.on_title_change(), |value| text(value)),
            parsed: callable(native.on_write_parsed(), empty),
            key: callable(browser.key.event(), |value| Ok(value.clone())),
            render: callable(browser.render.event(), |value| pair_numbers(("start", value.start), ("end", value.end))),
            selection: callable(browser.selection.event(), empty),
            dimensions: callable(browser.dimensions.event(), super::events::dimensions_value),
        }
    }
}
