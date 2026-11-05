// Copyright (c) 2019-2023 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::{
    unicode::UnicodeCharWidth as Width, unicode_service::UnicodeService, unicode_v6::UnicodeV6,
};
use xterm_wasm_support::{js_error::to_js_error, public_api::unicode_api::WebUnicodeApi};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(inline_js = "
export function signedProvider(reenter) {
 const p={version:'signed',events:[],charProperties:()=>2};
 Object.defineProperty(p,'wcwidth',{get(){p.events.push('get');return function(code){p.events.push([code,this===p]);if(code<0)reenter();return code<0?2:1;};}});
 return p;
}
export function throwingProvider(token) {return {version:'throws',get wcwidth(){throw token;},charProperties:()=>2};}
export function glyphProviderEvents(p) {return JSON.stringify(p.events);}
")]
extern "C" {
    #[wasm_bindgen(js_name=signedProvider)]
    fn signed_provider(reenter: &js_sys::Function) -> JsValue;
    #[wasm_bindgen(js_name=throwingProvider)]
    fn throwing_provider(token: &JsValue) -> JsValue;
    #[wasm_bindgen(js_name=glyphProviderEvents)]
    fn events(provider: &JsValue) -> String;
}
#[wasm_bindgen_test]
fn forwards_signed_number_and_receiver_before_reentrant_version_change() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let api = WebUnicodeApi::new(service.clone());
    let weak = Rc::downgrade(&service);
    let reenter = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        weak.upgrade()
            .unwrap()
            .set_active_version("6")
            .map_err(|e| to_js_error(&e))
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    let provider = signed_provider(reenter.as_ref().unchecked_ref());
    api.register(provider.clone()).unwrap();
    api.set_active_version(js_sys::JsString::from("signed"))
        .unwrap();
    assert_eq!(service.wcwidth(65).unwrap(), Width::Narrow);
    assert_eq!(service.wcwidth_i32(-2147483583).unwrap(), Width::Wide);
    assert_eq!(service.active_version(), "6");
    assert_eq!(service.wcwidth_i32(-1).unwrap(), Width::Zero);
    assert_eq!(
        events(&provider),
        "[\"get\",[65,true],\"get\",[-2147483583,true]]"
    );
}
#[wasm_bindgen_test]
fn signed_provider_getter_throw_keeps_original_js_value() {
    let service = Rc::new(UnicodeService::new());
    service.register(Rc::new(UnicodeV6::new())).unwrap();
    let api = WebUnicodeApi::new(service.clone());
    let token = js_sys::Object::new();
    api.register(throwing_provider(token.as_ref())).unwrap();
    api.set_active_version(js_sys::JsString::from("throws"))
        .unwrap();
    let error = service.wcwidth_i32(-1).unwrap_err();
    assert_eq!(to_js_error(&error), JsValue::from(token));
}
