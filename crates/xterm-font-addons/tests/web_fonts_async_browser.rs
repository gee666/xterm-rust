#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::FontFace;
use xterm_font_addons::{font_set::load_fonts, WebFontsAddon};

wasm_bindgen_test_configure!(run_in_browser);

fn get(value: &JsValue, key: &str) -> JsValue {
    Reflect::get(value, &key.into()).unwrap()
}

fn call(value: &JsValue, key: &str) {
    get(value, key)
        .unchecked_into::<Function>()
        .call0(value)
        .unwrap();
}

fn face(name: &str) -> FontFace {
    FontFace::new_with_u8_array(name, include_bytes!("fixtures/kongtext.regular.ttf")).unwrap()
}

fn host(family: &str) -> JsValue {
    Function::new_with_args("family", "const writes=[]; let current=family; return {writes, options: {get fontFamily(){return current},set fontFamily(v){writes.push(v);current=v}}};")
        .call1(&JsValue::UNDEFINED, &family.into()).unwrap()
}

fn controlled_face() -> (FontFace, JsValue) {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    let face = face("AsyncWeb");
    document.fonts().add(&face).unwrap();
    let control = Function::new_with_args("face", "let begin, finish; const started=new Promise(r=>begin=r); const pending=new Promise(r=>finish=r); face.load=function(){begin();return pending};return {started,finish(){finish(face)}};")
        .call1(&JsValue::UNDEFINED, face.as_ref()).unwrap();
    (face, control)
}

#[wasm_bindgen_test(async)]
async fn disposal_during_load_cancels_both_option_assignments() {
    let (_face, control) = controlled_face();
    let terminal = host("AsyncWeb, Local");
    let addon = WebFontsAddon::new(Some(false));
    addon.activate(terminal.clone()).unwrap();
    let operation = addon.relayout();
    JsFuture::from(get(&control, "started").unchecked_into::<Promise>())
        .await
        .unwrap();
    addon.dispose();
    call(&control, "finish");
    JsFuture::from(operation).await.unwrap();
    assert_eq!(Array::from(&get(&terminal, "writes")).length(), 0);
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .fonts()
        .clear();
}

#[wasm_bindgen_test(async)]
async fn reactivation_during_load_uses_current_terminal_and_old_family_snapshot() {
    let (_face, control) = controlled_face();
    let first = host("AsyncWeb, Local");
    let second = host("Other");
    let addon = WebFontsAddon::new(Some(false));
    addon.activate(first.clone()).unwrap();
    let operation = addon.relayout();
    JsFuture::from(get(&control, "started").unchecked_into::<Promise>())
        .await
        .unwrap();
    addon.activate(second.clone()).unwrap();
    call(&control, "finish");
    JsFuture::from(operation).await.unwrap();
    assert_eq!(Array::from(&get(&first, "writes")).length(), 0);
    let writes = Array::from(&get(&second, "writes"));
    assert_eq!(writes.get(0), "Local");
    assert_eq!(writes.get(1), "AsyncWeb, Local");
    addon.dispose();
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .fonts()
        .clear();
}

#[wasm_bindgen_test(async)]
async fn duplicate_family_requests_call_load_repeatedly_in_request_order() {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    let first = face("OrderWeb");
    let second = face("OrderWeb");
    second.set_weight("700");
    document.fonts().add(&first).unwrap();
    document.fonts().add(&second).unwrap();
    let calls = Function::new_with_args("a,b", "const calls=[]; a.load=function(){calls.push('a');return Promise.resolve(a)};b.load=function(){calls.push('b');return Promise.resolve(b)};return calls;")
        .call2(&JsValue::UNDEFINED, first.as_ref(), second.as_ref()).unwrap();
    let request = Array::of2(&"OrderWeb".into(), &"OrderWeb".into());
    let loaded = load_fonts(&document, Some(&request)).await.unwrap();
    assert_eq!(Array::from(&calls).join(","), "a,b,a,b");
    assert_eq!(loaded.length(), 4);
    assert!(Object::is(&loaded.get(0), first.as_ref()));
    assert!(Object::is(&loaded.get(1), second.as_ref()));
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn duplicate_descriptors_reuse_first_even_if_source_and_display_differ() {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    let first = face("HashWeb");
    let duplicate = FontFace::new_with_u8_array("HashWeb", &[0, 1, 2, 3]).unwrap();
    duplicate.set_display("swap");
    duplicate.set_feature_settings("\"liga\" 0");
    document.fonts().add(&first).unwrap();
    let loaded = load_fonts(&document, Some(&Array::of1(duplicate.as_ref())))
        .await
        .unwrap();
    assert_eq!(document.fonts().size(), 1);
    assert!(Object::is(&loaded.get(0), first.as_ref()));
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn disposing_from_first_option_setter_preserves_second_write_typeerror() {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    document.fonts().add(&face("ReentrantWeb")).unwrap();
    let addon = std::rc::Rc::new(WebFontsAddon::new(Some(false)));
    let weak = std::rc::Rc::downgrade(&addon);
    let dispose = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
        weak.upgrade().unwrap().dispose();
    }) as Box<dyn FnMut()>);
    let terminal = Function::new_with_args("dispose", "const writes=[];return {writes,options:{get fontFamily(){return 'ReentrantWeb'},set fontFamily(value){writes.push(value);dispose()}}};")
        .call1(&JsValue::UNDEFINED, dispose.as_ref()).unwrap();
    addon.activate(terminal.clone()).unwrap();
    let error = JsFuture::from(addon.relayout()).await.unwrap_err();
    assert_eq!(get(&error, "name"), "TypeError");
    let writes = Array::from(&get(&terminal, "writes"));
    assert_eq!(writes.length(), 1);
    assert_eq!(writes.get(0), "monospace");
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn family_lookup_is_case_sensitive_and_unquotes_registered_names() {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    let registered = face("CaseWeb");
    // Firefox CSS faces preserve a quote pair. WebKit treats quotes passed to
    // the FontFace constructor as literal name content, so model the returned
    // CSS attribute explicitly rather than constructing a different family.
    Function::new_with_args(
        "face",
        "Object.defineProperty(face,'family',{configurable:true,get(){return \"'CaseWeb'\"}});",
    )
    .call1(&JsValue::UNDEFINED, registered.as_ref())
    .unwrap();
    document.fonts().add(&registered).unwrap();
    assert_eq!(
        load_fonts(&document, Some(&Array::of1(&"CaseWeb".into())))
            .await
            .unwrap()
            .length(),
        1
    );
    let error = load_fonts(&document, Some(&Array::of1(&"caseweb".into())))
        .await
        .unwrap_err();
    assert_eq!(
        error,
        "font family \"caseweb\" not registered in document.fonts"
    );
    document.fonts().clear();
}
