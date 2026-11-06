#![cfg(target_arch = "wasm32")]
//! Public setter host operations shared with the unchanged source addon.
use js_sys::{Function, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{status_addons::ProgressAddon, terminal::Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn public_progress_math_matches_unchanged_source() -> Result<(), JsValue> {
    let term: JsValue = Terminal::new(JsValue::UNDEFINED)?.into();
    let addon: JsValue = ProgressAddon::new().into();
    let source = format!(
        r#"
{}
return (async()=>{{
  const Term=term.constructor, Addon=addon.constructor;
  term.dispose();addon.dispose();
  const actual=await progressMathCases(()=>{{
    const terminal=new Term(), addon=new Addon();
    const parent=document.createElement('div');
    parent.style.cssText='width:800px;height:450px';document.body.appendChild(parent);
    try{{terminal.open(parent);terminal._loadAddon(terminal,addon);}}
    catch(error){{terminal.dispose();parent.remove();throw error;}}
    return {{addon,close(){{try{{terminal.dispose();}}finally{{parent.remove();}}}}}};
  }});
  const expected={};
  if(JSON.stringify(actual)!==JSON.stringify(expected)){{
    throw Error('Progress Math mismatch\nactual '+JSON.stringify(actual)+'\nexpected '+JSON.stringify(expected));
  }}
}})();
"#,
        include_str!("fixtures/progress-math.js"),
        include_str!("fixtures/progress-math.json")
    );
    let promise = Function::new_with_args("term,addon", &source)
        .call2(&JsValue::UNDEFINED, &term, &addon)?
        .dyn_into::<Promise>()?;
    JsFuture::from(promise).await?;
    Ok(())
}
