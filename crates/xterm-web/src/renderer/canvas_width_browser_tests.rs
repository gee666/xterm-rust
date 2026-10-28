use super::*;
use js_sys::{Array, Function, JsString, Object, Reflect};
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_test::*;
use xterm_core::options::OptionValue;
fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}
fn iframe() -> Result<(web_sys::Element, Document), JsValue> {
    let d = document();
    let frame = d.create_element("iframe")?;
    d.body().unwrap().append_child(&frame)?;
    let child = Reflect::get(&frame, &"contentDocument".into())?;
    assert!(!child.is_null());
    Ok((frame, child.unchecked_into()))
}
fn dom_measurer(document: &Document) -> Result<CanvasWidthMeasurer<RawFont>, JsValue> {
    let global = js_sys::global();
    let key = JsValue::from_str("OffscreenCanvas");
    let old = Reflect::get(&global, &key)?;
    Reflect::set(&global, &key, &JsValue::UNDEFINED)?;
    let result = CanvasWidthMeasurer::new(document);
    Reflect::set(&global, &key, &old)?;
    result
}
fn font() -> RawFont {
    RawFont {
        family: "monospace".into(),
        size: 15.0.into(),
        ..RawFont::default()
    }
}
#[wasm_bindgen_test]
fn iframe_canvas_measures_and_flat_cache_rounds_only_after_first_read() -> Result<(), JsValue> {
    let (frame, d) = iframe()?;
    let measurer = dom_measurer(&d)?;
    let foreign = Function::new_with_args("ctx", "return ctx instanceof CanvasRenderingContext2D")
        .call1(&JsValue::UNDEFINED, measurer.contexts[0].as_js())?;
    assert_eq!(foreign.as_bool(), Some(false));
    let cache = SourceCanvasCache::new(measurer);
    let options = xterm_core::options::OptionsService::new(
        false,
        [
            ("fontFamily", OptionValue::from("monospace")),
            ("fontSize", OptionValue::from(15.0)),
        ],
    );
    cache.set_font_from_options(&options).unwrap();
    let first = cache.get(&[87], false, false).unwrap();
    assert!(first > 0.0);
    assert_eq!(
        cache.get(&[87], false, false).unwrap(),
        f64::from(first as f32)
    );
    assert!(cache.get(&[87, 105], true, true).unwrap() > 0.0);
    cache.dispose();
    assert_eq!(
        cache.get(&[87], false, false).unwrap(),
        f64::from(first as f32)
    );
    frame.remove();
    Ok(())
}
#[wasm_bindgen_test]
fn raw_utf16_and_bigint_reach_real_canvas_setters_in_variant_order() -> Result<(), JsValue> {
    let measurer = dom_measurer(&document())?;
    let events = Array::new();
    let observe=Function::new_with_args("ctx,events,index","const d=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(ctx),'font');Object.defineProperty(ctx,'font',{configurable:true,get(){return d.get.call(this)},set(v){events.push([index,v]);d.set.call(this,v)}});");
    for (index, context) in measurer.contexts.iter().enumerate() {
        observe.call3(
            &JsValue::UNDEFINED,
            context.as_js(),
            &events,
            &JsValue::from_f64(index as f64),
        )?;
    }
    let size =
        xterm_wasm_support::options::from_js(&js_sys::eval("123456789012345678901234567890n")?)?;
    let cache = SourceCanvasCache::new(measurer);
    cache
        .set_font(RawFont {
            family: OptionValue::String(vec![0xd800, 0, 0x85, 0xfeff]),
            size,
            ..RawFont::default()
        })
        .unwrap();
    assert_eq!(events.length(), 4);
    for (index, prefix) in ["normal", "bold", "italic normal", "italic bold"]
        .into_iter()
        .enumerate()
    {
        let event = Array::from(&events.get(index as u32));
        assert_eq!(event.get(0).as_f64(), Some(index as f64));
        let units = crate::utf16::from_js_string(&JsString::from(event.get(1)));
        let mut expected: Vec<u16> = format!("{prefix} 123456789012345678901234567890px ")
            .encode_utf16()
            .collect();
        expected.extend([0xd800, 0, 0x85]);
        assert_eq!(units, expected);
    }
    Ok(())
}
#[wasm_bindgen_test]
fn canvas_setter_failure_retains_exact_exception_and_committed_font_key() -> Result<(), JsValue> {
    let measurer = dom_measurer(&document())?;
    let sentinel = Object::new();
    let calls = Array::new();
    Function::new_with_args(
        "ctx,error,calls",
        "Object.defineProperty(ctx,'font',{configurable:true,set(v){calls.push(v);throw error;}})",
    )
    .call3(
        &JsValue::UNDEFINED,
        measurer.contexts[0].as_js(),
        &sentinel,
        &calls,
    )?;
    let cache = SourceCanvasCache::new(measurer);
    let key = font();
    let error = cache.set_font(key.clone()).unwrap_err();
    let WidthError::Measure(actual) = error else {
        panic!("expected original setter exception")
    };
    assert_eq!(actual, JsValue::from(sentinel));
    assert_eq!(calls.length(), 1);
    cache.set_font(key).unwrap();
    assert_eq!(calls.length(), 1);
    Ok(())
}
#[wasm_bindgen_test]
fn javascript_font_coercion_can_dispose_cache_without_an_outer_borrow() -> Result<(), JsValue> {
    let cache = SourceCanvasCache::new(dom_measurer(&document())?);
    let weak = cache.downgrade();
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let captured = std::rc::Rc::clone(&calls);
    let primitive = Closure::wrap(Box::new(move |hint: JsValue| -> JsValue {
        assert_eq!(hint.as_string().as_deref(), Some("string"));
        captured.set(captured.get() + 1);
        weak.upgrade().unwrap().dispose();
        JsValue::from_str("Face")
    }) as Box<dyn FnMut(JsValue) -> JsValue>)
    .into_js_value();
    let object = Object::new();
    Reflect::set(&object, &js_sys::Symbol::to_primitive(), &primitive)?;
    let family = xterm_wasm_support::options::from_js(&object)?;
    assert!(matches!(
        cache.set_font(RawFont {
            family,
            size: 15.0.into(),
            ..RawFont::default()
        }),
        Err(WidthError::Disposed)
    ));
    assert_eq!(calls.get(), 1);
    Ok(())
}
