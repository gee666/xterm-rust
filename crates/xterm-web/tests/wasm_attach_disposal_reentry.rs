#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Actual Rust Terminal/AttachAddon exports using the shared source socket trace.
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::bindings::AttachAddon};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn disposal_visits_all_handles_appended_by_socket_removal_reentry() {
    let terminal: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = AttachAddon::new(Object::new().unchecked_into(), JsValue::UNDEFINED)
        .unwrap()
        .into();
    let body = format!(
        r#"
{}
return (async()=>{{
  const actual=await attachDisposalReentryCases((socket,bidirectional)=>{{
    const term=new Terminal();
    const addon=new AttachAddon(socket,{{bidirectional}});
    return {{
      activate:()=>addon.activate(term),
      dispose:()=>addon.dispose(),
      input:value=>term.input(value,true),
      flush:async()=>{{
        await new Promise(resolve=>term.write('',resolve));
        return term.buffer.active.getLine(0).translateToString(true);
      }},
      free:()=>{{term.dispose();addon.free();term.free();}}
    }};
  }});
  const expected={};
  if(JSON.stringify(actual)!==JSON.stringify(expected)){{
    const mismatch=actual.findIndex((row,index)=>JSON.stringify(row)!==JSON.stringify(expected[index]));
    throw Error('Attach disposal mismatch '+JSON.stringify({{actual:actual[mismatch],expected:expected[mismatch]}}));
  }}
}})();
"#,
        include_str!("fixtures/attach-disposal-reentry.js"),
        include_str!("fixtures/attach-disposal-reentry.json")
    );
    let result = async {
        let promise = Function::new_with_args("Terminal,AttachAddon", &body).call2(
            &JsValue::UNDEFINED,
            &Reflect::get(&terminal, &"constructor".into())?,
            &Reflect::get(&addon, &"constructor".into())?,
        )?;
        JsFuture::from(Promise::from(promise)).await
    }
    .await;
    let free = Function::new_with_args("value", "value.free();");
    free.call1(&JsValue::UNDEFINED, &addon).unwrap();
    free.call1(&JsValue::UNDEFINED, &terminal).unwrap();
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}
