// Copyright (c) 2023, 2026 The xterm.js authors. MIT license.
use crate::{
    raw_pixel::{errors, limit, pixel_pair, same_log, set},
    support::*,
};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn raw_pixel_iip_and_kitty_relational_primitives_preserve_actual_pixels() {
    for (value, iip_insert, kitty_insert) in [
        ("undefined", false, true),
        ("null", false, false),
        ("false", false, false),
        ("true", false, true),
        ("0", false, false),
        ("-0", false, false),
        ("NaN", false, true),
        ("Infinity", true, true),
        ("-Infinity", false, false),
        ("-1", false, false),
        ("'1000'", true, true),
        ("'no'", false, true),
        ("[]", false, false),
        ("[1000]", true, true),
        ("new Number(1000)", true, true),
        ("0n", false, false),
        ("1n", false, true),
        ("4n", false, true),
        ("5n", true, true),
    ] {
        for (text, inserted) in [(iip(), iip_insert), (kitty(), kitty_insert)] {
            let pair = pixel_pair(&format!(
                "({{sixelSupport:false,showPlaceholder:false,pixelLimit:{value}}})"
            ));
            pair.write(text).await;
            pair.compare();
            assert_eq!(
                get(&pair.events, "count"),
                JsValue::from(u32::from(inserted)),
                "{value}: {text}"
            );
            if inserted {
                pair.compare_image(0.0, 0.0);
            }
        }
    }
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_iip_rechecks_after_resize_and_skips_nonimage_paths() {
    let pair=pixel_pair("({sixelSupport:false,showPlaceholder:false,pixelLimit:{n:0,log:[],[Symbol.toPrimitive](h){this.log.push(h);return ++this.n===1?1000:1}}})");
    for text in [
        "\x1b]1337;File=inline=1:AAAA\x07",
        "\x1b]1337;File=inline=0:AAAA\x07",
    ] {
        pair.write(text).await;
        pair.compare();
    }
    same_log(&pair, "[]");
    pair.write(iip()).await;
    pair.compare();
    same_log(&pair, "[\"number\",\"number\"]");
    assert_eq!(get(&pair.events, "count"), JsValue::from(0));
    let pair=pixel_pair("({sixelSupport:false,showPlaceholder:false,pixelLimit:{log:[],[Symbol.toPrimitive](h){this.log.push(h);return 1}}})");
    pair.write(iip()).await;
    pair.compare();
    same_log(&pair, "[\"number\"]");
    assert_eq!(get(&pair.events, "count"), JsValue::from(0));
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_iip_throws_at_each_comparison_without_catching_payload_error() {
    for value in [
        "Symbol('pixel')",
        "({token:{},[Symbol.toPrimitive](){throw this.token}})",
        "({[Symbol.toPrimitive](){return {}}})",
        "({token:{},n:0,[Symbol.toPrimitive](){if(++this.n===2)throw this.token;return 1000}})",
    ] {
        let pair = pixel_pair(&format!(
            "({{sixelSupport:false,showPlaceholder:false,pixelLimit:{value}}})"
        ));
        errors(&pair, iip());
        assert_eq!(get(&pair.events, "count"), JsValue::from(0));
        pair.compare();
    }
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_kitty_rechecks_actual_offset_bitmap_and_catches_limit_errors() {
    let offset = "\x1b_Ga=T,f=32,s=1,v=1,i=42,X=1,Y=1;AQID/w==\x1b\\";
    for (value,inserted,log) in [
        ("({log:[],n:0,[Symbol.toPrimitive](h){this.log.push(h);return ++this.n===1?1000:1}})",false,"[\"number\",\"number\"]"),
        ("({log:[],[Symbol.toPrimitive](h){this.log.push(h);return 1000}})",true,"[\"number\",\"number\"]"),
        ("({log:[],[Symbol.toPrimitive](h){this.log.push(h);throw {}}})",false,"[\"number\"]"),
        ("({log:[],n:0,[Symbol.toPrimitive](h){this.log.push(h);if(++this.n===2)throw {};return 1000}})",false,"[\"number\",\"number\"]"),
    ] {
        let pair=pixel_pair(&format!("({{sixelSupport:false,showPlaceholder:false,pixelLimit:{value}}})"));
        pair.write(offset).await;pair.compare();same_log(&pair,log);
        assert_eq!(get(&pair.events,"count"),JsValue::from(u32::from(inserted)));
        if inserted {pair.compare_image(0.0,0.0);}
    }
    let pair =
        pixel_pair("({sixelSupport:false,showPlaceholder:false,pixelLimit:Symbol('pixel')})");
    pair.write(kitty()).await;
    pair.compare();
    assert_eq!(get(&pair.events, "count"), JsValue::from(0));
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_comparisons_can_reset_or_dispose_at_live_protocol_boundaries() {
    for text in [iip(), kitty()] {
        for dispose in [false, true] {
            let pair=Rc::new(pixel_pair("({sixelSupport:false,showPlaceholder:false,pixelLimit:{callback:null,log:[],[Symbol.toPrimitive](h){this.log.push(h);this.callback();return 0}}})"));
            let weak = Rc::downgrade(&pair);
            let callback = Closure::<dyn Fn()>::new(move || {
                let pair = weak.upgrade().unwrap();
                if dispose {
                    pair.addon.dispose().unwrap()
                } else {
                    pair.addon.reset().unwrap();
                }
            });
            set(&limit(&pair, true), "callback", callback.as_ref());
            let callback_source =
                js_sys::Function::new_with_args("a,d", "return ()=>d?a.dispose():a.reset()")
                    .call2(
                        &JsValue::UNDEFINED,
                        &get(&pair.source, "addon"),
                        &dispose.into(),
                    )
                    .unwrap();
            set(&limit(&pair, false), "callback", &callback_source);
            pair.write(text).await;
            pair.compare();
            same_log(&pair, "[\"number\"]");
            assert_eq!(get(&pair.events, "count"), JsValue::from(0));
            for rust in [false, true] {
                set(&limit(&pair, rust), "callback", &JsValue::NULL);
            }
        }
    }
}
