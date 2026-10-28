#![cfg(target_arch = "wasm32")]

use js_sys::{Function, Promise, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{HtmlIFrameElement, Window};
use xterm_web::task_queue::BrowserTaskQueue;

wasm_bindgen_test_configure!(run_in_browser);

fn iframe() -> HtmlIFrameElement {
    let document = web_sys::window().unwrap().document().unwrap();
    let frame: HtmlIFrameElement = document
        .create_element("iframe")
        .unwrap()
        .dyn_into()
        .unwrap();
    document.body().unwrap().append_child(&frame).unwrap();
    frame
}

fn restore(window: &Window, key: &JsValue, original: &JsValue) {
    if original.is_undefined() {
        Reflect::delete_property(window, key).unwrap();
    } else {
        Reflect::set(window, key, original).unwrap();
    }
}

#[wasm_bindgen_test]
fn throwing_timer_and_idle_requests_release_borrows_and_callbacks() {
    for idle in [false, true] {
        let frame = iframe();
        let window = frame.content_window().unwrap();
        let key = JsValue::from_str(if idle {
            "requestIdleCallback"
        } else {
            "setTimeout"
        });
        let original = Reflect::get(&window, &key).unwrap();
        Reflect::set(
            &window,
            &key,
            &Function::new_no_args("throw new Error('schedule failure')"),
        )
        .unwrap();
        let queue = if idle {
            BrowserTaskQueue::idle(&window)
        } else {
            BrowserTaskQueue::priority(&window)
        }
        .unwrap();
        let calls = Rc::new(Cell::new(0));
        let observed = Rc::clone(&calls);
        let result = queue.enqueue(move |_| {
            observed.set(observed.get() + 1);
            false
        });
        restore(&window, &key, &original);
        assert!(result.is_err());
        assert!(!queue.has_pending_callback());
        assert_eq!(queue.remaining_tasks(), 1);
        assert_eq!(calls.get(), 0);
        queue.flush().unwrap();
        assert_eq!(calls.get(), 1);
        assert_eq!(queue.remaining_tasks(), 0);
        // Queue kind is chosen at construction, as upstream's class choice is.
        // WebKit has no native idle callback to restore. A second request must
        // return that host error again, not panic from a poisoned RefCell.
        if idle && original.is_undefined() {
            assert!(queue.enqueue(|_| false).is_err());
            queue.flush().unwrap();
        } else {
            queue.enqueue(|_| false).unwrap();
            queue.clear();
        }
        drop(queue);
        frame.remove();
    }
}

#[wasm_bindgen_test(async)]
async fn idle_fallback_uses_a_live_zero_delay_timer() {
    let frame = iframe();
    let window = frame.content_window().unwrap();
    let key = JsValue::from_str("requestIdleCallback");
    let original = Reflect::get(&window, &key).unwrap();
    assert!(Reflect::delete_property(&window, &key).unwrap());
    assert!(!Reflect::has(&window, &key).unwrap());
    let queue = BrowserTaskQueue::idle(&window).unwrap();
    let result = Promise::new(&mut |resolve, _| {
        let resolve = resolve.clone();
        queue
            .enqueue(move |_| {
                resolve.call0(&JsValue::NULL).unwrap();
                false
            })
            .unwrap();
    });
    JsFuture::from(result).await.unwrap();
    restore(&window, &key, &original);
    assert_eq!(queue.remaining_tasks(), 0);
    drop(queue);
    frame.remove();
}

#[wasm_bindgen_test(async)]
async fn dropping_final_owner_cancels_host_work_without_a_cycle() {
    let window = web_sys::window().unwrap();
    let calls = Rc::new(Cell::new(0));
    let queue = BrowserTaskQueue::priority(&window).unwrap();
    let observed = Rc::clone(&calls);
    queue
        .enqueue(move |_| {
            observed.set(observed.get() + 1);
            false
        })
        .unwrap();
    drop(queue);
    let sentinel = BrowserTaskQueue::priority(&window).unwrap();
    let promise = Promise::new(&mut |resolve, _| {
        let resolve = resolve.clone();
        sentinel
            .enqueue(move |_| {
                resolve.call0(&JsValue::NULL).unwrap();
                false
            })
            .unwrap();
    });
    JsFuture::from(promise).await.unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(Rc::strong_count(&calls), 1);
}
