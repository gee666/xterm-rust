// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use crate::{
    raw_kitty_limit::json,
    raw_palette::{palette_pair, set, value},
    raw_sixel_limit::parse,
    support::*,
};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn raw_palette_reply_errors_keep_default_assignment_and_exact_thrown_values() {
    let pair = palette_pair(
        r#"(()=>{
      let reads=0;return {sixelSupport:false,showPlaceholder:false,get sixelPaletteLimit(){
        return ++reads===1?32:{log:[],token:{},[Symbol.toPrimitive](hint){this.log.push(hint);throw this.token}}
      }};
    })()"#,
    );
    let (actual, expected) = parse(&pair, "\x1b[?1;2S");
    assert_eq!(actual.unwrap_err(), get(&value(&pair, true), "token"));
    assert_eq!(expected.unwrap_err(), get(&value(&pair, false), "token"));
    for rust in [true, false] {
        assert_eq!(json(&get(&value(&pair, rust), "log")), "[\"string\"]");
    }
    // Both threw after replacing _opts but before sending the reply.
    pair.compare();
    assert!(pair.replies.borrow().is_empty());
    for rust in [true, false] {
        set(&value(&pair, rust), "token", &JsValue::from("not used"));
        js_sys::Reflect::set(
            &value(&pair, rust),
            &js_sys::Symbol::to_primitive(),
            &js_sys::Function::new_no_args("return 64"),
        )
        .unwrap();
    }
    pair.write("\x1b[?1;1S").await;
    pair.compare();
}

#[wasm_bindgen_test(async)]
async fn raw_palette_hook_and_reset_failures_preserve_prior_phase_state() {
    let pair = palette_pair(
        r#"({showPlaceholder:false,sixelPaletteLimit:{
      fail:false,token:{},log:[],[Symbol.toPrimitive](hint){
        this.log.push(hint);if(this.fail || this.failAt===this.log.length)throw this.token;return 32}
    }})"#,
    );
    pair.ready().await;
    pair.write(sixel()).await;
    for rust in [true, false] {
        set(&value(&pair, rust), "fail", &JsValue::TRUE);
    }
    let (actual, expected) = parse(&pair, "\x1bP0;1q");
    assert_eq!(actual.unwrap_err(), get(&value(&pair, true), "token"));
    assert_eq!(expected.unwrap_err(), get(&value(&pair, false), "token"));
    pair.compare();
    // Release converts the original value first. Fail the second conversion,
    // after release and palette clearing but before the reset init.
    for rust in [true, false] {
        let raw = value(&pair, rust);
        set(&raw, "fail", &JsValue::FALSE);
        let next = get(&get(&raw, "log"), "length").as_f64().unwrap() + 2.0;
        set(&raw, "failAt", &next.into());
    }
    let actual = pair.addon.reset();
    let expected = get(&get(&pair.source, "addon"), "reset")
        .unchecked_into::<js_sys::Function>()
        .call0(&get(&pair.source, "addon"));
    assert_eq!(actual.unwrap_err(), get(&value(&pair, true), "token"));
    assert_eq!(expected.unwrap_err(), get(&value(&pair, false), "token"));
    pair.compare();
    let handler = call(
        &get(&get(&pair.source, "addon"), "_handlers"),
        "get",
        &["sixel".into()],
    );
    let source_palette = get(&get(&handler, "_dec"), "_palette");
    assert!(js_sys::Uint32Array::new(&source_palette)
        .to_vec()
        .iter()
        .all(|v| *v == 0));
    for rust in [true, false] {
        set(&value(&pair, rust), "fail", &JsValue::FALSE);
    }
    // Finish the failed hook with a new complete sequence. Zeroed color #1
    // produces transparent pixels only if the failed reset's clear survived.
    pair.write("\x1b\\\x1bP0;1q\"1;1;2;6#1!2~\x1b\\").await;
    pair.compare();
    pair.compare_image(0.0, 0.0);
    let image = pair.addon.get_image_at_buffer_cell(0.0, 0.0).unwrap();
    assert!(pixels(&image).iter().all(|v| *v == 0));
    assert_eq!(
        json(&get(&value(&pair, true), "log")),
        json(&get(&value(&pair, false), "log"))
    );
}

#[wasm_bindgen_test(async)]
async fn raw_palette_reentry_during_hook_and_reset_uses_live_decoder_without_borrows() {
    let pair = Rc::new(palette_pair(
        r#"({showPlaceholder:false,sixelPaletteLimit:{
      log:[],callback:null,[Symbol.toPrimitive](hint){this.log.push(hint);this.callback?.();return 32}
    }})"#,
    ));
    pair.ready().await;
    let weak = Rc::downgrade(&pair);
    let callback = Closure::<dyn Fn()>::new(move || {
        let pair = weak.upgrade().unwrap();
        // Clear first, so only the outer conversion performs the reentry.
        set(&value(&pair, true), "callback", &JsValue::NULL);
        pair.addon.reset().unwrap();
    });
    let source_callback = js_sys::Function::new_with_args(
        "addon",
        "return ()=>{addon._opts.sixelPaletteLimit.callback=null;addon.reset()}",
    )
    .call1(&JsValue::UNDEFINED, &get(&pair.source, "addon"))
    .unwrap();
    for reset in [false, true] {
        set(&value(&pair, true), "callback", callback.as_ref());
        set(&value(&pair, false), "callback", &source_callback);
        if reset {
            pair.addon.reset().unwrap();
            call(&get(&pair.source, "addon"), "reset", &[]);
        }
        pair.write(sixel()).await;
        pair.compare();
        pair.compare_image(0.0, 0.0);
        assert_eq!(
            json(&get(&value(&pair, true), "log")),
            json(&get(&value(&pair, false), "log"))
        );
    }
    set(&value(&pair, true), "callback", &JsValue::NULL);
}

#[wasm_bindgen_test(async)]
async fn raw_palette_async_initialization_validates_then_converts_in_all_bindings() {
    let api = public_api().await;
    let direct = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&eval(
        "import('/suite.js')",
    )))
    .await
    .unwrap();
    let run = js_sys::Function::new_with_args(
        "apis",
        r#"
      return (async()=>{
        const records=[];
        for(const api of apis){
          const row=[];
          for(const expression of ["4097","4097n","32n","Symbol('palette')",
            "({[Symbol.toPrimitive](){throw token}})",
            "({n:0,[Symbol.toPrimitive](hint){log.push(hint);if(++this.n===2)throw token;return 32}})"]){
            const token={},log=[];
            const value=eval(expression), addon=new api.ImageAddon({
              showPlaceholder:false,sixelPaletteLimit:value,iipSupport:false,kittySupport:false});
            const terminal=new api.Terminal({cols:20,rows:10,allowProposedApi:true,logLevel:'off'});
            let rejected=false,reason;
            const listener=event=>{rejected=true;reason=event.reason;event.preventDefault()};
            addEventListener('unhandledrejection',listener);
            try{
              addon.activate(terminal);
              for(let frame=0;!rejected;frame++){
                if(frame===120)throw Error('missing decoder rejection');
                await new Promise(requestAnimationFrame);
              }
              row.push({identity:reason===token,name:reason?.name,message:reason?.message,log});
            }finally{
              removeEventListener('unhandledrejection',listener);
              addon.dispose();terminal.dispose();addon.free?.();terminal.free?.();
            }
          }
          records.push(row);
        }
        if(records[0][4].identity!==true || records[0][5].identity!==true ||
          JSON.stringify(records[0][5].log)!=='["number","number"]')throw Error('source rejection contract');
        if(records.some(row=>JSON.stringify(row)!==JSON.stringify(records[0])))throw Error(JSON.stringify(records));
      })();
    "#,
    );
    promise(
        run.call1(
            &JsValue::UNDEFINED,
            &js_sys::Array::of3(
                &eval("({ImageAddon:SourceImageAddon,Terminal:SourceTerminal})"),
                &direct,
                &api,
            ),
        )
        .unwrap(),
    )
    .await;
}
use wasm_bindgen::JsCast;
