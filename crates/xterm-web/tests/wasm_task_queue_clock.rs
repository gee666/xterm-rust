#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::HtmlIFrameElement;
use xterm_web::task_queue::BrowserTaskQueue;
wasm_bindgen_test_configure!(run_in_browser);
#[path = "../../xterm-core/tests/support/task_queue_clock.rs"]
mod fixture;
use fixture::{cases, Expected};

struct Frame(HtmlIFrameElement);
impl Drop for Frame {
    fn drop(&mut self) {
        self.0.remove();
    }
}
fn frame() -> Frame {
    let document = web_sys::window().unwrap().document().unwrap();
    let element: HtmlIFrameElement = document
        .create_element("iframe")
        .unwrap()
        .dyn_into()
        .unwrap();
    document.body().unwrap().append_child(&element).unwrap();
    Frame(element)
}
#[wasm_bindgen_test]
fn clock_reentry_matches_unchanged_source_traces_and_error_identity() {
    let cases = cases();
    assert_eq!(cases.len(), 32);
    for case in cases {
        let frame = frame();
        let window = frame.0.content_window().unwrap();
        let trace = Array::new();
        let sentinel = Object::new();
        let callbacks = Array::new();
        let owner: Rc<RefCell<Option<BrowserTaskQueue>>> = Rc::new(RefCell::new(None));
        let weak = Rc::downgrade(&owner);
        let action = case.action.clone();
        let task_trace = trace.clone();
        let failure = sentinel.clone();
        let hook = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            let queue = weak.upgrade().unwrap().borrow().as_ref().unwrap().clone();
            if action == "flush" {
                queue.flush()?;
            } else {
                queue.clear();
                if action != "clear" {
                    let trace = task_trace.clone();
                    let failure = failure.clone();
                    let fail = action == "replace-error";
                    queue.enqueue_try(move |_| {
                        trace.push(&"task:new".into());
                        if fail {
                            Err(failure.clone().into())
                        } else {
                            Ok(false)
                        }
                    })?;
                }
            }
            Ok(())
        }) as Box<dyn Fn() -> Result<(), JsValue>>);
        // Only the host APIs are controlled. Dispatch and error translation run
        // through the production BrowserTaskQueue and the real WASM boundary.
        let timers = Function::new_with_args(
            "window,trace,callbacks",
            "let next=0;const request=cb=>{const id=++next;callbacks.push(cb);trace.push(`request:${id}`);return id;};
             window.setTimeout=window.requestIdleCallback=request;
             window.clearTimeout=window.cancelIdleCallback=id=>trace.push(`cancel:${id}`);",
        );
        timers
            .call3(&JsValue::NULL, &window, &trace, &callbacks)
            .unwrap();
        let clock = Function::new_with_args(
            "trace,hook,trigger",
            "let clock=0;return function(){trace.push(`now:${++clock}`);if(clock===trigger)hook();return 0;};",
        )
        .call3(
            &JsValue::NULL,
            &trace,
            hook.as_ref(),
            &JsValue::from_f64(case.trigger as f64),
        )
        .unwrap();
        Reflect::set(
            window.performance().unwrap().as_ref(),
            &"now".into(),
            &clock,
        )
        .unwrap();
        let queue = if case.mode == "idle" {
            BrowserTaskQueue::idle(&window).unwrap()
        } else {
            BrowserTaskQueue::priority(&window).unwrap()
        };
        *owner.borrow_mut() = Some(queue.clone());
        for name in &case.initial {
            let trace = trace.clone();
            let event = format!("task:{name}");
            queue
                .enqueue(move |_| {
                    trace.push(&event.clone().into());
                    false
                })
                .unwrap();
        }
        let deadline = Function::new_with_args(
            "trace",
            "return {timeRemaining(){trace.push('deadline');return 100;}};",
        )
        .call1(&JsValue::NULL, &trace)
        .unwrap();
        let result = callbacks
            .get(0)
            .unchecked_into::<Function>()
            .call1(&JsValue::NULL, &deadline);
        let error = result.err().map(|error| {
            if Object::is(&error, &sentinel) {
                "callback".to_string()
            } else {
                assert!(error.is_instance_of::<js_sys::TypeError>(), "{error:?}");
                "missing-task".to_string()
            }
        });
        let actual_trace: Vec<String> = trace
            .iter()
            .map(|event| event.as_string().unwrap())
            .collect();
        let actual = Expected {
            trace: actual_trace,
            error,
            remaining: queue.remaining_tasks(),
            pending: queue.has_pending_callback(),
        };
        assert_eq!(
            actual, case.expected,
            "{} {} clock{}",
            case.mode, case.action, case.trigger
        );
        queue.clear();
        queue.enqueue(|_| false).unwrap();
        queue.flush().unwrap();
        owner.borrow_mut().take();
    }
}
