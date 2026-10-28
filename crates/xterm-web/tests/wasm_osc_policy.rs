#![cfg(target_arch = "wasm32")]
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::links::{
    browser_linkifier::{BrowserLinkProvider, LinkReply},
    native_osc::{NativeOscLinkProvider, OscLinkHandler},
};
wasm_bindgen_test_configure!(run_in_browser);
fn reply(count: Rc<Cell<usize>>, last: Rc<RefCell<Option<usize>>>) -> LinkReply {
    Box::new(move |links| {
        count.set(count.get() + 1);
        *last.borrow_mut() = links.map(|links| links.len());
        Ok(())
    })
}
#[wasm_bindgen_test]
fn missing_empty_rows_and_per_run_protocol_getter_failure_order() {
    let native = Rc::new(xterm_core::headless::Terminal::new(5, 3, 0).unwrap());
    let settings_calls = Rc::new(Cell::new(0));
    let settings = settings_calls.clone();
    let policy_calls = Rc::new(Cell::new(0));
    let policy = policy_calls.clone();
    let marker: JsValue = js_sys::Object::new().into();
    let error = marker.clone();
    let handler = OscLinkHandler {
        allow_non_http_protocols: Rc::new(move || {
            policy.set(policy.get() + 1);
            if policy.get() == 2 {
                Err(error.clone())
            } else {
                Ok(false)
            }
        }),
        activate: Rc::new(|_, _| panic!("lookup must not activate")),
        hover: None,
        leave: None,
    };
    let provider = NativeOscLinkProvider::new(
        Rc::downgrade(&native),
        web_sys::window().unwrap(),
        move || {
            settings.set(settings.get() + 1);
            Ok(Some(handler.clone()))
        },
    );
    let count = Rc::new(Cell::new(0));
    let last = Rc::new(RefCell::new(None));
    provider
        .provide_links(99, reply(count.clone(), last.clone()))
        .unwrap();
    assert_eq!(settings_calls.get(), 0);
    assert_eq!(count.get(), 1);
    assert_eq!(*last.borrow(), None);
    provider
        .provide_links(1, reply(count.clone(), last.clone()))
        .unwrap();
    assert_eq!(settings_calls.get(), 1);
    assert_eq!(policy_calls.get(), 0);
    assert_eq!(*last.borrow(), Some(0));
    native
        .write("\x1b]8;;http://one\x1b\\aa\x1b]8;;http://two\x1b\\bb\x1b]8;;\x1b\\x")
        .unwrap();
    assert_eq!(
        provider
            .provide_links(1, reply(count.clone(), last.clone()))
            .unwrap_err(),
        marker
    );
    assert_eq!(settings_calls.get(), 2);
    assert_eq!(policy_calls.get(), 2);
    assert_eq!(
        count.get(),
        2,
        "a thrown getter must not invoke the reply callback"
    );
}
