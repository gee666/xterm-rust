// Copyright (c) 2026 The xterm.js authors. MIT license.
use crate::support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

fn handlers(pair: &Pair) -> JsValue {
    pair.addon.handler_inspection()
}
fn source_handlers(pair: &Pair) -> JsValue {
    get(&get(&pair.source, "addon"), "_handlers")
}
fn kitty_handler(map: &JsValue) -> JsValue {
    call(map, "get", &["kitty".into()])
}
fn keys(map: &JsValue) -> JsValue {
    js_sys::JSON::stringify(&js_sys::Array::from(&call(map, "keys", &[])))
        .unwrap()
        .into()
}
async fn bytes(record: &JsValue) -> Vec<u8> {
    let buffer = JsFuture::from(
        call(&get(record, "data"), "arrayBuffer", &[]).unchecked_into::<js_sys::Promise>(),
    )
    .await
    .unwrap();
    js_sys::Uint8Array::new(&buffer).to_vec()
}

#[wasm_bindgen_test]
fn handler_view_is_stable_before_activation_and_disabled_entries_are_absent() {
    let addon = ImageAddon::new(JsValue::UNDEFINED).unwrap();
    let map = addon.handler_inspection();
    assert_eq!(map, addon.handler_inspection());
    assert_eq!(get(&map, "size"), JsValue::from(0));
    assert_eq!(keys(&map), JsValue::from("[]"));
    addon.dispose().unwrap();
    let pair = Pair::new(false, "({sixelSupport:false,kittySupport:false})");
    assert_eq!(keys(&handlers(&pair)), keys(&source_handlers(&pair)));
    assert_eq!(keys(&handlers(&pair)), JsValue::from("[\"iip\"]"));
    for key in [
        "kitty".into(),
        1.into(),
        JsValue::NULL,
        eval("({toString(){throw Error('coercion')}})"),
    ] {
        assert_eq!(
            call(&handlers(&pair), "has", &[key.clone()]),
            call(&source_handlers(&pair), "has", &[key.clone()])
        );
        assert!(call(&handlers(&pair), "get", &[key]).is_undefined());
    }
}

#[wasm_bindgen_test(async)]
async fn kitty_image_views_expose_actual_blobs_and_distinct_replacement_records() {
    let pair = Pair::new(true, "({sixelSupport:false,showPlaceholder:false})");
    let actual = kitty_handler(&handlers(&pair));
    let expected = kitty_handler(&source_handlers(&pair));
    assert_eq!(actual, kitty_handler(&handlers(&pair)));
    let images = get(&actual, "images");
    let source = get(&expected, "images");
    pair.write(kitty()).await;
    pair.write("\x1b_Ga=t,f=32,s=1,v=1,i=43;BQYH/w==\x1b\\")
        .await;
    assert_eq!(keys(&images), keys(&source));
    assert_eq!(keys(&images), JsValue::from("[42,43]"));
    assert_eq!(
        keys(&get(&actual, "_kittyIdToStorageId")),
        keys(&get(&expected, "_kittyIdToStorageId"))
    );
    assert_eq!(
        call(&get(&actual, "_kittyIdToStorageId"), "get", &[42.into()]),
        call(&get(&expected, "_kittyIdToStorageId"), "get", &[42.into()])
    );
    let old = call(&images, "get", &[43.into()]);
    let source_old = call(&source, "get", &[43.into()]);
    assert_eq!(old, call(&images, "get", &[43.into()]));
    for key in ["id", "width", "height", "format", "compression"] {
        assert_eq!(get(&old, key), get(&source_old, key));
    }
    let blob = get(&old, "data");
    assert_eq!(blob, get(&old, "data"));
    assert_eq!(get(&blob, "size"), JsValue::from(4));
    assert_eq!(bytes(&old).await, vec![5, 6, 7, 255]);
    assert_eq!(bytes(&old).await, bytes(&source_old).await);
    pair.write("\x1b_Ga=t,f=32,s=1,v=1,i=43;CAkK/w==\x1b\\")
        .await;
    let current = call(&images, "get", &[43.into()]);
    assert_ne!(old, current);
    assert_eq!(bytes(&current).await, vec![8, 9, 10, 255]);
    assert_eq!(
        bytes(&current).await,
        bytes(&call(&source, "get", &[43.into()])).await
    );
    assert_eq!(bytes(&old).await, bytes(&source_old).await);
    for key in ["42".into(), JsValue::NULL, f64::NAN.into(), 42.5.into()] {
        assert_eq!(
            call(&images, "has", &[key.clone()]),
            call(&source, "has", &[key.clone()])
        );
        assert!(call(&images, "get", &[key]).is_undefined());
    }
    pair.write("\x1b_Ga=d,d=I,i=42\x1b\\").await;
    assert_eq!(keys(&images), keys(&source));
    assert_eq!(
        keys(&get(&actual, "_kittyIdToStorageId")),
        JsValue::from("[]")
    );
    pair.compare();
}

#[wasm_bindgen_test(async)]
async fn pending_membership_tracks_chunks_delete_completion_and_reset() {
    let pair = Pair::new(true, "({sixelSupport:false})");
    let actual = get(&kitty_handler(&handlers(&pair)), "pendingTransmissions");
    let expected = get(
        &kitty_handler(&source_handlers(&pair)),
        "pendingTransmissions",
    );
    for (input, expected_keys) in [
        ("\x1b_Ga=t,f=32,s=1,v=1,i=55,m=1;AQID\x1b\\", "[55]"),
        ("\x1b_Ga=t,f=32,s=1,v=1,i=56,m=1;BAUG\x1b\\", "[55,56]"),
        ("\x1b_Ga=d,d=I,i=56\x1b\\", "[55]"),
        ("\x1b_Gi=55,m=0;/w==\x1b\\", "[]"),
        ("\x1b_Ga=t,f=32,s=1,v=1,i=57,m=1;AQID\x1b\\", "[57]"),
        ("\x1b[!p", "[]"),
    ] {
        pair.write(input).await;
        assert_eq!(keys(&actual), keys(&expected));
        assert_eq!(keys(&actual), JsValue::from(expected_keys));
        assert_eq!(get(&actual, "size"), get(&expected, "size"));
        for key in [
            JsValue::from(55),
            56.into(),
            57.into(),
            "55".into(),
            f64::NAN.into(),
        ] {
            assert_eq!(
                call(&actual, "has", &[key.clone()]),
                call(&expected, "has", &[key])
            );
        }
    }
}

#[wasm_bindgen_test(async)]
async fn retained_handler_maps_observe_activation_replacement_and_live_clear_iteration() {
    let pair = Pair::new(true, "({sixelSupport:false,showPlaceholder:false})");
    pair.write(kitty()).await;
    let actual = handlers(&pair);
    let expected = source_handlers(&pair);
    let old = kitty_handler(&actual);
    let source_old = kitty_handler(&expected);
    let iterator = call(&actual, "keys", &[]);
    let source_iterator = call(&expected, "keys", &[]);
    let next = |value: &JsValue| js_sys::JSON::stringify(&call(value, "next", &[])).unwrap();
    assert_eq!(next(&iterator), next(&source_iterator));
    let source_addon = get(&pair.source, "addon");
    pair.addon.activate(&pair.terminal).unwrap();
    call(&source_addon, "activate", &[get(&pair.source, "terminal")]);
    assert_eq!(actual, handlers(&pair));
    assert_ne!(old, kitty_handler(&actual));
    assert_eq!(get(&get(&old, "images"), "size"), JsValue::from(1));
    assert_eq!(
        get(&get(&kitty_handler(&actual), "images"), "size"),
        JsValue::from(0)
    );
    assert_eq!(next(&iterator), next(&source_iterator));
    pair.addon.dispose().unwrap();
    call(&source_addon, "dispose", &[]);
    assert_eq!(get(&actual, "size"), JsValue::from(0));
    assert_eq!(
        get(&get(&old, "images"), "size"),
        get(&get(&source_old, "images"), "size")
    );
    pair.addon.activate(&pair.terminal).unwrap();
    call(&source_addon, "activate", &[get(&pair.source, "terminal")]);
    assert_eq!(next(&iterator), next(&source_iterator));
    assert_eq!(next(&iterator), next(&source_iterator));
    assert_eq!(next(&iterator), next(&source_iterator));
    let exhausted = next(&iterator);
    assert_eq!(exhausted, next(&source_iterator));
    pair.addon.activate(&pair.terminal).unwrap();
    call(&source_addon, "activate", &[get(&pair.source, "terminal")]);
    assert_eq!(next(&iterator), exhausted);
    assert_eq!(keys(&actual), keys(&expected));
}

#[wasm_bindgen_test(async)]
async fn kitty_map_callbacks_can_reset_without_borrows_and_preserve_thrown_values() {
    let pair = Pair::new(true, "({sixelSupport:false})");
    pair.write(kitty()).await;
    let actual = kitty_handler(&handlers(&pair));
    let expected = kitty_handler(&source_handlers(&pair));
    let check = js_sys::Function::new_with_args(
        "handler",
        r#"
      const map=handler.images, token={};
      const callback=()=>{throw token};
      callback.call=()=>{throw Error('call accessed')};
      try {map.forEach(callback);throw Error('not thrown')} catch(error){if(error!==token)throw error}
      const receiver={seen:[]};
      map.forEach(function(value,key,owner){
        if(owner!==map || value!==map.get(key))throw Error('identity');
        this.seen.push(key);handler.reset();
      },receiver);
      return JSON.stringify([receiver.seen,map.size]);
    "#,
    );
    assert_eq!(
        check.call1(&JsValue::UNDEFINED, &actual).unwrap(),
        check.call1(&JsValue::UNDEFINED, &expected).unwrap()
    );
    assert_eq!(get(&get(&actual, "images"), "size"), JsValue::from(0));
}
