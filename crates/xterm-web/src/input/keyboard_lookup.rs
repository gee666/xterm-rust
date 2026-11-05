// Licensed under MIT. Host primitives only; Rust owns keyboard encoding policy.
use crate::utf16::{from_js_string, to_js_string};
use std::cell::OnceCell;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use xterm_core::input::{
    keyboard_event_source::lookup::{Hint, HostValue, Value},
    kitty_keyboard::lookup::Table as KittyTable,
    win32_input_mode::lookup::Table,
};

#[wasm_bindgen(module = "/src/input/win32_tables.js")]
extern "C" {
    #[wasm_bindgen(js_name = win32CreateTables)]
    fn create_win32_tables() -> js_sys::Array;
    #[wasm_bindgen(catch, js_name = win32TemplateText)]
    fn template_text(value: &JsValue) -> Result<js_sys::JsString, JsValue>;
}
#[wasm_bindgen(module = "/src/input/kitty_tables.js")]
extern "C" {
    #[wasm_bindgen(js_name = kittyCreateTables)]
    fn create_kitty_tables() -> js_sys::Array;
    #[wasm_bindgen(catch, js_name = keyboardDefaultText)]
    fn default_text(value: &JsValue) -> Result<js_sys::JsString, JsValue>;
}

/// One owner alongside the browser input service. Tables initialize lazily
/// like the source handlers and survive mode changes and repeated events.
/// No RefCell borrow crosses prototype getters or conversion callbacks.
#[derive(Default)]
pub struct KeyboardTables {
    win32: OnceCell<js_sys::Array>,
    kitty: OnceCell<js_sys::Array>,
}

impl KeyboardTables {
    /// The actual source Win32 receiver, including mutable own properties.
    pub fn table(&self, table: Table) -> JsValue {
        let index = match table {
            Table::VirtualKey => 0,
            Table::ScanCode => 1,
            Table::ControlChar => 2,
        };
        self.win32.get_or_init(create_win32_tables).get(index)
    }

    /// The actual source Kitty receiver. Object/array literals cannot invoke
    /// getters or inherited setters, so initialization cannot reenter OnceCell.
    pub fn kitty_table(&self, table: KittyTable) -> JsValue {
        let index = match table {
            KittyTable::Functional => 0,
            KittyTable::Tilde => 1,
            KittyTable::Letter => 2,
            KittyTable::Ss3 => 3,
        };
        self.kitty.get_or_init(create_kitty_tables).get(index)
    }

    pub fn lookup(&self, table: Table, key: &[u16]) -> Result<Value<JsValue>, JsValue> {
        lookup(&self.table(table), key)
    }

    pub fn kitty_lookup(&self, table: KittyTable, key: &[u16]) -> Result<Value<JsValue>, JsValue> {
        lookup(&self.kitty_table(table), key)
    }
}

fn lookup(receiver: &JsValue, key: &[u16]) -> Result<Value<JsValue>, JsValue> {
    let value = js_sys::Reflect::get(receiver, &to_js_string(key))?;
    Ok(Value::Host(Box::new(RawValue(value))))
}

struct RawValue(JsValue);
impl HostValue<JsValue> for RawValue {
    fn is_undefined(&self) -> bool {
        self.0.is_undefined()
    }
    fn is_truthy(&self) -> bool {
        self.0.is_truthy()
    }
    fn number(&self) -> Option<f64> {
        self.0.as_f64()
    }
    fn to_utf16(&self, hint: Hint) -> Result<Vec<u16>, JsValue> {
        let text = match hint {
            Hint::String => template_text(&self.0)?,
            Hint::Default => default_text(&self.0)?,
        };
        Ok(from_js_string(&text))
    }
}
