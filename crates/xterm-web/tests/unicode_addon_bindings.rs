#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Reflect};
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};

// The public browser facade supplies this exact one-line ABI adapter. The raw
// wasm-bindgen class cannot receive its JavaScript wrapper as Rust's &self.
#[wasm_bindgen(inline_js = "export function publicLoadAddon(terminal, addon) { return terminal._loadAddon(terminal, addon); }")]
extern "C" {
    #[wasm_bindgen(catch, js_name = publicLoadAddon)]
    fn public_load_addon(terminal: &JsValue, addon: &JsValue) -> Result<(), JsValue>;
}
use wasm_bindgen_test::*;
use xterm_web::{
    unicode_addons::{Unicode11Addon, UnicodeGraphemesAddon},
    Terminal,
};

wasm_bindgen_test_configure!(run_in_browser);

fn call(receiver: &JsValue, method: &str, argument: Option<&JsValue>) -> Result<JsValue, JsValue> {
    let function: Function = Reflect::get(receiver, &method.into())?.dyn_into()?;
    match argument {
        Some(value) => function.call1(receiver, value),
        None => function.call0(receiver),
    }
}
fn options(proposed: bool) -> JsValue {
    let options = Object::new();
    assert!(Reflect::set(&options, &"allowProposedApi".into(), &proposed.into()).unwrap());
    options.into()
}

#[wasm_bindgen_test]
fn actual_addon_exports_gate_activation_before_any_provider_registration() {
    let terminal = Terminal::new(options(false)).unwrap();
    let eleven = Unicode11Addon::new();
    let graphemes = UnicodeGraphemesAddon::new();
    for error in [
        eleven.activate(&terminal).unwrap_err(),
        graphemes.activate(&terminal).unwrap_err(),
    ] {
        let message = Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .unwrap();
        assert!(message.contains("allowProposedApi"), "{message}");
    }
    eleven.dispose();
    graphemes.dispose().unwrap();
    let value: JsValue = terminal.into();
    call(&value, "dispose", None).unwrap();
}

#[wasm_bindgen_test]
fn javascript_loadaddon_manages_real_provider_activation_and_disposal() {
    let terminal = Terminal::new(options(true)).unwrap();
    let unicode = terminal.shared_native().unicode().unwrap();
    let value: JsValue = terminal.into();
    let eleven: JsValue = Unicode11Addon::new().into();
    let graphemes: JsValue = UnicodeGraphemesAddon::new().into();
    public_load_addon(&value, &eleven).unwrap();
    assert_eq!(unicode.versions(), ["6", "11"]);
    assert_eq!(unicode.active_version(), "6");
    public_load_addon(&value, &graphemes).unwrap();
    assert_eq!(unicode.versions(), ["6", "11", "15", "15-graphemes"]);
    assert_eq!(unicode.active_version(), "15-graphemes");
    for line in include_str!("../../xterm-core/tests/fixtures/unicode-grapheme-strings.tsv").lines()
    {
        let (text, expected) = line.split_once('\t').unwrap();
        let units: Vec<u16> = text
            .split(',')
            .map(|v| u16::from_str_radix(v, 16).unwrap())
            .collect();
        assert_eq!(
            unicode.get_string_cell_width_utf16(&units).unwrap(),
            expected.parse::<i64>().unwrap()
        );
    }
    let options = Reflect::get(&value, &"options".into()).unwrap();
    assert!(Reflect::set(&options, &"allowProposedApi".into(), &JsValue::FALSE).unwrap());
    call(&graphemes, "dispose", None).unwrap();
    assert_eq!(
        unicode.active_version(),
        "6",
        "disposal uses its retained service, not a newly gated getter"
    );
    unicode.set_active_version("11").unwrap();
    call(&graphemes, "dispose", None).unwrap();
    assert_eq!(
        unicode.active_version(),
        "11",
        "managed addon disposal must be idempotent"
    );
    call(&value, "dispose", None).unwrap();
}
