#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Shared unchanged-source traces run through real Rust Terminal/AttachAddon.
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::bindings::AttachAddon};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn socket_removal_errors_repeat_calls_and_retained_callbacks_match_source(
) -> Result<(), JsValue> {
    let terminal: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = AttachAddon::new(Object::new().unchecked_into(), JsValue::UNDEFINED)
        .unwrap()
        .into();
    let body = format!(
        r#"
{}
return (async()=>{{
  const create=(socket,bidirectional)=>{{
    const term=new Terminal();
    const addon=new AttachAddon(socket,{{bidirectional}});
    return {{
      activate:()=>addon.activate(term), dispose:()=>addon.dispose(),
      input:value=>term.input(value,true),
      flush:async()=>{{
        await new Promise(resolve=>term.write('',resolve));
        return term.buffer.active.getLine(0).translateToString(true);
      }},
      free:()=>{{term.dispose();addon.free();term.free();}}
    }};
  }};
  await attachRepeatedSuccess(create);
  const actual={{rows:await attachRemovalCases(create),registration:await attachRetainedRegistration(create),rawErrors:await attachRawRemovalErrors(create),literalArguments:await attachLiteralArguments(create),nonCallable:await attachNonCallableMethods(create)}};
  const expected={};
  if(JSON.stringify(actual)!==JSON.stringify(expected)){{
    const mismatch=actual.rows.findIndex((row,index)=>JSON.stringify(row)!==JSON.stringify(expected.rows[index]));
    throw Error('Attach removal mismatch '+JSON.stringify({{actual:actual.rows[mismatch],expected:expected.rows[mismatch],registration:actual.registration}}));
  }}
}})();
"#,
        include_str!("fixtures/attach-removal.js"),
        include_str!("fixtures/attach-removal.json")
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
    // Return host failures through the harness without a second Rust panic.
    result.map(|_| ())
}
