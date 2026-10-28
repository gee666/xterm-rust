#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::terminal::addon_manager::BrowserAddonManager;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function addon(label, log, activateError, disposeError) {
  return {
    activate(t) { log.push('activate:' + label + ':' + t.cols); if (activateError) throw activateError; },
    dispose() { log.push('dispose:' + label); if (disposeError) throw disposeError; }
  };
}
export function recursiveAddon(log) {
  const a = {activate() {},dispose() {log.push('self');a.dispose();}};
  return a;
}
export function terminalObject() {return {cols:5};}
")]
extern "C" {
    fn addon(
        label: &str,
        log: &Array,
        activate_error: &JsValue,
        dispose_error: &JsValue,
    ) -> JsValue;
    #[wasm_bindgen(js_name = recursiveAddon)]
    fn recursive_addon(log: &Array) -> JsValue;
    #[wasm_bindgen(js_name = terminalObject)]
    fn terminal_object() -> JsValue;
}
fn dispose(addon: &JsValue) -> Result<JsValue, JsValue> {
    Reflect::get(addon, &"dispose".into())?
        .dyn_into::<Function>()?
        .call0(addon)
}
fn log_values(log: &Array) -> Vec<String> {
    log.iter().map(|value| value.as_string().unwrap()).collect()
}

#[wasm_bindgen_test]
fn addon_activation_reverse_order_and_direct_disposal_match_original() {
    let manager = BrowserAddonManager::new();
    let log = Array::new();
    let terminal = terminal_object();
    let a = addon("a", &log, &JsValue::UNDEFINED, &JsValue::UNDEFINED);
    let b = addon("b", &log, &JsValue::UNDEFINED, &JsValue::UNDEFINED);
    let c = addon("c", &log, &JsValue::UNDEFINED, &JsValue::UNDEFINED);
    manager.load_addon(&terminal, &a).unwrap();
    manager.load_addon(&terminal, &b).unwrap();
    manager.load_addon(&terminal, &c).unwrap();
    dispose(&b).unwrap();
    dispose(&b).unwrap();
    manager.dispose().unwrap();
    manager.dispose().unwrap();
    assert_eq!(
        log_values(&log),
        [
            "activate:a:5",
            "activate:b:5",
            "activate:c:5",
            "dispose:b",
            "dispose:c",
            "dispose:a"
        ]
    );
}

#[wasm_bindgen_test]
fn original_callback_exceptions_keep_identity_and_partial_cleanup_state() {
    let manager = BrowserAddonManager::new();
    let log = Array::new();
    let terminal = terminal_object();
    let error: JsValue = js_sys::Object::new().into();
    for (label, throw) in [("a", false), ("b", true), ("c", false)] {
        let item = addon(
            label,
            &log,
            &JsValue::UNDEFINED,
            if throw { &error } else { &JsValue::UNDEFINED },
        );
        manager.load_addon(&terminal, &item).unwrap();
    }
    assert_eq!(manager.dispose().err().unwrap(), error);
    manager.dispose().unwrap();
    manager.dispose().unwrap();
    assert_eq!(
        log_values(&log),
        [
            "activate:a:5",
            "activate:b:5",
            "activate:c:5",
            "dispose:c",
            "dispose:b",
            "dispose:a"
        ]
    );
    let failed = addon("failed", &log, &error, &JsValue::UNDEFINED);
    assert_eq!(manager.load_addon(&terminal, &failed).err().unwrap(), error);
    manager.dispose().unwrap();
    assert_eq!(log_values(&log).last().unwrap(), "dispose:failed");
}

#[wasm_bindgen_test]
fn duplicate_instances_cross_manager_wrappers_dispose_original_once() {
    let first = BrowserAddonManager::new();
    let second = BrowserAddonManager::new();
    let log = Array::new();
    let item = addon("shared", &log, &JsValue::UNDEFINED, &JsValue::UNDEFINED);
    let terminal = terminal_object();
    first.load_addon(&terminal, &item).unwrap();
    second.load_addon(&terminal, &item).unwrap();
    first.dispose().unwrap();
    second.dispose().unwrap();
    dispose(&item).unwrap();
    assert_eq!(
        log_values(&log),
        ["activate:shared:5", "activate:shared:5", "dispose:shared"]
    );
}

#[wasm_bindgen_test]
fn self_disposal_and_saved_method_after_manager_drop_keep_source_semantics() {
    let manager = BrowserAddonManager::new();
    let log = Array::new();
    let item = recursive_addon(&log);
    manager.load_addon(&terminal_object(), &item).unwrap();
    manager.dispose().unwrap();
    assert_eq!(log_values(&log), ["self"]);
    let item = addon("saved", &log, &JsValue::UNDEFINED, &JsValue::UNDEFINED);
    manager.load_addon(&terminal_object(), &item).unwrap();
    let saved = Reflect::get(&item, &"dispose".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();
    drop(manager);
    drop(item);
    saved.call0(&JsValue::UNDEFINED).unwrap();
    saved.call0(&JsValue::UNDEFINED).unwrap();
    assert_eq!(
        log_values(&log),
        ["self", "activate:saved:5", "dispose:saved"]
    );
}
