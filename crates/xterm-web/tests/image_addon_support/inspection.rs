// Copyright (c) 2020 The xterm.js authors. MIT license.
use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

fn images(pair: &Pair) -> JsValue {
    get(&pair.addon.storage_inspection(), "_images")
}
fn source_images(pair: &Pair) -> JsValue {
    get(&get(&get(&pair.source, "addon"), "_storage"), "_images")
}
fn keys(map: &JsValue) -> String {
    let iterator = call(map, "keys", &[]);
    let array = js_sys::Array::from(&iterator);
    js_sys::JSON::stringify(&array)
        .unwrap()
        .as_string()
        .unwrap()
}

#[wasm_bindgen_test(async)]
async fn storage_inspection_reads_real_surfaces_identity_and_layers() {
    let pair = Pair::new(true, "({showPlaceholder:false})");
    assert_eq!(
        pair.addon.storage_inspection(),
        pair.addon.storage_inspection()
    );
    let map = images(&pair);
    assert_eq!(map, images(&pair));
    assert_eq!(get(&map, "size"), JsValue::from(0));
    pair.write(iip()).await;
    pair.write("\x1b[3;1H\x1b_Ga=T,f=32,s=1,v=1,z=-1;AQID/w==\x1b\\")
        .await;
    let source = source_images(&pair);
    assert_eq!(get(&map, "size"), get(&source, "size"));
    assert_eq!(keys(&map), keys(&source));
    for id in [1, 2] {
        let actual = call(&map, "get", &[id.into()]);
        assert_eq!(actual, call(&map, "get", &[id.into()]));
        let expected = call(&source, "get", &[id.into()]);
        for key in ["layer", "zIndex", "tileCount", "bufferType"] {
            assert_eq!(get(&actual, key), get(&expected, key));
        }
        let a = get(&actual, "orig");
        let b = get(&expected, "orig");
        assert_eq!(get(&a, "width"), get(&b, "width"));
        assert_eq!(get(&a, "height"), get(&b, "height"));
    }
    let image = call(&map, "get", &[1.into()]);
    assert_eq!(
        pixels(&get(&image, "orig")),
        pixels(&get(&call(&source, "get", &[1.into()]), "orig"))
    );
    for key in [
        "1".into(),
        1.5.into(),
        f64::NAN.into(),
        (-1).into(),
        JsValue::NULL,
    ] {
        assert_eq!(
            call(&map, "has", &[key.clone()]),
            call(&source, "has", &[key.clone()])
        );
        assert!(call(&map, "get", &[key]).is_undefined());
    }
}

#[wasm_bindgen_test(async)]
async fn retained_storage_iteration_observes_reset_appends_and_exhaustion() {
    let pair = Pair::new(true, "({showPlaceholder:false,sixelSupport:false})");
    pair.write(iip()).await;
    let map = images(&pair);
    let source = source_images(&pair);
    let actual = call(&map, "keys", &[]);
    let expected = call(&source, "keys", &[]);
    let next = |iterator: &JsValue| js_sys::JSON::stringify(&call(iterator, "next", &[])).unwrap();
    assert_eq!(next(&actual), next(&expected));
    pair.addon.reset().unwrap();
    call(&get(&pair.source, "addon"), "reset", &[]);
    pair.write(iip()).await;
    assert_eq!(next(&actual), next(&expected));
    assert_eq!(next(&actual), next(&expected));
    pair.write(iip()).await;
    assert_eq!(next(&actual), next(&expected));
    assert_eq!(keys(&map), keys(&source));
    pair.addon.dispose().unwrap();
    call(&get(&pair.source, "addon"), "dispose", &[]);
    assert_eq!(get(&map, "size"), JsValue::from(0));
    assert_eq!(get(&map, "size"), get(&source, "size"));
}

#[wasm_bindgen_test(async)]
async fn inspection_for_each_releases_borrows_before_reset_callback() {
    let pair = Pair::new(true, "({showPlaceholder:false,sixelSupport:false})");
    pair.write(iip()).await;
    pair.write("\x1b[3;1H").await;
    pair.write(iip()).await;
    let run = js_sys::Function::new_with_args(
        "map,reset",
        r#"
      const receiver={seen:[]};
      map.forEach(function(value,key,owner){
        if(owner!==map || value!==map.get(key))throw Error('inspection identity');
        this.seen.push(key); reset();
      },receiver);
      return JSON.stringify(receiver.seen);
    "#,
    );
    let native = pair.terminal.shared_native();
    let reset = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
        native.write("\x1b[!p").unwrap();
    });
    let source_addon = get(&pair.source, "addon");
    use wasm_bindgen::JsCast;
    let source_reset = get(&source_addon, "reset")
        .unchecked_into::<js_sys::Function>()
        .bind0(&source_addon);
    let actual = run
        .call2(&JsValue::UNDEFINED, &images(&pair), reset.as_ref())
        .unwrap();
    let expected = run
        .call2(&JsValue::UNDEFINED, &source_images(&pair), &source_reset)
        .unwrap();
    assert_eq!(actual, expected);
    assert_eq!(actual, JsValue::from("[1]"));
    assert_eq!(get(&images(&pair), "size"), JsValue::from(0));
}
