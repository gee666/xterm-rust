#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{search_addon::SearchAddon, terminal::Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn public_highlight_bounds_source_counterexample() -> Result<(), JsValue> {
    let term: JsValue = Terminal::new(js_sys::JSON::parse(r#"{"cols":20,"rows":3}"#)?)?.into();
    let addon: JsValue = SearchAddon::new(JsValue::UNDEFINED)?.into();
    let body = format!(
        r#"
term.loadAddon=function(addon){{return this._loadAddon(this,addon);}};
const parent=document.createElement('div');document.body.appendChild(parent);
const write=data=>new Promise(resolve=>term.write(data,resolve));
const equal=(a,b)=>{{if(JSON.stringify(a)!==JSON.stringify(b))throw Error('actual '+JSON.stringify(a)+' expected '+JSON.stringify(b));}};
term.loadAddon(addon);term.open(parent);
return (async()=>{{try{{
{}
return await checkSearchCallArgs(term,addon,write,equal);
}}finally{{term.dispose();parent.remove();}}}})();
"#,
        include_str!("fixtures/search-highlight-bounds.js")
    );
    let promise = Function::new_with_args("term,addon", &body)
        .call2(&JsValue::UNDEFINED, &term, &addon)?
        .dyn_into::<Promise>()?;
    JsFuture::from(promise).await?;
    Ok(())
}
