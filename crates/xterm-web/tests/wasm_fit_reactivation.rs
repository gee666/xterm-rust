#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::Function;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::{terminal::Terminal, utility_addons::fit_bindings::FitAddon};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn fit_resizes_the_current_terminal_after_css_callback_reactivation() {
    let prototype: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = FitAddon::new().into();
    let source = concat!(
        include_str!("fixtures/fit-reactivation.js"),
        r#"
const parents=[];
function createPair() {
  return [[13,7,800,450],[17,8,600,350]].map(([cols,rows,width,height])=>{
    const parent=document.createElement('div');
    parent.style.cssText=`width:${width}px;height:${height}px`;
    document.body.appendChild(parent);parents.push(parent);
    const term=new prototype.constructor({cols,rows,scrollback:0,allowProposedApi:true});
    term.open(parent);
    return term;
  });
}
try {
  const actual=fitReactivationCases(()=>new addon.constructor(),createPair,window);
  const expected=JSON.parse(expectedText);
  if(JSON.stringify(actual)!==JSON.stringify(expected))throw Error('source trace mismatch '+JSON.stringify(actual));
} finally {
  for(const parent of parents)parent.remove();
  prototype.dispose();
}
"#
    );
    Function::new_with_args("prototype,addon,expectedText", source)
        .call3(
            &JsValue::UNDEFINED,
            &prototype,
            &addon,
            &JsValue::from_str(include_str!("fixtures/fit-reactivation.json")),
        )
        .unwrap();
}
