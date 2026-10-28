#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Object, Promise, Reflect};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlIFrameElement, Window};
use xterm_web::task_queue::BrowserTaskQueue;
wasm_bindgen_test_configure!(run_in_browser);
fn frame() -> (HtmlIFrameElement, Window) {
    let document = web_sys::window().unwrap().document().unwrap();
    let frame: HtmlIFrameElement = document
        .create_element("iframe")
        .unwrap()
        .dyn_into()
        .unwrap();
    document.body().unwrap().append_child(&frame).unwrap();
    let window = frame.content_window().unwrap();
    // Stable priority deadlines make these protocol-order tests independent of
    // machine load. Other scheduler tests execute the native browser clock.
    Reflect::set(
        window.performance().unwrap().as_ref(),
        &"now".into(),
        &Function::new_no_args("return 0"),
    )
    .unwrap();
    (frame, window)
}
fn record_timers(window: &Window, hook: &Function) -> (JsValue, Array, Array) {
    let old = Reflect::get(window, &"setTimeout".into()).unwrap();
    let callbacks = Array::new();
    let cancelled = Array::new();
    let factory=Function::new_with_args("callbacks,hook","let next=0;return function(callback){const id=++next;callbacks.push(callback);hook();return id;}");
    let request = factory.call2(&JsValue::NULL, &callbacks, hook).unwrap();
    Reflect::set(window, &"setTimeout".into(), &request).unwrap();
    let cancel = Function::new_with_args("cancelled", "return function(id){cancelled.push(id);}")
        .call1(&JsValue::NULL, &cancelled)
        .unwrap();
    Reflect::set(window, &"clearTimeout".into(), &cancel).unwrap();
    (old, callbacks, cancelled)
}
#[wasm_bindgen_test]
fn request_reentry_does_not_cancel_or_invalidate_an_overwritten_callback() {
    let (frame, window) = frame();
    let owner: Rc<RefCell<Option<BrowserTaskQueue>>> = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&owner);
    let first = Cell::new(true);
    let hook = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
        if first.replace(false) {
            let queue = weak.upgrade().unwrap().borrow().as_ref().unwrap().clone();
            queue.clear();
            queue.enqueue(|_| false)?;
        }
        Ok(())
    }) as Box<dyn Fn() -> Result<(), JsValue>>);
    let (old, callbacks, cancelled) = record_timers(&window, hook.as_ref().unchecked_ref());
    let queue = BrowserTaskQueue::priority(&window).unwrap();
    *owner.borrow_mut() = Some(queue.clone());
    queue.enqueue(|_| false).unwrap();
    assert_eq!(callbacks.length(), 2);
    assert_eq!(cancelled.length(), 0);
    queue.clear();
    assert_eq!(cancelled.length(), 1);
    assert_eq!(cancelled.get(0).as_f64(), Some(1.0));
    // Source overwrites token2 with token1 but never cancels callback2. It must
    // remain callable even after clearing the currently tracked token1.
    callbacks
        .get(1)
        .unchecked_into::<Function>()
        .call0(&JsValue::NULL)
        .unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
    Reflect::set(&window, &"setTimeout".into(), &old).unwrap();
    owner.borrow_mut().take();
    drop(queue);
    frame.remove();
}
#[wasm_bindgen_test]
fn callback_retained_before_request_throws_remains_callable() {
    let (frame, window) = frame();
    let callbacks = Array::new();
    let sentinel = Object::new();
    let factory = Function::new_with_args(
        "callbacks,error",
        "return function(callback){callbacks.push(callback);throw error;}",
    );
    let request = factory
        .call2(&JsValue::NULL, &callbacks, &sentinel)
        .unwrap();
    let old = Reflect::get(&window, &"setTimeout".into()).unwrap();
    Reflect::set(&window, &"setTimeout".into(), &request).unwrap();
    let queue = BrowserTaskQueue::priority(&window).unwrap();
    let calls = Rc::new(Cell::new(0));
    let seen = calls.clone();
    let error = queue
        .enqueue(move |_| {
            seen.set(seen.get() + 1);
            false
        })
        .unwrap_err();
    assert!(Object::is(&error, &sentinel));
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(!queue.has_pending_callback());
    callbacks
        .get(0)
        .unchecked_into::<Function>()
        .call0(&JsValue::NULL)
        .unwrap();
    assert_eq!(calls.get(), 1);
    Reflect::set(&window, &"setTimeout".into(), &old).unwrap();
    drop(queue);
    frame.remove();
}
#[wasm_bindgen_test(async)]
async fn iframe_idle_deadline_uses_its_own_realm_interface() {
    let (frame, window) = frame();
    let queue = BrowserTaskQueue::idle(&window).unwrap();
    let promise = Promise::new(&mut |resolve, _| {
        let resolve = resolve.clone();
        queue
            .enqueue_try(move |_| {
                resolve.call0(&JsValue::NULL)?;
                Ok(false)
            })
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
    queue.clear();
    drop(queue);
    frame.remove();
}
#[wasm_bindgen_test(async)]
async fn source_idle_choice_is_cached_before_global_capability_changes() {
    let window = web_sys::window().unwrap();
    let key = JsValue::from_str("requestIdleCallback");
    let existed = Reflect::has(&window, &key).unwrap();
    let original = Reflect::get(&window, &key).unwrap();
    if existed {
        Reflect::set(&window, &key, &JsValue::UNDEFINED).unwrap();
        let queue = BrowserTaskQueue::source_idle().unwrap();
        let result = queue.enqueue(|_| false);
        Reflect::set(&window, &key, &original).unwrap();
        assert!(result.is_err());
        queue.flush().unwrap();
    } else {
        Reflect::set(
            &window,
            &key,
            &Function::new_no_args("throw Error('late idle function must not be selected')"),
        )
        .unwrap();
        let queue = BrowserTaskQueue::source_idle().unwrap();
        let promise = Promise::new(&mut |resolve, _| {
            let resolve = resolve.clone();
            queue
                .enqueue_try(move |_| {
                    resolve.call0(&JsValue::NULL)?;
                    Ok(false)
                })
                .unwrap();
        });
        Reflect::delete_property(&window, &key).unwrap();
        JsFuture::from(promise).await.unwrap();
        assert_eq!(queue.remaining_tasks(), 0);
        queue.clear();
    }
}
#[wasm_bindgen_test]
fn fallible_flush_retains_error_identity_and_retry_index() {
    let (frame, window) = frame();
    let queue = BrowserTaskQueue::priority(&window).unwrap();
    let sentinel = Object::new();
    let failure = sentinel.clone();
    let first = Cell::new(true);
    queue
        .enqueue_try(move |_| {
            if first.replace(false) {
                Err(failure.clone().into())
            } else {
                Ok(false)
            }
        })
        .unwrap();
    let error = queue.flush().unwrap_err();
    assert!(Object::is(&error, &sentinel));
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(queue.has_pending_callback());
    queue.flush().unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
    assert!(!queue.has_pending_callback());
    frame.remove();
}
