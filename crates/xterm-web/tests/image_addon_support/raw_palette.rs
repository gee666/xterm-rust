// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use crate::{raw_kitty_limit::json, raw_sixel_limit::parse, support::*};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

pub(super) fn value(pair: &Pair, rust: bool) -> JsValue {
    let options = if rust {
        pair.addon.option_inspection()
    } else {
        get(&get(&pair.source, "addon"), "_opts")
    };
    get(&options, "sixelPaletteLimit")
}
pub(super) fn palette_pair(options: &str) -> Pair {
    // Fail an old-code control before opening any terminal or scheduling work.
    let addon = ImageAddon::new(eval("({sixelPaletteLimit:Symbol('preflight')})")).unwrap();
    addon.dispose().unwrap();
    Pair::new(true, options)
}
pub(super) fn set(object: &JsValue, key: &str, value: &JsValue) {
    js_sys::Reflect::set(object, &key.into(), value).unwrap();
}

#[wasm_bindgen_test(async)]
async fn raw_palette_constructor_and_default_copies_retain_identity_in_all_bindings() {
    let api = public_api().await;
    let direct = promise_module().await;
    let run = js_sys::Function::new_with_args(
        "apis",
        r#"
      for (const api of apis) {
        const values=[undefined,null,false,true,0,-0,NaN,Infinity,-Infinity,1,-1,
          '', '32', 'no', 0n, 32n, Symbol('palette'),{},[],new Number(32),
          document.all,()=>{}, { [Symbol.toPrimitive](){throw Error('eager')} },
          (()=>{const {proxy,revoke}=Proxy.revocable({},{});revoke();return proxy})()];
        for(const value of values){
          const addon=new api.ImageAddon({sixelPaletteLimit:value});
          try {
            for(const action of [()=>{},()=>addon.reset(),()=>addon.dispose()]){
              action();
              if(!Object.is(addon._opts.sixelPaletteLimit,value))throw Error('raw palette identity');
            }
          } finally { addon.dispose(); addon.free?.(); }
        }
        let reads=0;
        const first=Symbol('first'),second={tag:'default'};
        const addon=new api.ImageAddon({get sixelPaletteLimit(){return ++reads===1?first:second}});
        try{
          if(reads!==2 || addon._opts.sixelPaletteLimit!==first)throw Error('copy order');
          addon.reset();
          if(reads!==2 || addon._opts.sixelPaletteLimit!==second)throw Error('reset default');
        }finally{addon.dispose();addon.free?.();}
      }
    "#,
    );
    let apis = js_sys::Array::of3(&eval("({ImageAddon:SourceImageAddon})"), &direct, &api);
    run.call1(&JsValue::UNDEFINED, &apis).unwrap();
}
async fn promise_module() -> JsValue {
    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&eval("import('/suite.js')")))
        .await
        .unwrap()
}

#[wasm_bindgen_test(async)]
async fn raw_palette_disabled_controls_use_string_hint_and_restore_second_copy() {
    let pair = palette_pair(
        r#"(()=>{
      let reads=0;const make=n=>({n,log:[],[Symbol.toPrimitive](hint){this.log.push(hint);return this.n}});
      return {sixelSupport:false,showPlaceholder:false,get sixelPaletteLimit(){return make(++reads*128)}};
    })()"#,
    );
    let original = value(&pair, true);
    let source_original = value(&pair, false);
    for text in [
        "\x1b[?1;1S",
        "\x1b[?1;4S",
        "\x1b[?1;3;4097S",
        "\x1b[?1;3;16:8S",
    ] {
        pair.write(text).await;
        pair.compare();
    }
    assert_eq!(json(&get(&original, "log")), "[\"string\"]");
    assert_eq!(
        json(&get(&original, "log")),
        json(&get(&source_original, "log"))
    );
    pair.write("\x1b[?1;3;16S").await;
    assert_eq!(value(&pair, true), JsValue::from(16));
    pair.write("\x1b[?1;2S").await;
    pair.compare();
    let default = value(&pair, true);
    assert_eq!(get(&default, "n"), JsValue::from(256));
    assert_eq!(json(&get(&default, "log")), "[\"string\"]");
    pair.addon.reset().unwrap();
    call(&get(&pair.source, "addon"), "reset", &[]);
    assert!(js_sys::Object::is(&value(&pair, true), &default));
    pair.compare();

    // Template interpolation retains every UTF-16 unit, including lone surrogates.
    let pair =
        palette_pair("({sixelSupport:false,showPlaceholder:false,sixelPaletteLimit:'a\\ud800b'})");
    pair.write("\x1b[?1;1S").await;
    pair.compare();
    assert!(pair.replies.borrow()[0].contains(&0xd800));
}

#[wasm_bindgen_test(async)]
async fn raw_palette_pending_and_ready_decoder_calls_match_pixels_and_conversion_order() {
    let pair = palette_pair(
        r#"({showPlaceholder:false,sixelPaletteLimit:{
      log:[],n:0,[Symbol.toPrimitive](hint){this.log.push(hint);return ++this.n===2?8192:32}
    }})"#,
    );
    for rust in [true, false] {
        assert_eq!(json(&get(&value(&pair, rust), "log")), "[]");
    }
    let (actual, expected) = parse(&pair, "\x1bP0;1q~\x1b\\");
    actual.unwrap();
    expected.unwrap();
    pair.addon.reset().unwrap();
    call(&get(&pair.source, "addon"), "reset", &[]);
    for rust in [true, false] {
        assert_eq!(json(&get(&value(&pair, rust), "log")), "[]");
    }
    pair.ready().await;
    for rust in [true, false] {
        assert_eq!(
            json(&get(&value(&pair, rust), "log")),
            "[\"number\",\"number\"]"
        );
    }
    for text in [
        sixel(),
        "\x1b[?1;1S",
        "\x1b[?1;3;16S",
        "\x1b[3;1H",
        sixel(),
        "\x1b[?1;2S",
        sixel(),
    ] {
        pair.write(text).await;
        pair.compare();
    }
    // Original numeric state lives in Rust, including palette-index wrapping.
    pair.compare_image(0.0, 0.0);
    assert_eq!(
        json(&get(&value(&pair, true), "log")),
        json(&get(&value(&pair, false), "log"))
    );
}
