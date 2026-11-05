#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::links::browser_debounce::BrowserTimeDebouncer;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/timer_scheduler_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runTimerSchedulerCases)]
    fn cases(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runRawTimerSchedulerCases)]
    fn raw_cases(run: &Function) -> Result<u32, JsValue>;
}
fn run(host: JsValue, program: Function) -> Result<JsValue, JsValue> {
    let events: Array = js_sys::Reflect::get(&host, &"events".into())?.dyn_into()?;
    let window = js_sys::Reflect::get(&host, &"window".into())?.unchecked_into();
    let output = events.clone();
    let model = Rc::new(BrowserTimeDebouncer::try_new(
        window,
        100.0,
        move |range| {
            output.push(
                &[
                    JsValue::from("render"),
                    range.start.into(),
                    range.end.into(),
                ]
                .into_iter()
                .collect::<Array>(),
            );
            Ok(())
        },
    )?);
    let nested = model.clone();
    let dispatch = Closure::<dyn Fn(Array) -> Array>::new(move |args: Array| {
        let result = match args.get(0).as_string().as_deref() {
            Some("refresh") => nested.refresh(
                args.get(1).as_f64(),
                args.get(2).as_f64(),
                args.get(3).as_f64().expect("numeric test count"),
            ),
            Some("dispose") => nested.dispose(),
            _ => Err(js_sys::Error::new("unknown timer action").into()),
        };
        match result {
            Ok(()) => [JsValue::TRUE, JsValue::UNDEFINED],
            Err(error) => [JsValue::FALSE, error],
        }
        .into_iter()
        .collect()
    });
    let result = program.call1(&JsValue::UNDEFINED, dispatch.as_ref());
    model.dispose()?;
    result
}
#[wasm_bindgen_test]
fn real_timer_scheduler_matches_source_reentry_and_errors() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(cases(run.as_ref().unchecked_ref())?, 76);
    Ok(())
}
#[wasm_bindgen_test]
fn real_timer_scheduler_retains_raw_thrown_values() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(raw_cases(run.as_ref().unchecked_ref())?, 42);
    Ok(())
}
#[wasm_bindgen_test(async)]
async fn real_window_timeout_delivers_rows_retained_after_cancel() -> Result<(), JsValue> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let owner = RefCell::new(None);
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let setup = || {
            let output = events.clone();
            let resolve = resolve.clone();
            // Infinity always takes the trailing branch. WebIDL converts its
            // timeout delay to zero, so delivery is asynchronous without a sleep
            // or an assumption about how long the page has been alive.
            let model = BrowserTimeDebouncer::try_new(
                web_sys::window().expect("browser window"),
                f64::INFINITY,
                move |range| {
                    output.borrow_mut().push((range.start, range.end));
                    resolve.call0(&JsValue::UNDEFINED).map(|_| ())
                },
            )?;
            model.refresh(Some(3.0), Some(9.0), 10.0)?;
            model.dispose()?;
            model.refresh(Some(2.0), Some(6.0), 8.0)?;
            assert!(events.borrow().is_empty());
            Ok::<_, JsValue>(model)
        };
        match setup() {
            Ok(model) => {
                owner.replace(Some(model));
            }
            Err(error) => {
                reject
                    .call1(&JsValue::UNDEFINED, &error)
                    .expect("reject test promise");
            }
        }
    });
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    assert_eq!(*events.borrow(), [(2.0, 7.0)]);
    owner
        .borrow()
        .as_ref()
        .expect("initialized timer")
        .dispose()?;
    Ok(())
}
