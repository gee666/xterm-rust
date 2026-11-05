// Copyright (c) 2020 The xterm.js authors. MIT license.
use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

#[wasm_bindgen_test(async)]
async fn retained_inspection_stays_with_its_original_activation() {
    let pair = Pair::new(true, "({showPlaceholder:false,sixelSupport:false})");
    pair.write(iip()).await;
    let old = get(&pair.addon.storage_inspection(), "_images");
    let source_addon = get(&pair.source, "addon");
    let source_old = get(&get(&source_addon, "_storage"), "_images");
    pair.addon.activate(&pair.terminal).unwrap();
    call(&source_addon, "activate", &[get(&pair.source, "terminal")]);
    let current = get(&pair.addon.storage_inspection(), "_images");
    let source_current = get(&get(&source_addon, "_storage"), "_images");
    assert_ne!(old, current);
    assert_eq!(get(&old, "size"), JsValue::from(1));
    assert_eq!(get(&current, "size"), JsValue::from(0));
    pair.write(iip()).await;
    for (actual, expected) in [(&old, &source_old), (&current, &source_current)] {
        assert_eq!(get(actual, "size"), get(expected, "size"));
        assert_eq!(get(actual, "size"), JsValue::from(1));
    }
    pair.addon.dispose().unwrap();
    call(&source_addon, "dispose", &[]);
    for (actual, expected) in [(&old, &source_old), (&current, &source_current)] {
        assert_eq!(get(actual, "size"), JsValue::from(0));
        assert_eq!(get(actual, "size"), get(expected, "size"));
    }
}
