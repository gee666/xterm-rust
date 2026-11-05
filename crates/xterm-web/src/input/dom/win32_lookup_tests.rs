// Licensed under MIT. Compare the real Rust core, live DOM reader and host maps.
use super::LiveKeyEvent;
use crate::{input::win32_lookup::Win32Tables, utf16::to_js_string};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::input::win32_input_mode::{lookup::Table, Win32InputMode};

#[wasm_bindgen(module = "/tests/fixtures/win32_prototype_driver.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = compareWin32Prototypes)]
    fn compare(factory: &js_sys::Function) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub struct Win32Probe {
    tables: Win32Tables,
}
#[wasm_bindgen]
impl Win32Probe {
    pub fn table(&self, index: u32) -> Result<JsValue, JsValue> {
        Ok(self.tables.table(match index {
            0 => Table::VirtualKey,
            1 => Table::ScanCode,
            2 => Table::ControlChar,
            _ => return Err(js_sys::RangeError::new("Invalid Win32 test table").into()),
        }))
    }
    pub fn encode(&self, event: JsValue, down: bool) -> Result<js_sys::JsString, JsValue> {
        let event = event.unchecked_into::<web_sys::KeyboardEvent>();
        let result = Win32InputMode
            .try_evaluate_keyboard_event(&LiveKeyEvent(&event, &self.tables), down)?;
        Ok(to_js_string(
            &result.key.expect("Win32 always encodes a record"),
        ))
    }
}

#[wasm_bindgen_test]
fn win32_prototype_values_and_coercion_match_source_in_this_browser_realm() {
    let factory = Closure::<dyn Fn() -> JsValue>::new(|| {
        Win32Probe {
            tables: Win32Tables::default(),
        }
        .into()
    });
    let result = compare(factory.as_ref().unchecked_ref()).unwrap();
    let cases = js_sys::Reflect::get(&result, &"cases".into())
        .unwrap()
        .as_f64()
        .unwrap();
    assert_eq!(cases, 212.0);
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
    let independent = js_sys::Reflect::get(&result, &"independent".into()).unwrap();
    assert_eq!(
        js_sys::Reflect::get(&independent, &"cases".into())
            .unwrap()
            .as_f64(),
        Some(380.0)
    );
    let failures =
        js_sys::Array::from(&js_sys::Reflect::get(&independent, &"failures".into()).unwrap());
    assert_eq!(
        failures.length(),
        0,
        "{}",
        js_sys::JSON::stringify(&independent)
            .unwrap()
            .as_string()
            .unwrap()
    );
}
