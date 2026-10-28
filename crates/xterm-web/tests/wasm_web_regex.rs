#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::{Function, JsString, Object, Reflect, RegExp};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::{link_snapshot::LinkSnapshot, web_link_provider::*, web_links::*};
use xterm_core::{
    lifecycle::{DisposableHandle, NoopDisposable},
    terminal::CoreTerminal,
};
use xterm_web::{utf16::to_js_string, utility_addons::matcher::ObjectLinkMatcher};
wasm_bindgen_test_configure!(run_in_browser);
#[path = "../../xterm-addons/tests/support/utility_sparse.rs"]
mod sparse;
#[wasm_bindgen_test]
fn sparse_rows_above_signed_wasm32_and_virtual_end_beyond_u32_are_exact() {
    sparse::assert_source_cases(&xterm_web::utility_addons::matcher::BrowserLinkMatcher);
}
type Log = Rc<RefCell<Vec<&'static str>>>;
struct Host {
    engine: CoreTerminal,
    log: Log,
    provider: RefCell<Option<Provider<()>>>,
}
impl WebLinkTerminal<()> for Host {
    fn buffer(&self) -> Result<Box<dyn LinkBuffer>, LinkError> {
        self.log.borrow_mut().push("buffer");
        Ok(Box::new(LinkSnapshot::from_engine(&self.engine)?))
    }
    fn register_link_provider(
        &self,
        provider: Provider<()>,
    ) -> Result<DisposableHandle, LinkError> {
        *self.provider.borrow_mut() = Some(provider);
        Ok(Rc::new(NoopDisposable))
    }
    fn open_link(&self, _: &(), _: &[u16]) -> Result<(), LinkError> {
        Err(LinkError::Message("test does not open links".into()))
    }
}
#[wasm_bindgen_test]
fn flags_twice_then_source_precede_actual_buffer_snapshot() {
    let log = Log::default();
    let flags_log = log.clone();
    let source_log = log.clone();
    let flags = Closure::wrap(Box::new(move || -> JsValue {
        flags_log.borrow_mut().push("flags");
        JsString::from("").into()
    }) as Box<dyn FnMut() -> JsValue>);
    let source = Closure::wrap(Box::new(move || -> JsValue {
        source_log.borrow_mut().push("source");
        JsString::from("http://example.com").into()
    }) as Box<dyn FnMut() -> JsValue>);
    let pattern = Object::new();
    for (key, getter) in [("flags", flags.as_ref()), ("source", source.as_ref())] {
        let descriptor = Object::new();
        Reflect::set(&descriptor, &"get".into(), getter).unwrap();
        Object::define_property(&pattern, &key.into(), &descriptor);
    }
    let mut engine = CoreTerminal::new(80, 24, 1000).unwrap();
    engine.write("http://example.com").unwrap();
    let host = Rc::new(Host {
        engine,
        log: log.clone(),
        provider: RefCell::default(),
    });
    let addon = WebLinksAddon::new(
        Rc::new(ObjectLinkMatcher::new(pattern.into())),
        None,
        LinkOptions::default(),
    );
    addon.activate(host.clone()).unwrap();
    assert!(
        log.borrow().is_empty(),
        "RegExp properties are not eagerly read at activation"
    );
    let query = host.provider.borrow().clone().unwrap();
    assert_eq!(query(1).unwrap().len(), 1);
    assert_eq!(*log.borrow(), ["flags", "flags", "source", "buffer"]);
}
#[wasm_bindgen_test]
fn regexp_compile_changes_are_visible_on_later_requests() {
    let regex = RegExp::new("http://one", "");
    let matcher = ObjectLinkMatcher::new(regex.clone().into());
    let input = "http://one http://two".encode_utf16().collect::<Vec<_>>();
    assert_eq!(
        String::from_utf16(&matcher.matches(&input, &RegexSpec::default()).unwrap()[0].text)
            .unwrap(),
        "http://one"
    );
    let compile = Reflect::get(regex.as_ref(), &"compile".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();
    compile
        .call2(regex.as_ref(), &"http://two".into(), &"".into())
        .unwrap();
    assert_eq!(
        String::from_utf16(&matcher.matches(&input, &RegexSpec::default()).unwrap()[0].text)
            .unwrap(),
        "http://two"
    );
}
#[wasm_bindgen_test]
fn custom_regexp_source_retains_lone_utf16_surrogates() {
    let mut text = "http://example.com/".encode_utf16().collect::<Vec<_>>();
    text.push(0xd800);
    let pattern = Object::new();
    Reflect::set(&pattern, &"source".into(), &to_js_string(&text)).unwrap();
    Reflect::set(&pattern, &"flags".into(), &JsString::from("")).unwrap();
    let found = ObjectLinkMatcher::new(pattern.into())
        .matches(&text, &RegexSpec::default())
        .unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].text, text);
}
