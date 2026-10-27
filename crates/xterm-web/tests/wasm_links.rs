#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Promise, Reflect};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{
    links::{
        browser::{default_activate, is_http_url},
        browser_debounce::{BrowserRenderDebouncer, BrowserTimeDebouncer},
        debounce::RenderRange,
    },
    utf16::from_js_string,
};
wasm_bindgen_test_configure!(run_in_browser);
fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
struct PropertyRestore {
    object: JsValue,
    key: JsValue,
    value: JsValue,
}
impl PropertyRestore {
    fn set(object: &JsValue, key: &str, value: &JsValue) -> Self {
        let key = JsValue::from_str(key);
        let old = Reflect::get(object, &key).unwrap();
        assert!(Reflect::set(object, &key, value).unwrap());
        Self {
            object: object.clone(),
            key,
            value: old,
        }
    }
}
impl Drop for PropertyRestore {
    fn drop(&mut self) {
        assert!(Reflect::set(&self.object, &self.key, &self.value).unwrap());
    }
}
#[wasm_bindgen_test]
fn whatwg_protocol_validation_handles_case_malformed_and_surrogates() {
    for uri in [
        "https://example.com/",
        "HTTP://EXAMPLE.COM",
        "https:example.com",
        "  https://example.com\n",
    ] {
        assert!(is_http_url(&utf16(uri)), "{uri}");
    }
    for uri in [
        "javascript:alert(1)",
        "mailto:x@example.com",
        "not a url",
        "http://[invalid",
    ] {
        assert!(!is_http_url(&utf16(uri)), "{uri}");
    }
    let mut uri = utf16("https://example.com/");
    uri.push(0xd800);
    assert!(is_http_url(&uri));
}
#[wasm_bindgen_test]
fn default_activation_cancel_and_opener_order() {
    let window = web_sys::window().unwrap();
    let global = js_sys::global();
    let setup=Function::new_no_args("globalThis.__linkPrompt = ''; globalThis.__linkOpens = 0; globalThis.__linkPopup = {opener: 'initial', location: {href: ''}};");
    setup.call0(&JsValue::NULL).unwrap();
    let confirm = Function::new_with_args("text", "globalThis.__linkPrompt=text; return false;");
    let open = Function::new_no_args("globalThis.__linkOpens++; return globalThis.__linkPopup;");
    let _confirm = PropertyRestore::set(window.as_ref(), "confirm", confirm.as_ref());
    let _open = PropertyRestore::set(window.as_ref(), "open", open.as_ref());
    let mut uri = utf16("https://example.com/");
    uri.push(0xd800);
    default_activate(&window, &uri).unwrap();
    assert_eq!(
        Reflect::get(&global, &JsValue::from_str("__linkOpens"))
            .unwrap()
            .as_f64(),
        Some(0.0)
    );
    let prompt: js_sys::JsString = Reflect::get(&global, &JsValue::from_str("__linkPrompt"))
        .unwrap()
        .unchecked_into();
    let mut expected = utf16("Do you want to navigate to ");
    expected.extend_from_slice(&uri);
    expected.extend(utf16(
        "?\n\nWARNING: This link could potentially be dangerous",
    ));
    assert_eq!(from_js_string(&prompt), expected);
    let yes = Function::new_no_args("return true;");
    let _yes = PropertyRestore::set(window.as_ref(), "confirm", yes.as_ref());
    default_activate(&window, &uri).unwrap();
    let popup = Reflect::get(&global, &JsValue::from_str("__linkPopup")).unwrap();
    assert!(Reflect::get(&popup, &JsValue::from_str("opener"))
        .unwrap()
        .is_null());
    let location = Reflect::get(&popup, &JsValue::from_str("location")).unwrap();
    let href: js_sys::JsString = Reflect::get(&location, &JsValue::from_str("href"))
        .unwrap()
        .unchecked_into();
    assert_eq!(from_js_string(&href), uri);
    for name in ["__linkPrompt", "__linkOpens", "__linkPopup"] {
        Reflect::delete_property(&global, &JsValue::from_str(name)).unwrap();
    }
}
#[wasm_bindgen_test(async)]
async fn real_animation_frame_coalesces_and_runs_callbacks_after_render() {
    let window = web_sys::window().unwrap();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    let d = BrowserRenderDebouncer::new(window, move |range| output.borrow_mut().push(range));
    d.refresh(Some(-1.0), Some(8.0), 10.0).unwrap();
    d.refresh(Some(4.0), Some(99.0), 6.0).unwrap();
    let completed = Promise::new(&mut |resolve, _| {
        let output = log.clone();
        d.add_refresh_callback(move |time| {
            assert_eq!(time, 0.0);
            assert_eq!(output.borrow().len(), 1);
            resolve.call0(&JsValue::NULL).unwrap();
        })
        .unwrap();
    });
    JsFuture::from(completed).await.unwrap();
    assert_eq!(
        *log.borrow(),
        vec![RenderRange {
            start: 0.0,
            end: 5.0
        }]
    );
    d.dispose().unwrap();
}
#[wasm_bindgen_test(async)]
async fn real_cancelled_frame_never_renders() {
    let window = web_sys::window().unwrap();
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    let d = BrowserRenderDebouncer::new(window.clone(), move |_| output.set(output.get() + 1));
    d.refresh(None, None, 5.0).unwrap();
    d.dispose().unwrap();
    let witness = BrowserRenderDebouncer::new(window, |_| {});
    let completed = Promise::new(&mut |resolve, _| {
        witness
            .add_refresh_callback(move |_| {
                resolve.call0(&JsValue::NULL).unwrap();
            })
            .unwrap();
    });
    JsFuture::from(completed).await.unwrap();
    assert_eq!(count.get(), 0);
}
#[wasm_bindgen_test(async)]
async fn real_trailing_timer_renders_without_polling_or_sleep() {
    let window = web_sys::window().unwrap();
    let holder = Rc::new(RefCell::new(None));
    let output = holder.clone();
    let threshold = window.performance().unwrap().now() + 1.0;
    let d = BrowserTimeDebouncer::new(window, threshold, move |range| {
        let resolve: Function = output.borrow_mut().take().unwrap();
        resolve
            .call1(&JsValue::NULL, &JsValue::from_f64(range.end))
            .unwrap();
    })
    .unwrap();
    let completed = Promise::new(&mut |resolve, _| {
        *holder.borrow_mut() = Some(resolve);
        d.refresh(None, None, 7.0).unwrap();
    });
    assert_eq!(JsFuture::from(completed).await.unwrap().as_f64(), Some(6.0));
    d.dispose().unwrap();
}
