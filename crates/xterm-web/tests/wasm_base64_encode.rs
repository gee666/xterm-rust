#![cfg(target_arch = "wasm32")]
//! Shared source host-method traces plus actual Rust terminal OSC52 reply.
use js_sys::Function;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::bindings::Base64};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn encode_host_operations_match_unchanged_source() {
    let codec: JsValue = Base64::new().into();
    let source = format!(
        r#"
{}
const actual=base64EncodeCases(()=>new codec.constructor());
const expected={};
if(JSON.stringify(actual)!==JSON.stringify(expected)){{
  throw Error('encodeText source mismatch\nactual '+JSON.stringify(actual)+'\nexpected '+JSON.stringify(expected));
}}
"#,
        include_str!("fixtures/base64-encode.js"),
        include_str!("fixtures/base64-encode.json")
    );
    let result = Function::new_with_args("codec", &source).call1(&JsValue::UNDEFINED, &codec);
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
async fn modern_raw_encoded_value_is_interpolated_into_live_osc52_reply() {
    let terminal: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = xterm_web::utility_addons::bindings::ClipboardAddon::new(
        JsValue::UNDEFINED,
        JsValue::UNDEFINED,
    )
    .into();
    let source = r#"
return (async()=>{
  const original=Object.getOwnPropertyDescriptor(Uint8Array.prototype,'toBase64');
  const instance=new addon.constructor(undefined,{readText(){return '€';}});
  const log=[], replies=[];
  const subscription=term.onData(value=>replies.push(value));
  try {
    Object.defineProperty(Uint8Array.prototype,'toBase64',{configurable:true,value(){
      log.push(Array.from(this));
      return {toString(){log.push('interpolate');return 'host-result';}};
    }});
    term._loadAddon(term,instance);
    await new Promise(resolve=>term.write('\x1b]52;c;?\x07',resolve));
    const actual=JSON.stringify({log,replies});
    const expected=JSON.stringify({log:[[226,130,172],'interpolate'],replies:['\x1b]52;c;host-result\x07']});
    if(actual!==expected)throw Error('live OSC52 mismatch '+actual+' expected '+expected);
  } finally {
    if(original)Object.defineProperty(Uint8Array.prototype,'toBase64',original);
    else delete Uint8Array.prototype.toBase64;
    subscription.dispose();term.dispose();addon.free();
  }
})();
"#;
    let promise = Function::new_with_args("term,addon", source)
        .call2(&JsValue::UNDEFINED, &terminal, &addon)
        .unwrap();
    let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await;
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}
