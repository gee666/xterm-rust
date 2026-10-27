#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, JsString, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, FontFace};
use xterm_web::{font_set::load_fonts, WebFontsAddon};

wasm_bindgen_test_configure!(run_in_browser);

fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}

fn face(name: &str) -> FontFace {
    FontFace::new_with_u8_array(
        name,
        include_bytes!("../../../source_repo/demo/fonts/kongtext.regular.ttf"),
    )
    .unwrap()
}

fn requests(values: &[JsValue]) -> Array {
    values.iter().collect()
}

fn host(family: &str) -> JsValue {
    // This records the public option setter, not an xterm.js implementation.
    Function::new_with_args("family", "const writes = []; let current = family; return {writes, options: {get fontFamily(){return current}, set fontFamily(value){writes.push(value); current=value}}};")
        .call1(&JsValue::UNDEFINED, &JsValue::from_str(family)).unwrap()
}

fn writes(host: &JsValue) -> Array {
    Reflect::get(host, &JsValue::from_str("writes"))
        .unwrap()
        .unchecked_into()
}

#[wasm_bindgen_test(async)]
async fn loads_fontface_buffers_and_deduplicates_all_five_descriptors() {
    let document = document();
    document.fonts().clear();
    let first = face("FontCase");
    let duplicate = face("FontCase");
    let selected = load_fonts(
        &document,
        Some(&requests(&[
            first.clone().into(),
            duplicate.into(),
            "FontCase".into(),
        ])),
    )
    .await
    .unwrap();
    assert_eq!(document.fonts().size(), 1);
    assert_eq!(selected.length(), 3);
    for result in selected {
        assert!(Object::is(&result, first.as_ref()));
        assert_eq!(Reflect::get(&result, &"status".into()).unwrap(), "loaded");
    }
    let weighted = face("FontCase");
    weighted.set_weight("700");
    load_fonts(&document, Some(&requests(&[weighted.into()])))
        .await
        .unwrap();
    assert_eq!(document.fonts().size(), 2);
    assert_eq!(load_fonts(&document, None).await.unwrap().length(), 2);
    assert_eq!(
        load_fonts(&document, Some(&Array::new()))
            .await
            .unwrap()
            .length(),
        2
    );
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn unknown_family_preserves_prior_addition_and_exact_error() {
    let document = document();
    document.fonts().clear();
    let first = face("BeforeError");
    let error = load_fonts(
        &document,
        Some(&requests(&[first.clone().into(), "missing".into()])),
    )
    .await
    .unwrap_err();
    assert_eq!(
        error,
        "font family \"missing\" not registered in document.fonts"
    );
    assert!(document.fonts().has(&first));
    assert_eq!(document.fonts().size(), 1);
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn failed_font_load_rejects_without_relayout_assignments() {
    let document = document();
    document.fonts().clear();
    let broken = FontFace::new_with_u8_array("BrokenFont", &[0, 1, 2, 3]).unwrap();
    document.fonts().add(&broken).unwrap();
    let terminal = host("BrokenFont, monospace");
    let addon = WebFontsAddon::new(Some(false));
    addon.activate(terminal.clone()).unwrap();
    assert!(JsFuture::from(addon.relayout()).await.is_err());
    assert_eq!(writes(&terminal).length(), 0);
    addon.dispose();
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn relayout_partitions_families_and_restores_exact_original() {
    let document = document();
    document.fonts().clear();
    let web = face("WebFont");
    document.fonts().add(&web).unwrap();
    let terminal = host(" 'WebFont', Local, \"A B\"");
    let addon = WebFontsAddon::new(Some(false));
    addon.activate(terminal.clone()).unwrap();
    JsFuture::from(addon.relayout()).await.unwrap();
    let changes = writes(&terminal);
    assert_eq!(changes.length(), 2);
    assert_eq!(changes.get(0), "Local, \"A B\"");
    assert_eq!(changes.get(1), " 'WebFont', Local, \"A B\"");
    addon.dispose();
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn no_registered_family_and_disposed_addon_have_no_writes() {
    let document = document();
    document.fonts().clear();
    let terminal = host("Local, monospace");
    let addon = WebFontsAddon::new(Some(false));
    assert!(!addon.initial_relayout());
    addon.activate(terminal.clone()).unwrap();
    JsFuture::from(addon.relayout()).await.unwrap();
    assert_eq!(writes(&terminal).length(), 0);
    addon.dispose();
    addon.dispose();
    JsFuture::from(addon.relayout()).await.unwrap();
    assert_eq!(writes(&terminal).length(), 0);
}

#[wasm_bindgen_test(async)]
async fn empty_clean_families_use_monospace_then_restore() {
    let document = document();
    document.fonts().clear();
    document.fonts().add(&face("OnlyWeb")).unwrap();
    let terminal = host("OnlyWeb");
    let addon = WebFontsAddon::new(Some(false));
    addon.activate(terminal.clone()).unwrap();
    JsFuture::from(addon.relayout()).await.unwrap();
    assert_eq!(writes(&terminal).get(0), "monospace");
    assert_eq!(writes(&terminal).get(1), "OnlyWeb");
    addon.dispose();
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn default_initial_relayout_runs_after_ready() {
    let document = document();
    document.fonts().clear();
    document.fonts().add(&face("AutoWeb")).unwrap();
    let terminal = host("AutoWeb");
    let addon = WebFontsAddon::new(None);
    assert!(addon.initial_relayout());
    addon.activate(terminal.clone()).unwrap();
    // An explicit relayout gives an event-driven completion boundary. Both
    // operations await the same ready promise, without timing sleeps.
    JsFuture::from(addon.relayout()).await.unwrap();
    assert!(writes(&terminal).length() >= 2);
    addon.dispose();
    document.fonts().clear();
}

#[wasm_bindgen_test]
fn boundary_helpers_preserve_lone_surrogate_font_names() {
    let input = JsString::from_char_code(&[0x22, 0xD800, 0x22]);
    let output = xterm_web::utf16::to_js_string(&xterm_web::utf16::from_js_string(&input));
    assert!(Object::is(input.as_ref(), output.as_ref()));
}
