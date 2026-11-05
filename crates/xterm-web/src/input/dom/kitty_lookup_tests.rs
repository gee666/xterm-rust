// Licensed under MIT. Real Rust core plus production live event and realm maps.
use super::LiveKeyEvent;
use crate::{input::keyboard_lookup::KeyboardTables, utf16::to_js_string};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::input::kitty_keyboard::{self, lookup::Table, EventType};

#[wasm_bindgen(module = "/tests/fixtures/kitty_lookup_driver.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = compareKittyRealm)]
    fn compare(factory: &js_sys::Function) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub struct KittyProbe {
    tables: KeyboardTables,
}
#[wasm_bindgen]
impl KittyProbe {
    pub fn table(&self, index: u32) -> Result<JsValue, JsValue> {
        Ok(self.tables.kitty_table(match index {
            0 => Table::Functional,
            1 => Table::Tilde,
            2 => Table::Letter,
            3 => Table::Ss3,
            _ => return Err(js_sys::RangeError::new("Invalid Kitty test table").into()),
        }))
    }
    pub fn encode(
        &self,
        event: JsValue,
        flags: u32,
        event_type: u8,
        mac: bool,
    ) -> Result<JsValue, JsValue> {
        let event_type = match event_type {
            1 => EventType::Press,
            2 => EventType::Repeat,
            3 => EventType::Release,
            _ => return Err(js_sys::RangeError::new("Invalid Kitty test event").into()),
        };
        let event = event.unchecked_into::<web_sys::KeyboardEvent>();
        let result = kitty_keyboard::try_evaluate(
            &LiveKeyEvent(&event, &self.tables),
            flags,
            event_type,
            mac,
        )?;
        let output = js_sys::Object::new();
        js_sys::Reflect::set(&output, &"type".into(), &(result.result_type as u8).into())?;
        js_sys::Reflect::set(&output, &"cancel".into(), &result.cancel.into())?;
        let key = result
            .key
            .map(|key| JsValue::from(to_js_string(&key)))
            .unwrap_or(JsValue::UNDEFINED);
        js_sys::Reflect::set(&output, &"key".into(), &key)?;
        Ok(output.into())
    }
}

#[wasm_bindgen_test]
fn kitty_raw_lookup_values_match_source_in_this_realm() {
    let factory = Closure::<dyn Fn() -> JsValue>::new(|| {
        KittyProbe {
            tables: KeyboardTables::default(),
        }
        .into()
    });
    let result = compare(factory.as_ref().unchecked_ref()).unwrap();
    assert_eq!(
        js_sys::Reflect::get(&result, &"cases".into())
            .unwrap()
            .as_f64(),
        Some(10224.0)
    );
    let failures = js_sys::Array::from(&js_sys::Reflect::get(&result, &"failures".into()).unwrap());
    assert_eq!(
        failures.length(),
        0,
        "{}",
        js_sys::JSON::stringify(&result)
            .unwrap()
            .as_string()
            .unwrap()
    );
}
