// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use crate::support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

fn compare_options(actual: &JsValue, source: &JsValue) {
    assert_eq!(
        js_sys::JSON::stringify(&js_sys::Object::keys(&actual.clone().into())).unwrap(),
        js_sys::JSON::stringify(&js_sys::Object::keys(&get(source, "_opts").into())).unwrap()
    );
    assert_eq!(
        js_sys::JSON::stringify(actual).unwrap(),
        js_sys::JSON::stringify(&get(source, "_opts")).unwrap()
    );
}

#[wasm_bindgen_test]
fn typed_constructor_option_inspection_matches_source() {
    for options in [
        "undefined",
        "null",
        "({})",
        "({enableSizeReports:false,pixelLimit:5,sixelSupport:false,sixelScrolling:false,sixelPaletteLimit:1024,sixelSizeLimit:1000,storageLimit:10,showPlaceholder:false,iipSupport:false,iipSizeLimit:1000,kittySupport:false,kittySizeLimit:1000})",
        "Object.create({sixelPaletteLimit:512})",
        "(()=>{let count=0;return {get sixelPaletteLimit(){return ++count*128}}})()",
    ] {
        let rust = ImageAddon::new(eval(options)).unwrap();
        let source = js_sys::Function::new_with_args("options", "return new SourceImageAddon(options)")
            .call1(&JsValue::UNDEFINED, &eval(options))
            .unwrap();
        compare_options(&rust.option_inspection(), &source);
        assert_eq!(rust.option_inspection(), rust.option_inspection());
        rust.reset().unwrap();
        call(&source, "reset", &[]);
        compare_options(&rust.option_inspection(), &source);
        rust.dispose().unwrap();
        call(&source, "dispose", &[]);
    }
}

#[wasm_bindgen_test(async)]
async fn retained_option_view_tracks_controls_setters_and_reset() {
    let pair = Pair::new(true, "({showPlaceholder:false,sixelPaletteLimit:256})");
    pair.ready().await;
    let actual = pair.addon.option_inspection();
    let source = get(&pair.source, "addon");
    compare_options(&actual, &source);
    for text in [
        "\x1b[?80h",
        "\x1b[?80l",
        "\x1b[?1;3;64S",
        "\x1b[?1;2S",
        "\x1b[?1;3;32S",
        "\x1b[!p",
        "\x1bc",
    ] {
        pair.write(text).await;
        assert_eq!(actual, pair.addon.option_inspection());
        compare_options(&actual, &source);
        pair.compare();
    }
    for value in [true, false] {
        pair.addon.set_show_placeholder(value).unwrap();
        js_sys::Reflect::set(&source, &"showPlaceholder".into(), &value.into()).unwrap();
        compare_options(&actual, &source);
    }
    pair.addon.set_storage_limit(64.0).unwrap();
    js_sys::Reflect::set(&source, &"storageLimit".into(), &64.into()).unwrap();
    compare_options(&actual, &source);
    assert!(pair.addon.set_storage_limit(-1.0).is_err());
    let failed = js_sys::Function::new_with_args(
        "addon",
        "try { addon.storageLimit=-1; return false; } catch(e) { return true; }",
    )
    .call1(&JsValue::UNDEFINED, &source)
    .unwrap();
    assert_eq!(failed, JsValue::TRUE);
    compare_options(&actual, &source);
    pair.addon.reset().unwrap();
    call(&source, "reset", &[]);
    compare_options(&actual, &source);
    assert_eq!(get(&actual, "storageLimit"), JsValue::from(64));
    pair.addon.dispose().unwrap();
    call(&source, "dispose", &[]);
    compare_options(&actual, &source);
}

#[wasm_bindgen_test]
fn option_diagnostic_rejects_writes_and_survives_owner_release() {
    let rust = ImageAddon::new(JsValue::UNDEFINED).unwrap();
    let view = rust.option_inspection();
    assert!(js_sys::Object::is_frozen(&view.clone().into()));
    assert!(!js_sys::Reflect::set(&view, &"pixelLimit".into(), &1.into()).unwrap());
    assert!(!js_sys::Reflect::set(&view, &"extra".into(), &1.into()).unwrap());
    assert!(
        !js_sys::Reflect::delete_property(&view.clone().into(), &"sixelScrolling".into()).unwrap()
    );
    assert_eq!(get(&view, "pixelLimit"), JsValue::from(16777216));
    rust.set_show_placeholder(false).unwrap();
    rust.dispose().unwrap();
    drop(rust);
    assert_eq!(get(&view, "showPlaceholder"), JsValue::FALSE);
    assert_eq!(get(&view, "sixelPaletteLimit"), JsValue::from(4096));
}
