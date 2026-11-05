#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::links::browser_debounce::BrowserRenderDebouncer;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/render_scheduler_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runRenderSchedulerCases)]
    fn cases(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runRenderWindowFailureCases)]
    fn window_failures(run: &Function) -> Result<u32, JsValue>;
}
fn invoke(host: &JsValue, name: &str, args: &Array) -> Result<JsValue, JsValue> {
    let function: Function = js_sys::Reflect::get(host, &name.into())?.dyn_into()?;
    function.apply(host, args)
}
fn run(host: JsValue, program: Function) -> Result<JsValue, JsValue> {
    let events: Array = js_sys::Reflect::get(&host, &"events".into())?.dyn_into()?;
    let source = host.clone();
    let render = events.clone();
    let model = Rc::new(BrowserRenderDebouncer::try_new_with_fallible_window_source(
        Rc::new(move || invoke(&source, "owner", &Array::new()).map(JsCast::unchecked_into)),
        move |range| {
            render.push(
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
    ));
    let dispatch_model = model.clone();
    let dispatch = Closure::<dyn Fn(Array) -> Result<JsValue, JsValue>>::new(move |args: Array| {
        let result = match args.get(0).as_string().as_deref() {
            Some("refresh") => dispatch_model
                .refresh(
                    args.get(1).as_f64(),
                    args.get(2).as_f64(),
                    args.get(3).as_f64().expect("numeric test row count"),
                )
                .map(|()| JsValue::UNDEFINED),
            Some("callback") => {
                let events = events.clone();
                let tag = args.get(1);
                dispatch_model
                    .add_refresh_callback(move |time| {
                        events.push(
                            &[JsValue::from("callback"), tag.clone(), time.into()]
                                .into_iter()
                                .collect::<Array>(),
                        );
                    })
                    .map(JsValue::from)
            }
            Some("dispose") => dispatch_model.dispose().map(|()| JsValue::UNDEFINED),
            _ => return Err(js_sys::Error::new("unknown render action").into()),
        };
        Ok(match result {
            Ok(value) => [JsValue::TRUE, value],
            Err(error) => [JsValue::FALSE, error],
        }
        .into_iter()
        .collect::<Array>()
        .into())
    });
    program.call1(&JsValue::UNDEFINED, dispatch.as_ref())
}
#[wasm_bindgen_test]
fn real_render_scheduler_matches_source_reentry_and_error_identity() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(cases(run.as_ref().unchecked_ref())?, 72);
    Ok(())
}

#[wasm_bindgen_test]
fn current_window_preserves_every_javascript_thrown_value() -> Result<(), JsValue> {
    let run =
        Closure::wrap(Box::new(run) as Box<dyn Fn(JsValue, Function) -> Result<JsValue, JsValue>>);
    assert_eq!(window_failures(run.as_ref().unchecked_ref())?, 42);
    Ok(())
}

#[wasm_bindgen_test(async)]
async fn real_animation_frame_delivers_retained_rows_then_zero_time_callbacks(
) -> Result<(), JsValue> {
    let events = Rc::new(RefCell::new(Vec::new()));
    let output = events.clone();
    let model = BrowserRenderDebouncer::new(web_sys::window().expect("browser window"), move |r| {
        output
            .borrow_mut()
            .push(format!("render:{}:{}", r.start, r.end));
    });
    model.refresh(Some(2.0), Some(20.0), 10.0)?;
    model.dispose()?;
    model.refresh(Some(4.0), Some(6.0), 8.0)?;
    let events_out = events.clone();
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let resolve = resolve.clone();
        let output = events_out.clone();
        if let Err(error) = model.add_try_refresh_callback(move |time| {
            output.borrow_mut().push(format!("callback:{time}"));
            resolve.call0(&JsValue::UNDEFINED).map(|_| ())
        }) {
            reject
                .call1(&JsValue::UNDEFINED, &error)
                .expect("test Promise rejection");
        }
    });
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    assert_eq!(*events.borrow(), ["render:2:7", "callback:0"]);
    model.dispose()?;
    Ok(())
}
