#![cfg(target_arch = "wasm32")]
//! Same 17 raw clear() traces as the unchanged source addon, this time through
//! real Rust Terminal parser registrations, OSC writes and browser bindings.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{status_addons::ProgramStatusAddon, terminal::Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn clear_coercion_matches_unchanged_source() {
    let term: JsValue = Terminal::new(JsValue::UNDEFINED).unwrap().into();
    let addon: JsValue = ProgramStatusAddon::new().into();
    let source = format!(
        r#"
{}
return (async()=>{{
  const Term=term.constructor, Addon=addon.constructor;
  term.dispose();addon.dispose();
  const terminals=new WeakMap();
  const actual=await statusClearCases(()=>{{
    const terminal=new Term(), addon=new Addon();
    const parent=document.createElement('div');
    parent.style.cssText='width:800px;height:450px';document.body.appendChild(parent);
    try{{terminal.open(parent);terminal._loadAddon(terminal,addon);}}
    catch(error){{terminal.dispose();parent.remove();throw error;}}
    terminals.set(addon,terminal);
    return {{addon,close(){{try{{terminal.dispose();}}finally{{parent.remove();}}}}}};
  }},(addon,data)=>new Promise(resolve=>terminals.get(addon).write('\x1b]7501;'+data+'\x07',resolve)));
  const expected={};
  if(JSON.stringify(actual)!==JSON.stringify(expected)){{
    throw Error('clear() source mismatch\nactual '+JSON.stringify(actual)+'\nexpected '+JSON.stringify(expected));
  }}
}})();
"#,
        include_str!("fixtures/status-clear.js"),
        include_str!("fixtures/status-clear.json")
    );
    let promise = Function::new_with_args("term,addon", &source)
        .call2(&JsValue::UNDEFINED, &term, &addon)
        .unwrap()
        .dyn_into::<Promise>()
        .unwrap();
    let result = JsFuture::from(promise).await;
    if let Err(error) = &result {
        web_sys::console::error_1(error);
    }
    assert!(result.is_ok());
}
