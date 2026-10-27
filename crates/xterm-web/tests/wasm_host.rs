#![cfg(target_arch = "wasm32")]

use std::{cell::{Cell, RefCell}, rc::Rc};
use js_sys::{JsString, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Event, HtmlElement};
use xterm_web::{
    dom::{BrowserDom, DomOptions}, dom_listener::DomListener,
    task_queue::BrowserTaskQueue, utf16::{from_js_string, to_js_string}, BrowserHost,
};

wasm_bindgen_test_configure!(run_in_browser);

fn parent() -> HtmlElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    parent
}

#[wasm_bindgen_test]
fn exact_dom_structure_and_attributes() {
    let parent = parent();
    let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
    assert_eq!(dom.element.class_name(), "terminal xterm");
    assert_eq!(dom.element.dir(), "ltr");
    assert_eq!(dom.element.children().length(), 2);
    assert_eq!(dom.element.first_element_child().unwrap(), dom.viewport.clone().into());
    assert_eq!(dom.element.last_element_child().unwrap(), dom.screen.clone().into());
    assert_eq!(dom.helpers.parent_element().unwrap(), dom.screen.clone().into());
    assert_eq!(dom.textarea.parent_element().unwrap(), dom.helpers.clone().into());
    assert_eq!(dom.composition.parent_element().unwrap(), dom.helpers.clone().into());
    assert_eq!(dom.textarea.get_attribute("aria-label").unwrap(), "Terminal input");
    let source_textarea = dom.document.create_element("textarea").unwrap();
    for (key, value) in [
        ("aria-multiline", "false"), ("autocomplete", "off"),
        ("autocorrect", "off"), ("autocapitalize", "off"), ("spellcheck", "false"),
    ] {
        // Upstream uses setAttribute. Firefox canonicalizes autocapitalize=off
        // to none, so compare the source operation in the same browser.
        source_textarea.set_attribute(key, value).unwrap();
        assert_eq!(dom.textarea.get_attribute(key), source_textarea.get_attribute(key), "{key}");
    }
    assert_eq!(dom.textarea.tab_index(), 0);
    assert!(!dom.textarea.read_only());
    drop(dom);
    assert_eq!(parent.child_element_count(), 0);
    parent.remove();
}

#[wasm_bindgen_test]
fn options_update_existing_nodes_and_chromeos_accessibility() {
    let parent = parent();
    let options = DomOptions {
        allow_transparency: true, disable_stdin: true, is_chrome_os: true,
        prompt_label: "入力 🦀".into(), ..DomOptions::default()
    };
    let dom = BrowserDom::new(&parent, &options).unwrap();
    assert!(dom.element.class_list().contains("allow-transparency"));
    assert!(dom.textarea.read_only());
    assert_eq!(dom.textarea.get_attribute("aria-multiline"), None);
    assert_eq!(dom.textarea.get_attribute("aria-label").unwrap(), "入力 🦀");
    dom.textarea.set_value("retained composition");
    dom.set_allow_transparency(false).unwrap();
    dom.set_disable_stdin(false);
    assert!(!dom.element.class_list().contains("allow-transparency"));
    assert!(!dom.textarea.read_only());
    assert_eq!(dom.textarea.value(), "retained composition");
    drop(dom);
    parent.remove();
}

#[wasm_bindgen_test]
fn dispose_removes_only_owned_root_and_is_repeatable() {
    let parent = parent();
    let sibling = parent.owner_document().unwrap().create_element("span").unwrap();
    parent.append_child(&sibling).unwrap();
    let mut dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
    dom.dispose();
    dom.dispose();
    assert!(dom.is_disposed());
    assert!(dom.element.parent_element().is_none());
    assert_eq!(parent.child_element_count(), 1);
    assert_eq!(parent.first_element_child().unwrap(), sibling);
    drop(dom);
    assert_eq!(parent.child_element_count(), 1);
    parent.remove();
}

#[wasm_bindgen_test]
fn repeated_open_does_not_move_root_or_replace_nodes() {
    let first = parent();
    let second = parent();
    let mut host = BrowserHost::new();
    assert!(host.element().is_none());
    host.focus().unwrap();
    host.blur().unwrap();
    host.open(first.clone()).unwrap();
    let root = host.element().unwrap();
    let textarea = host.textarea().unwrap();
    host.open(second.clone()).unwrap();
    assert_eq!(host.element().unwrap(), root);
    assert_eq!(host.textarea().unwrap(), textarea);
    assert_eq!(root.parent_element().unwrap(), first.clone().into());
    assert_eq!(second.child_element_count(), 0);
    host.focus().unwrap();
    assert_eq!(first.owner_document().unwrap().active_element().unwrap(), textarea.clone().into());
    host.blur().unwrap();
    host.dispose();
    host.dispose();
    assert!(root.parent_element().is_none());
    assert!(host.open(second.clone()).is_err());
    first.remove();
    second.remove();
}

#[wasm_bindgen_test]
fn detached_parent_is_valid() {
    let parent = parent();
    parent.remove();
    let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
    assert_eq!(dom.element.parent_element().unwrap(), parent.clone().into());
    assert!(!dom.element.is_connected());
}

#[wasm_bindgen_test]
fn listener_explicit_disposal_and_drop_remove_callbacks() {
    let element = parent();
    let count = Rc::new(Cell::new(0));
    let observed = Rc::clone(&count);
    let mut listener = DomListener::new(element.as_ref(), "test-event", true, move |_| {
        observed.set(observed.get() + 1);
    }).unwrap();
    element.dispatch_event(&Event::new("test-event").unwrap()).unwrap();
    assert_eq!(count.get(), 1);
    listener.dispose().unwrap();
    listener.dispose().unwrap();
    assert!(listener.is_disposed());
    element.dispatch_event(&Event::new("test-event").unwrap()).unwrap();
    assert_eq!(count.get(), 1);
    let observed = Rc::clone(&count);
    let listener = DomListener::new(element.as_ref(), "test-event", false, move |_| {
        observed.set(observed.get() + 1);
    }).unwrap();
    drop(listener);
    element.dispatch_event(&Event::new("test-event").unwrap()).unwrap();
    assert_eq!(count.get(), 1);
    element.remove();
}

#[wasm_bindgen_test]
fn listener_can_dispose_itself_during_dispatch() {
    let element = parent();
    let owner: Rc<RefCell<Option<DomListener>>> = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&owner);
    let count = Rc::new(Cell::new(0));
    let observed = Rc::clone(&count);
    *owner.borrow_mut() = Some(DomListener::new(element.as_ref(), "self-dispose", false, move |_| {
        observed.set(observed.get() + 1);
        weak.upgrade().unwrap().borrow_mut().take().unwrap().dispose().unwrap();
    }).unwrap());
    element.dispatch_event(&Event::new("self-dispose").unwrap()).unwrap();
    element.dispatch_event(&Event::new("self-dispose").unwrap()).unwrap();
    assert_eq!(count.get(), 1);
    element.remove();
}

#[wasm_bindgen_test]
fn utf16_roundtrip_preserves_every_codeunit_and_large_payloads() {
    let units: Vec<u16> = (0..=u16::MAX).collect();
    assert_eq!(from_js_string(&to_js_string(&units)), units);
    let units = vec![0xD800, 0, 0xDC00, 0xFEFF, 0xD83E, 0xDD80].repeat(40_000);
    assert_eq!(from_js_string(&to_js_string(&units)), units);
    assert_eq!(from_js_string(&JsString::from("")), Vec::<u16>::new());
    assert_eq!(to_js_string(&[]).length(), 0);
}

async fn drain(queue: &BrowserTaskQueue) {
    let promise = Promise::new(&mut |resolve, _| {
        let resolve = resolve.clone();
        queue.enqueue(move |_| {
            resolve.call0(&JsValue::NULL).unwrap();
            false
        });
    });
    JsFuture::from(promise).await.unwrap();
}

#[wasm_bindgen_test(async)]
async fn priority_queue_repeats_fifo_and_releases_callback() {
    let queue = BrowserTaskQueue::priority(&web_sys::window().unwrap()).unwrap();
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    queue.enqueue(move |_| {
        observed.set(observed.get() + 1);
        observed.get() < 3
    });
    assert_eq!(calls.get(), 0);
    assert!(queue.has_pending_callback());
    drain(&queue).await;
    assert_eq!(calls.get(), 3);
    assert_eq!(queue.remaining_tasks(), 0);
}

#[wasm_bindgen_test(async)]
async fn idle_queue_and_cancellation_use_live_event_loop() {
    let window = web_sys::window().unwrap();
    for queue in [BrowserTaskQueue::priority(&window).unwrap(), BrowserTaskQueue::idle(&window).unwrap()] {
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        queue.enqueue(move |_| { observed.set(observed.get() + 1); false });
        queue.clear();
        assert!(!queue.has_pending_callback());
        drain(&queue).await;
        assert_eq!(calls.get(), 0);
        let observed = Rc::clone(&calls);
        queue.enqueue(move |_| { observed.set(observed.get() + 1); false });
        queue.flush();
        assert_eq!(calls.get(), 1);
        assert!(!queue.has_pending_callback());
        drain(&queue).await;
        assert_eq!(calls.get(), 1);
    }
}
