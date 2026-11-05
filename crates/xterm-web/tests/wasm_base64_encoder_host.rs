#![cfg(target_arch = "wasm32")]
//! Unchanged-source encoder host traces and actual Rust OSC52 integration.
use js_sys::{Function, Promise};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::bindings::Base64};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn encoder_host_operations_match_unchanged_source() {
    let codec: JsValue = Base64::new().into();
    let source = format!(
        r#"
{}
const actual=base64EncodeHostCases(()=>new codec.constructor());
const expected={};
if(JSON.stringify(actual)!==JSON.stringify(expected)){{
  throw Error('encoder host mismatch\nactual '+JSON.stringify(actual)+'\nexpected '+JSON.stringify(expected));
}}
"#,
        include_str!("fixtures/base64-encode-host.js"),
        include_str!("fixtures/base64-encode-host.json")
    );
    let result = Function::new_with_args("codec", &source).call1(&JsValue::UNDEFINED, &codec);
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
async fn custom_encoder_receives_provider_result_in_real_osc52_query() {
    let terminal: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = xterm_web::utility_addons::bindings::ClipboardAddon::new(
        JsValue::UNDEFINED,
        JsValue::UNDEFINED,
    )
    .into();
    let source = r#"
return (async()=>{
  const original=Object.getOwnPropertyDescriptor(globalThis,'TextEncoder');
  const raw={provider:true}, log=[], replies=[];
  const instance=new addon.constructor(undefined,{readText(){log.push('read');return raw;}});
  const subscription=term.onData(value=>replies.push(value));
  try {
    Object.defineProperty(globalThis,'TextEncoder',{configurable:true,value:function(){
      log.push('construct');this.encode=function(data){
        log.push(data===raw?'raw provider value':'wrong input');
        return {toBase64(){log.push('encode bytes');return 'encoded-reply';}};
      };
    }});
    term._loadAddon(term,instance);
    await new Promise(resolve=>term.write('\x1b]52;c;?\x07',resolve));
    const actual=JSON.stringify({log,replies});
    const expected=JSON.stringify({log:['read','construct','raw provider value','encode bytes'],replies:['\x1b]52;c;encoded-reply\x07']});
    if(actual!==expected)throw Error('live encoder host mismatch '+actual+' expected '+expected);
  } finally {
    Object.defineProperty(globalThis,'TextEncoder',original);
    subscription.dispose();term.dispose();addon.free();
  }
})();
"#;
    let promise = Function::new_with_args("term,addon", source)
        .call2(&JsValue::UNDEFINED, &terminal, &addon)
        .unwrap();
    let result = wasm_bindgen_futures::JsFuture::from(Promise::from(promise)).await;
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}
