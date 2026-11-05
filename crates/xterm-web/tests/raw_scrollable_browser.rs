#![cfg(target_arch = "wasm32")]
//! Shared live engine and raw update-object entry points. Element/scheduler remain301.
use js_sys::{Array, Function};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::{
    links::debounce::FrameScheduler,
    scroll::{
        scrollable::{ScrollError, Scrollable},
        state::{DimensionUpdate, PositionUpdate, ScrollState},
    },
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/raw_scrollable_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runLiveCases)]
    fn cases(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runClockCases)]
    fn clock_cases(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runGetterCases)]
    fn getter_cases(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runSchedulerCases)]
    fn scheduler_cases(run: &Function) -> Result<u32, JsValue>;
}
#[derive(Default)]
struct Frames {
    next: Cell<u32>,
    active: RefCell<Vec<u32>>,
    trace: RefCell<Vec<JsValue>>,
    fail_request: RefCell<Option<JsValue>>,
    fail_cancel: RefCell<Option<JsValue>>,
    request_hook: RefCell<Option<Function>>,
    cancel_hook: RefCell<Option<Function>>,
}
#[derive(Clone)]
struct Scheduler(Rc<Frames>);
impl FrameScheduler for Scheduler {
    type Handle = u32;
    type Error = JsValue;
    fn request_frame(&mut self) -> Result<u32, JsValue> {
        let hook = self.0.request_hook.borrow().clone();
        if let Some(hook) = hook {
            hook.call0(&JsValue::UNDEFINED)?;
        }
        if let Some(error) = self.0.fail_request.borrow().clone() {
            return Err(error);
        }
        let id = self.0.next.get() + 1;
        self.0.next.set(id);
        self.0.active.borrow_mut().push(id);
        self.0
            .trace
            .borrow_mut()
            .push(row(&["request".into(), id.into()]));
        Ok(id)
    }
    fn cancel_frame(&mut self, id: u32) -> Result<(), JsValue> {
        let hook = self.0.cancel_hook.borrow().clone();
        if let Some(hook) = hook {
            hook.call0(&JsValue::UNDEFINED)?;
        }
        if let Some(error) = self.0.fail_cancel.borrow().clone() {
            return Err(error);
        }
        self.0.active.borrow_mut().retain(|active| *active != id);
        self.0
            .trace
            .borrow_mut()
            .push(row(&["cancel".into(), id.into()]));
        Ok(())
    }
    fn cleanup_error(&mut self, error: JsValue) {
        web_sys::console::error_1(&error);
    }
    fn request_frame_shared(scheduler: &RefCell<Self>) -> Result<u32, JsValue> {
        let mut scheduler = scheduler.borrow().clone();
        scheduler.request_frame()
    }
    fn cancel_frame_shared(scheduler: &RefCell<Self>, id: u32) -> Result<(), JsValue> {
        let mut scheduler = scheduler.borrow().clone();
        scheduler.cancel_frame(id)
    }
}
fn row(values: &[JsValue]) -> JsValue {
    values.iter().collect::<Array>().into()
}
fn fields(state: ScrollState<JsValue>) -> JsValue {
    row(&[
        state.raw_left,
        state.raw_top,
        state.width,
        state.scroll_width,
        state.left,
        state.height,
        state.scroll_height,
        state.top,
    ])
}
fn optional(value: JsValue) -> Option<JsValue> {
    (!value.is_undefined()).then_some(value)
}
fn position(args: &Array) -> PositionUpdate<JsValue> {
    PositionUpdate {
        left: optional(args.get(1)),
        top: optional(args.get(2)),
    }
}
fn error(error: ScrollError<JsValue, JsValue>) -> JsValue {
    match error {
        ScrollError::Value(error) | ScrollError::Scheduler(error) => error,
        ScrollError::Dispatch(error) => js_sys::Error::new(&error.to_string()).into(),
        ScrollError::MissingAnimation => js_sys::TypeError::new("animation removed").into(),
    }
}
#[derive(Clone)]
enum Clock {
    Value(JsValue),
    Callback(Function),
}
fn run(duration: JsValue, setup: Function) -> Result<JsValue, JsValue> {
    let now = Rc::new(RefCell::new(Clock::Value(1000.0.into())));
    let read_now = now.clone();
    let frames = Rc::new(Frames::default());
    let model =
        Scrollable::try_new_with_clock(Scheduler(frames.clone()), false, duration, move || {
            let clock = read_now.borrow().clone();
            match clock {
                Clock::Value(value) => Ok(value),
                Clock::Callback(callback) => callback.call0(&JsValue::UNDEFINED),
            }
        })?;
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = events.clone();
    let listener = model.on_scroll().subscribe(move |event| {
        output.borrow_mut().push(row(&[
            event.in_smooth_scrolling.into(),
            fields(event.previous.clone()),
            fields(event.current.clone()),
            row(&[
                event.changed.width.into(),
                event.changed.scroll_width.into(),
                event.changed.left.into(),
                event.changed.height.into(),
                event.changed.scroll_height.into(),
                event.changed.top.into(),
            ]),
        ]));
    });
    let dispatch = Closure::<dyn Fn(Array) -> Result<JsValue, JsValue>>::new(move |args: Array| {
        let result = match args.get(0).as_string().as_deref() {
            Some("dimensions") => model
                .set_scroll_dimensions(
                    DimensionUpdate {
                        width: optional(args.get(1)),
                        scroll_width: optional(args.get(2)),
                        height: optional(args.get(3)),
                        scroll_height: optional(args.get(4)),
                    },
                    args.get(5).is_truthy(),
                )
                .map(|()| JsValue::UNDEFINED)
                .map_err(error),
            Some("now") => model
                .set_scroll_position_now(position(&args))
                .map(|()| JsValue::UNDEFINED)
                .map_err(error),
            Some("smooth") => model
                .set_scroll_position_smooth(position(&args), args.get(3).is_truthy())
                .map(|()| JsValue::UNDEFINED)
                .map_err(error),
            Some("rawDimensions") => model
                .set_raw_scroll_dimensions(&args.get(1), args.get(2).is_truthy())
                .map(|()| JsValue::UNDEFINED)
                .map_err(error),
            Some("rawNow") => model
                .set_raw_scroll_position_now(&args.get(1))
                .map(|()| JsValue::UNDEFINED)
                .map_err(error),
            Some("rawSmooth") => model
                .set_raw_scroll_position_smooth(&args.get(1), args.get(2).is_truthy())
                .map(|()| JsValue::UNDEFINED)
                .map_err(error),
            Some("rawValidate") => model
                .try_validate_raw_scroll_position(&args.get(1))
                .map(|value| row(&[value.left, value.top])),
            Some("duration") => {
                model.set_smooth_scroll_duration(args.get(1));
                Ok(JsValue::UNDEFINED)
            }
            Some("clock") => {
                *now.borrow_mut() = Clock::Value(args.get(1));
                Ok(JsValue::UNDEFINED)
            }
            Some("clockFn") => {
                *now.borrow_mut() = Clock::Callback(args.get(1).dyn_into::<Function>()?);
                Ok(JsValue::UNDEFINED)
            }
            Some("frame") => {
                let deliver = {
                    let mut active = frames.active.borrow_mut();
                    if active.is_empty() {
                        false
                    } else {
                        active.remove(0);
                        true
                    }
                };
                if deliver {
                    model.frame().map(|()| JsValue::UNDEFINED).map_err(error)
                } else {
                    Ok(JsValue::UNDEFINED)
                }
            }
            Some("requestFn") | Some("cancelFn") => {
                let hook = optional(args.get(1))
                    .map(JsCast::dyn_into::<Function>)
                    .transpose()?;
                if args.get(0).as_string().as_deref() == Some("requestFn") {
                    *frames.request_hook.borrow_mut() = hook;
                } else {
                    *frames.cancel_hook.borrow_mut() = hook;
                }
                Ok(JsValue::UNDEFINED)
            }
            Some("failRequest") => {
                *frames.fail_request.borrow_mut() = optional(args.get(1));
                Ok(JsValue::UNDEFINED)
            }
            Some("failCancel") => {
                *frames.fail_cancel.borrow_mut() = optional(args.get(1));
                Ok(JsValue::UNDEFINED)
            }
            Some("validate") => model
                .try_validate_scroll_position(position(&args))
                .map(|value| row(&[value.left, value.top])),
            Some("dispose") => model.dispose().map(|()| JsValue::UNDEFINED).map_err(error),
            Some("snapshot") => {
                let future = model.future_scroll_position();
                Ok(row(&[
                    fields(model.scroll_state()),
                    row(&[future.left, future.top]),
                    model.has_pending_scroll_animation().into(),
                    events.borrow().iter().collect::<Array>().into(),
                    frames.trace.borrow().iter().collect::<Array>().into(),
                    frames
                        .active
                        .borrow()
                        .iter()
                        .map(|id| JsValue::from(*id))
                        .collect::<Array>()
                        .into(),
                ]))
            }
            _ => return Err(js_sys::Error::new("unknown live test action").into()),
        };
        Ok(match result {
            Ok(value) => row(&[true.into(), value]),
            Err(error) => row(&[false.into(), error]),
        })
    });
    let result = setup.call1(&JsValue::UNDEFINED, dispatch.as_ref());
    listener.dispose();
    result
}
#[wasm_bindgen_test]
fn live_source_values_errors_and_coercion_reentry() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(cases(run.as_ref().unchecked_ref())?, 605);
    Ok(())
}
#[wasm_bindgen_test]
fn live_clock_values_errors_and_reentry() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(clock_cases(run.as_ref().unchecked_ref())?, 126);
    Ok(())
}
#[wasm_bindgen_test]
fn live_scheduler_reentry_and_error_identity() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(scheduler_cases(run.as_ref().unchecked_ref())?, 84);
    Ok(())
}
#[wasm_bindgen_test]
fn live_update_getters_reentry_and_error_identity() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(getter_cases(run.as_ref().unchecked_ref())?, 5804);
    Ok(())
}
