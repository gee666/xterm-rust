#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Object, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_core::headless::Terminal;
use xterm_web::{js_error::to_js_error, terminal::parser_api::WebParserApi, utf16::from_js_string};

#[wasm_bindgen(inline_js = "
export function callback(log, label, handled, asynchronous) {
  return (...args) => asynchronous ? Promise.resolve().then(() => {log.push([label,args]);return handled;}) : (log.push([label,args]),handled);
}
export function throwingCallback(error) {return () => {throw error;};}
export function rejectingCallback(error) {return () => Promise.reject(error);}
export function reentrantCallback(api, family, log) {
  const weak=new WeakRef(api);
  const methods=['registerCsiHandler','registerDcsHandler','registerEscHandler','registerOscHandler','registerApcHandler'];
  let installed=false;
  return () => {
    log.push('first');
    if(!installed) {
      installed=true;
      weak.deref()[methods[family]](family===3?666:{final:'Z'},()=>{log.push('second');return true;});
    }
    return false;
  };
}
")]
extern "C" {
    fn callback(log: &Array, label: &str, handled: bool, asynchronous: bool) -> Function;
    #[wasm_bindgen(js_name = throwingCallback)]
    fn throwing_callback(error: &JsValue) -> Function;
    #[wasm_bindgen(js_name = rejectingCallback)]
    fn rejecting_callback(error: &JsValue) -> Function;
    #[wasm_bindgen(js_name = reentrantCallback)]
    fn reentrant_callback(api: &JsValue, family: u32, log: &Array) -> Function;
}
fn identifier() -> JsValue {
    let id = Object::new();
    Reflect::set(&id, &"final".into(), &"Z".into()).unwrap();
    id.into()
}
fn labels(log: &Array) -> Vec<String> {
    log.iter()
        .map(|entry| Array::from(&entry).get(0).as_string().unwrap())
        .collect()
}

#[wasm_bindgen_test(async)]
async fn all_families_preserve_async_boolean_bubbling_and_owned_arguments() {
    for family in 0..5 {
        let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
        let api = WebParserApi::new(terminal.clone());
        let log = Array::new();
        for label in ["a", "b", "c"] {
            let cb = callback(&log, label, false, label == "b");
            match family {
                0 => {
                    api.register_csi_handler(identifier(), cb).unwrap();
                }
                1 => {
                    api.register_dcs_handler(identifier(), cb).unwrap();
                }
                2 => {
                    api.register_esc_handler(identifier(), cb).unwrap();
                }
                3 => {
                    api.register_osc_handler(666.0, cb).unwrap();
                }
                4 => {
                    api.register_apc_handler(identifier(), cb).unwrap();
                }
                _ => unreachable!(),
            }
        }
        let sequence = [
            "\x1b[1;2Zafter",
            "\x1bP1;2Zdata\x1b\\after",
            "\x1bZafter",
            "\x1b]666;data\x07after",
            "\x1b_Zdata\x1b\\after",
        ][family];
        assert!(terminal.write(sequence).is_err());
        assert_eq!(labels(&log), ["c"]);
        let pending = terminal.take_pending().unwrap().unwrap();
        let handled = pending.resolve().await.unwrap();
        assert!(!handled);
        terminal.resume(handled).unwrap();
        assert_eq!(labels(&log), ["c", "b", "a"]);
        let arguments = Array::from(&Array::from(&log.get(0)).get(1));
        if family == 0 || family == 1 {
            let params = Array::from(&arguments.get(if family == 0 { 0 } else { 1 }));
            assert_eq!(params.length(), 2);
            assert_eq!(params.get(0).as_f64(), Some(1.0));
            assert_eq!(params.get(1).as_f64(), Some(2.0));
        }
        if family == 1 || family >= 3 {
            assert_eq!(arguments.get(0).as_string().as_deref(), Some("data"));
        }
        if family == 2 {
            assert_eq!(arguments.length(), 0);
        }
        let line = terminal
            .buffer()
            .unwrap()
            .normal()
            .get_line(0)
            .unwrap()
            .unwrap();
        assert_eq!(
            line.borrow().translate_to_string(true, None, None, None),
            "after".encode_utf16().collect::<Vec<_>>()
        );
    }
}

#[wasm_bindgen_test]
fn apc_payload_crosses_js_boundary_losslessly_and_disposal_unregisters() {
    let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
    let api = WebParserApi::new(terminal.clone());
    let log = Array::new();
    let id = identifier();
    Reflect::set(&id, &"prefix".into(), &"?".into()).unwrap();
    let handle = api
        .register_apc_handler(id.clone(), callback(&log, "apc", true, false))
        .unwrap();
    assert!(Reflect::get(&id, &"prefix".into()).unwrap().is_undefined());
    let data = [0x1b, 0x5f, 0x5a, 0xd83d, 0xde00, 0x1b, 0x5c];
    terminal.write_utf16(&data).unwrap();
    let entry = Array::from(&log.get(0));
    let args = Array::from(&entry.get(1));
    assert_eq!(from_js_string(&args.get(0).into()), [0xd83d, 0xde00]);
    let dispose = Reflect::get(&handle, &"dispose".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();
    dispose.call0(&JsValue::UNDEFINED).unwrap();
    dispose.call0(&JsValue::UNDEFINED).unwrap();
    terminal.write_utf16(&data).unwrap();
    assert_eq!(log.length(), 1);
}

#[wasm_bindgen_test]
fn javascript_callbacks_can_register_more_public_handlers_reentrantly() {
    let sequences = [
        "\x1b[Z",
        "\x1bPZpayload\x1b\\",
        "\x1bZ",
        "\x1b]666;payload\x07",
        "\x1b_Zpayload\x1b\\",
    ];
    for (family, sequence) in sequences.iter().enumerate() {
        let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
        let api: JsValue = WebParserApi::new(terminal.clone()).into();
        let log = Array::new();
        let callback = reentrant_callback(&api, u32::try_from(family).unwrap(), &log);
        let name = [
            "registerCsiHandler",
            "registerDcsHandler",
            "registerEscHandler",
            "registerOscHandler",
            "registerApcHandler",
        ][family];
        let id = if family == 3 {
            JsValue::from_f64(666.0)
        } else {
            identifier()
        };
        Reflect::get(&api, &name.into())
            .unwrap()
            .unchecked_into::<Function>()
            .call2(&api, &id, &callback)
            .unwrap();
        terminal.write(sequence).unwrap();
        terminal.write(sequence).unwrap();
        assert_eq!(
            log.iter()
                .map(|value| value.as_string().unwrap())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        terminal.dispose().unwrap();
    }
}

#[wasm_bindgen_test]
fn identifier_error_messages_and_read_only_apc_mutation_match_source() {
    let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
    let api = WebParserApi::new(terminal);
    for (final_byte, expected) in [
        ("", "final must be a single byte"),
        ("aa", "final must be a single byte"),
        ("?", "final must be in range 64 .. 126"),
    ] {
        let id = Object::new();
        Reflect::set(&id, &"final".into(), &final_byte.into()).unwrap();
        let error = api
            .register_csi_handler(id.into(), callback(&Array::new(), "never", false, false))
            .err()
            .unwrap();
        assert_eq!(
            Reflect::get(&error, &"message".into())
                .unwrap()
                .as_string()
                .as_deref(),
            Some(expected)
        );
    }
    let frozen = Object::new();
    Reflect::set(&frozen, &"final".into(), &"Z".into()).unwrap();
    let frozen = Object::freeze(&frozen);
    let error = api
        .register_apc_handler(
            frozen.into(),
            callback(&Array::new(), "never", false, false),
        )
        .err()
        .unwrap();
    assert!(error.is_instance_of::<js_sys::TypeError>());
}

#[wasm_bindgen_test(async)]
async fn thrown_and_rejected_callback_values_keep_identity_through_native_errors() {
    for asynchronous in [false, true] {
        let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
        let api = WebParserApi::new(terminal.clone());
        let error: JsValue = Object::new().into();
        let cb = if asynchronous {
            rejecting_callback(&error)
        } else {
            throwing_callback(&error)
        };
        api.register_csi_handler(identifier(), cb).unwrap();
        let failure = terminal.write("\x1b[Zafter").unwrap_err();
        if asynchronous {
            let pending = terminal.take_pending().unwrap().unwrap();
            let failure = pending.resolve().await.unwrap_err();
            assert_eq!(to_js_error(&failure), error);
        } else {
            assert_eq!(to_js_error(&failure), error);
        }
        assert_eq!(terminal.buffer().unwrap().normal().cursor_x().unwrap(), 0);
    }
}
