#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Promise};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Document, HtmlElement, UiEvent, Window};
use xterm_core::lifecycle::Disposable;
use xterm_web::links::{
    animation_queue::WindowAnimationQueues,
    dom_helpers::{get_window, page_position},
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test(async)]
async fn priority_stability_cancellation_and_reentrant_next_frame() {
    let window = web_sys::window().unwrap();
    let mut queues = WindowAnimationQueues::default();
    let queue = queues.for_window(&window);
    let same_window = queues.for_window(&window);
    let log = Rc::new(RefCell::new(Vec::new()));
    let finish = Rc::new(RefCell::new(None));
    let resolve_later = finish.clone();
    let second_frame = Promise::new(&mut |resolve, _| {
        *finish.borrow_mut() = Some(resolve);
    });
    let out = log.clone();
    queue
        .schedule(0.0, move |_| {
            out.borrow_mut().push("zero");
            Ok(())
        })
        .unwrap();
    let out = log.clone();
    queue
        .schedule(10.0, move |next_queue| {
            out.borrow_mut().push("first ten");
            let log = out.clone();
            let finish = resolve_later.clone();
            next_queue.schedule(100.0, move |_| {
                log.borrow_mut().push("next frame");
                let resolve: Function = finish.borrow_mut().take().unwrap();
                resolve.call0(&JsValue::NULL).unwrap();
                Ok(())
            })?;
            Ok(())
        })
        .unwrap();
    let out = log.clone();
    same_window
        .schedule(10.0, move |_| {
            out.borrow_mut().push("second ten");
            Ok(())
        })
        .unwrap();
    let out = log.clone();
    let mutable = queue
        .schedule(-1.0, move |_| {
            out.borrow_mut().push("raised");
            Ok(())
        })
        .unwrap();
    mutable.set_priority(20.0);
    let canceled = queue
        .schedule(1000.0, |_| panic!("canceled item must not run"))
        .unwrap();
    canceled.dispose();
    canceled.dispose();
    let out = log.clone();
    queue
        .schedule(15.0, move |_| {
            out.borrow_mut().push("error");
            Err(JsValue::from_str("expected queue callback error"))
        })
        .unwrap();
    let first_frame = Promise::new(&mut |resolve, _| {
        queue
            .schedule(f64::NEG_INFINITY, move |_| {
                resolve.call0(&JsValue::NULL).unwrap();
                Ok(())
            })
            .unwrap();
    });
    JsFuture::from(first_frame).await.unwrap();
    assert_eq!(
        *log.borrow(),
        vec!["raised", "error", "first ten", "second ten", "zero"]
    );
    JsFuture::from(second_frame).await.unwrap();
    assert_eq!(log.borrow().last(), Some(&"next frame"));
}
#[wasm_bindgen_test]
fn owner_document_event_view_and_page_coordinates() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let iframe: web_sys::HtmlIFrameElement = document
        .create_element("iframe")
        .unwrap()
        .dyn_into()
        .unwrap();
    document.body().unwrap().append_child(&iframe).unwrap();
    let child_window: Window = iframe.content_window().unwrap();
    let child_document: Document = iframe.content_document().unwrap();
    // HTMLDocument.createElement('div') guarantees HTMLElement, but instanceof
    // checks against the parent's constructor are not valid across realms.
    let element: HtmlElement = child_document
        .create_element("div")
        .unwrap()
        .unchecked_into();
    element
        .style()
        .set_css_text("position:absolute;left:100px;top:2000px;width:120px;height:50px;");
    child_document
        .body()
        .unwrap()
        .append_child(&element)
        .unwrap();
    child_window.scroll_to_with_x_and_y(0.0, 100.0);
    assert_eq!(
        get_window(Some(element.as_ref()), None, &window),
        child_window
    );
    let create = Function::new_with_args("view", "return new UIEvent('click', { view });");
    let event: UiEvent = create
        .call1(&JsValue::NULL, child_window.as_ref())
        .unwrap()
        .dyn_into()
        .unwrap();
    assert_eq!(get_window(None, Some(&event), &window), child_window);
    // Node ownerDocument wins over an unrelated event view.
    let outer: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    assert_eq!(
        get_window(Some(outer.as_ref()), Some(&event), &window),
        window
    );
    assert_eq!(get_window(None, None, &window), window);
    let page = page_position(&element, &window).unwrap();
    assert_eq!(
        (page.left, page.top, page.width, page.height),
        (100.0, 2000.0, 120.0, 50.0)
    );
    iframe.remove();
}

#[wasm_bindgen_test(async)]
async fn interval_cancels_reentrantly_and_rejects_reuse_after_dispose() {
    use std::cell::Cell;
    use xterm_web::links::window_interval::WindowIntervalTimer;
    let window = web_sys::window().unwrap();
    let timer = WindowIntervalTimer::new(None, window.clone());
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    let completed = Promise::new(&mut |resolve, _| {
        let output = output.clone();
        let timer_handle = timer.downgrade();
        timer
            .cancel_and_set(
                move || {
                    output.set(output.get() + 1);
                    timer_handle.upgrade().unwrap().cancel()?;
                    resolve.call0(&JsValue::NULL)?;
                    Ok(())
                },
                0.0,
                None,
            )
            .unwrap();
    });
    JsFuture::from(completed).await.unwrap();
    assert_eq!(count.get(), 1);
    timer.dispose().unwrap();
    timer.dispose().unwrap();
    let error = timer
        .cancel_and_set(|| Ok(()), 0.0, Some(&window))
        .unwrap_err();
    assert_eq!(
        error.dyn_into::<js_sys::Error>().unwrap().message().as_string().unwrap(),
        "Calling cancelAndSet on a disposed IntervalTimer"
    );
}
