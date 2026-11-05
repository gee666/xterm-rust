#![cfg(target_arch = "wasm32")]
//! Shared BufferSource/host-order traces and an actual Rust terminal OSC52 write.
use js_sys::Function;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::bindings::Base64};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn decode_host_operations_match_unchanged_source() {
    let codec: JsValue = Base64::new().into();
    let source = format!(
        r#"
{}
const actual=base64DecodeCases(()=>new codec.constructor());
const expected={};
if(JSON.stringify(actual)!==JSON.stringify(expected)){{
  throw Error('decodeText source mismatch\nactual '+JSON.stringify(actual)+'\nexpected '+JSON.stringify(expected));
}}
"#,
        include_str!("fixtures/base64-decode.js"),
        include_str!("fixtures/base64-decode.json")
    );
    let result = Function::new_with_args("codec", &source).call1(&JsValue::UNDEFINED, &codec);
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}

#[wasm_bindgen_test]
async fn modern_buffer_source_reaches_live_clipboard_provider() {
    let terminal: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = xterm_web::utility_addons::bindings::ClipboardAddon::new(
        JsValue::UNDEFINED,
        JsValue::UNDEFINED,
    )
    .into();
    let source = r#"
return (async()=>{
  const original=Object.getOwnPropertyDescriptor(Uint8Array,'fromBase64');
  const writes=[];
  const instance=new addon.constructor(undefined,{writeText(selection,text){writes.push([selection,text]);}});
  try {
    Object.defineProperty(Uint8Array,'fromBase64',{configurable:true,value(data){
      if(data!=='QQ==')throw Error('wrong payload '+data);
      return new DataView(new Uint8Array([88,65,226,130,172,89]).buffer,1,4);
    }});
    term._loadAddon(term,instance);
    await new Promise(resolve=>term.write('\x1b]52;c;QQ==\x07',resolve));
    const actual=JSON.stringify(writes), expected=JSON.stringify([['c','A€']]);
    if(actual!==expected)throw Error('live OSC52 write mismatch '+actual+' expected '+expected);
  } finally {
    if(original)Object.defineProperty(Uint8Array,'fromBase64',original);
    else delete Uint8Array.fromBase64;
    term.dispose();addon.free();
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
