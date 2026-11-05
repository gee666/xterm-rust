#![cfg(target_arch = "wasm32")]
//! Exercise the actual BrowserScrollScheduler, not the raw test scheduler.
use js_sys::{Array, Function};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::{
    links::animation_queue::WindowAnimationQueues,
    scroll::{
        browser_scheduler::{new_scrollable_with_queue_source, scroll_error},
        state::{DimensionUpdate, PositionUpdate},
    },
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/browser_scroll_scheduler_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runBrowserSchedulerCases)]
    fn cases(run: &Function) -> Result<u32, JsValue>;
}
fn run(setup: Function) -> Result<JsValue, JsValue> {
    let window = web_sys::window().expect("browser test window");
    let queue = WindowAnimationQueues::default().for_window(&window);
    let hook = Rc::new(RefCell::new(None::<Function>));
    let read_hook = hook.clone();
    let model = new_scrollable_with_queue_source(
        Rc::new(move || {
            let callback = read_hook.borrow().clone();
            if let Some(callback) = callback {
                callback.call0(&JsValue::UNDEFINED)?;
            }
            Ok(queue.clone())
        }),
        false,
        90.0,
    );
    let dispatch = Closure::<dyn Fn(Array) -> Result<JsValue, JsValue>>::new(move |args: Array| {
        let result = match args.get(0).as_string().as_deref() {
            Some("hook") => {
                *hook.borrow_mut() = if args.get(1).is_undefined() {
                    None
                } else {
                    Some(args.get(1).dyn_into::<Function>()?)
                };
                Ok(JsValue::UNDEFINED)
            }
            Some("dimensions") => model
                .set_scroll_dimensions(
                    DimensionUpdate {
                        width: args.get(1).as_f64(),
                        scroll_width: args.get(2).as_f64(),
                        height: args.get(3).as_f64(),
                        scroll_height: args.get(4).as_f64(),
                    },
                    false,
                )
                .map(|()| JsValue::UNDEFINED)
                .map_err(scroll_error),
            Some("now") => model
                .set_scroll_position_now(PositionUpdate {
                    top: args.get(1).as_f64(),
                    ..Default::default()
                })
                .map(|()| JsValue::UNDEFINED)
                .map_err(scroll_error),
            Some("smooth") => model
                .set_scroll_position_smooth(
                    PositionUpdate {
                        top: args.get(1).as_f64(),
                        ..Default::default()
                    },
                    args.get(2).is_truthy(),
                )
                .map(|()| JsValue::UNDEFINED)
                .map_err(scroll_error),
            Some("dispose") => model
                .dispose()
                .map(|()| JsValue::UNDEFINED)
                .map_err(scroll_error),
            Some("snapshot") => {
                let state = model.scroll_state();
                Ok([
                    JsValue::from(state.width),
                    state.scroll_width.into(),
                    state.height.into(),
                    state.scroll_height.into(),
                    state.top.into(),
                    model.future_scroll_position().top.into(),
                    model.has_pending_scroll_animation().into(),
                ]
                .into_iter()
                .collect::<Array>()
                .into())
            }
            _ => return Err(js_sys::Error::new("unknown queue test action").into()),
        };
        Ok(match result {
            Ok(value) => [JsValue::TRUE, value].into_iter().collect::<Array>().into(),
            Err(error) => [JsValue::FALSE, error]
                .into_iter()
                .collect::<Array>()
                .into(),
        })
    });
    setup.call1(&JsValue::UNDEFINED, dispatch.as_ref())
}
#[wasm_bindgen_test]
fn actual_browser_queue_source_can_reenter_and_throw_original_value() -> Result<(), JsValue> {
    let run = Closure::wrap(Box::new(run) as Box<dyn Fn(Function) -> Result<JsValue, JsValue>>);
    assert_eq!(cases(run.as_ref().unchecked_ref())?, 18);
    Ok(())
}
