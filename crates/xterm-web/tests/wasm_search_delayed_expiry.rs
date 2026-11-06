#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{search_addon::SearchAddon, terminal::Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn delayed_cache_expiry_matches_source_in_both_directions() -> Result<(), JsValue> {
    let terminal: JsValue = Terminal::new(js_sys::JSON::parse(
        r#"{"cols":20,"rows":3,"scrollback":1000}"#,
    )?)?
    .into();
    let addon: JsValue = SearchAddon::new(JsValue::UNDEFINED)?.into();
    let body = format!(
        r#"
term.loadAddon=function(addon){{return this._loadAddon(this,addon);}};
const parent=document.createElement('div');parent.style.cssText='width:640px;height:240px;';document.body.appendChild(parent);
const style=document.createElement('style');style.textContent=css;document.head.appendChild(style);
const write=data=>new Promise(resolve=>term.write(data,resolve));
const equal=(actual,expected)=>{{if(JSON.stringify(actual)!==JSON.stringify(expected))throw Error('actual '+JSON.stringify(actual)+' expected '+JSON.stringify(expected));}};
{fixture}
return (async()=>{{try{{term.loadAddon(addon);term.open(parent);equal(await checkSearchCallArgs(term,addon,write,equal),6);}}
finally{{term.dispose();parent.remove();style.remove();}}}})();
"#,
        fixture = include_str!("fixtures/search-delayed-expiry.js")
    );
    let promise = Function::new_with_args("term,addon,css", &body)
        .call3(
            &JsValue::UNDEFINED,
            &terminal,
            &addon,
            &include_str!("../assets/xterm.css").into(),
        )?
        .dyn_into::<Promise>()?;
    JsFuture::from(promise).await?;
    Ok(())
}
