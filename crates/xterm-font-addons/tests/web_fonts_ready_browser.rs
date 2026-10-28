#![cfg(target_arch = "wasm32")]

use js_sys::{Array, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::FontFace;
use xterm_font_addons::WebFontsAddon;

wasm_bindgen_test_configure!(run_in_browser);

fn property(value: &JsValue, name: &str) -> JsValue {
    Reflect::get(value, &name.into()).unwrap()
}

fn call(value: &JsValue, name: &str) {
    property(value, name)
        .unchecked_into::<Function>()
        .call0(value)
        .unwrap();
}

struct ReadyOverride(JsValue);
impl ReadyOverride {
    fn new() -> Self {
        let control = Function::new_no_args("const set=document.fonts, previous=Object.getOwnPropertyDescriptor(set,'ready');let begin,finish,fail;const started=new Promise(r=>begin=r),pending=new Promise((r,j)=>{finish=r;fail=j});const error=new Error('controlled ready failure');Object.defineProperty(set,'ready',{configurable:true,get(){begin();return pending}});return {started,error,finish(){finish(set)},fail(){fail(error)},restore(){if(previous)Object.defineProperty(set,'ready',previous);else delete set.ready}};")
            .call0(&JsValue::UNDEFINED).unwrap();
        Self(control)
    }
    async fn started(&self) {
        JsFuture::from(property(&self.0, "started").unchecked_into::<Promise>())
            .await
            .unwrap();
    }
}
impl Drop for ReadyOverride {
    fn drop(&mut self) {
        call(&self.0, "restore");
    }
}

fn terminal() -> JsValue {
    Function::new_no_args("const writes=[];return {writes,options:{get fontFamily(){return 'ReadyFont'},set fontFamily(value){writes.push(value)}}};")
        .call0(&JsValue::UNDEFINED).unwrap()
}

fn face() -> FontFace {
    FontFace::new_with_u8_array("ReadyFont", include_bytes!("fixtures/kongtext.regular.ttf"))
        .unwrap()
}

#[wasm_bindgen_test(async)]
async fn loader_enumerates_only_after_ready_resolves() {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    let ready = ReadyOverride::new();
    let addon = WebFontsAddon::new(Some(false));
    let result = addon.load_fonts(Some(Array::of1(&"ReadyFont".into())));
    ready.started().await;
    let face = face();
    document.fonts().add(&face).unwrap();
    call(&ready.0, "finish");
    let loaded = Array::from(&JsFuture::from(result).await.unwrap());
    assert_eq!(loaded.length(), 1);
    assert!(Object::is(&loaded.get(0), face.as_ref()));
    document.fonts().clear();
}

#[wasm_bindgen_test(async)]
async fn ready_rejection_retains_error_identity_before_font_loading() {
    let document = web_sys::window().unwrap().document().unwrap();
    document.fonts().clear();
    let ready = ReadyOverride::new();
    let addon = WebFontsAddon::new(Some(false));
    let result = addon.load_fonts(None);
    ready.started().await;
    call(&ready.0, "fail");
    let error = JsFuture::from(result).await.unwrap_err();
    assert!(Object::is(&error, &property(&ready.0, "error")));
    assert_eq!(document.fonts().size(), 0);
}

#[wasm_bindgen_test(async)]
async fn dispose_during_explicit_relayout_ready_preserves_source_typeerror() {
    let ready = ReadyOverride::new();
    let addon = WebFontsAddon::new(Some(false));
    let terminal = terminal();
    addon.activate(terminal.clone()).unwrap();
    let result = addon.relayout();
    ready.started().await;
    addon.dispose();
    call(&ready.0, "finish");
    let error = JsFuture::from(result).await.unwrap_err();
    assert_eq!(property(&error, "name"), "TypeError");
    assert_eq!(Array::from(&property(&terminal, "writes")).length(), 0);
}

#[wasm_bindgen_test(async)]
async fn dispose_before_initial_relayout_ready_prevents_later_writes() {
    let ready = ReadyOverride::new();
    let addon = WebFontsAddon::new(None);
    let terminal = terminal();
    addon.activate(terminal.clone()).unwrap();
    ready.started().await;
    addon.dispose();
    call(&ready.0, "finish");
    // Drain promise reactions, not a timed retry or sleep. Initial relayout's
    // callback checks for a terminal before its second ready await.
    JsFuture::from(Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
    JsFuture::from(Promise::resolve(&JsValue::UNDEFINED))
        .await
        .unwrap();
    assert_eq!(Array::from(&property(&terminal, "writes")).length(), 0);
}
