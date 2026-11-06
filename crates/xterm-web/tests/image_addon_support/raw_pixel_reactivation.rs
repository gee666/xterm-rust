// Copyright (c) 2020, 2023, 2026 The xterm.js authors. MIT license.
use crate::{
    raw_pixel::{limit, pixel_pair, same_log, set},
    support::*,
};
use std::{
    cell::Cell,
    rc::{Rc, Weak},
};
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use xterm_web::Terminal;

// Cleanup must also run when the old-context control fails its comparison.
struct Secondary {
    pair: Weak<Pair>,
    terminal: Terminal,
    parent: web_sys::Element,
    source: JsValue,
}
impl Drop for Secondary {
    fn drop(&mut self) {
        if let Some(pair) = self.pair.upgrade() {
            for rust in [false, true] {
                set(&limit(&pair, rust), "callback", &JsValue::NULL);
            }
            pair.addon.dispose().unwrap();
            call(&get(&pair.source, "addon"), "dispose", &[]);
        }
        self.terminal.dispose().unwrap();
        call(&self.source, "dispose", &[]);
        self.parent.remove();
    }
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_activation_reentry_keeps_sixel_context_but_reads_later_protocol_contexts_live(
) -> Result<(), JsValue> {
    let pair=Rc::new(pixel_pair("({showPlaceholder:false,pixelLimit:{callback:null,log:[],[Symbol.toPrimitive](h){this.log.push(h);this.callback?.();return 16777216}}})"));
    pair.ready().await;
    let terminal = Terminal::new(eval(
        "({cols:20,rows:10,fontSize:14,allowProposedApi:true,logLevel:'off'})",
    ))
    .unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document.create_element("div").unwrap();
    parent
        .set_attribute("style", "position:fixed;top:0;left:25%")
        .unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let source=eval("(()=>{const terminal=new SourceTerminal({cols:20,rows:10,fontSize:14,allowProposedApi:true,logLevel:'off'});const parent=document.body.appendChild(document.createElement('div'));parent.style.cssText='position:fixed;top:0;left:75%';terminal.open(parent);return {terminal,parent,dispose(){terminal.dispose();parent.remove()}}})()");
    let second = Rc::new(Secondary {
        pair: Rc::downgrade(&pair),
        terminal,
        parent,
        source,
    });
    let callback = Closure::<dyn Fn()>::new({
        let weak = Rc::downgrade(&pair);
        let second = second.clone();
        let called = Cell::new(false);
        move || {
            if !called.replace(true) {
                weak.upgrade()
                    .unwrap()
                    .addon
                    .activate(&second.terminal)
                    .unwrap();
            }
        }
    });
    set(&limit(&pair, true), "callback", callback.as_ref());
    let source_callback = js_sys::Function::new_with_args(
        "addon,terminal",
        "let called=false;return ()=>{if(!called){called=true;addon.activate(terminal)}}",
    )
    .call2(
        &JsValue::UNDEFINED,
        &get(&pair.source, "addon"),
        &get(&second.source, "terminal"),
    )
    .unwrap();
    set(&limit(&pair, false), "callback", &source_callback);
    pair.addon.activate(&pair.terminal).unwrap();
    call(
        &get(&pair.source, "addon"),
        "activate",
        &[get(&pair.source, "terminal")],
    );
    pair.ready().await;
    same_log(&pair, "[\"number\",\"number\",\"number\"]");
    // The outer handler keeps its original Terminal, but later source
    // constructors use the renderer/storage installed by nested activation.
    pair.write(iip()).await;
    let actual = pair.addon.storage_usage()?;
    let expected = get(&get(&pair.source, "addon"), "storageUsage")
        .as_f64()
        .unwrap();
    if actual != expected {
        return Err(js_sys::Error::new(&format!(
            "IIP after activation must use current storage: {actual} != {expected}"
        ))
        .into());
    }
    pair.compare();
    assert_eq!(get(&pair.events, "count"), JsValue::from(1));
    assert!(pair.addon.storage_usage().unwrap() > 0.0);
    pair.compare_image(0.0, 0.0);
    pair.write("\x1b[3;1H").await;
    pair.write(kitty()).await;
    pair.compare();
    assert_eq!(get(&pair.events, "count"), JsValue::from(2));
    drop(callback);
    drop(second);
    Ok(())
}
