use super::*;
use xterm_core::options::OptionValue;

#[wasm_bindgen_test]
fn update_getter_errors_preserve_identity_and_earlier_listener_mutation() {
    let setup = Setup::new(&JsValue::UNDEFINED);
    setup.dimensions();
    let fixture = Function::new_no_args(
        "const log=[], sentinel={};return {log,sentinel,options:{
         get handleMouseWheel(){log.push('handle');return false;},
         get mouseWheelScrollSensitivity(){log.push('sensitivity');throw sentinel;}}};",
    )
    .call0(&JsValue::NULL)
    .unwrap();
    let raw = Reflect::get(&fixture, &"options".into()).unwrap();
    let sentinel = Reflect::get(&fixture, &"sentinel".into()).unwrap();
    assert_eq!(setup.element.update_options(&raw).unwrap_err(), sentinel);
    assert_eq!(
        js_sys::JSON::stringify(&Reflect::get(&fixture, &"log".into()).unwrap())
            .unwrap()
            .as_string()
            .unwrap(),
        "[\"handle\",\"handle\",\"sensitivity\"]"
    );
    assert_eq!(
        setup.element.options().borrow().handle_mouse_wheel,
        false.into()
    );
    let event = wheel();
    setup
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert!(!event.default_prevented());
    assert_eq!(setup.element.get_scroll_position().top, 0.0);
}

#[wasm_bindgen_test]
fn owned_model_mutates_options_first_and_uses_integer_positions() {
    let window = web_sys::window().unwrap();
    let screen: HtmlElement = window
        .document()
        .unwrap()
        .create_element("div")
        .unwrap()
        .unchecked_into();
    let environment = environment(&window);
    let queue = WindowAnimationQueues::default().for_window(&window);
    let options = Function::new_no_args(
        "const log=[];const options={get lazyRender(){log.push('lazy');return false;},
         set mouseWheelSmoothScroll(value){log.push(value);}};return {options,log};",
    )
    .call0(&JsValue::NULL)
    .unwrap();
    let raw = Reflect::get(&options, &"options".into()).unwrap();
    let element = SmoothScrollableElement::new_owned(screen, raw, environment, queue).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&Reflect::get(&options, &"log".into()).unwrap())
            .unwrap()
            .as_string()
            .unwrap(),
        "[false,\"lazy\",\"lazy\"]"
    );
    assert_eq!(
        element.options().borrow().mouse_wheel_smooth_scroll,
        true.into(),
        "setter-only property reads undefined, so source resolution retains its default"
    );
    element
        .set_scroll_dimensions(DimensionUpdate {
            height: Some(40.0),
            scroll_height: Some(400.0),
            ..Default::default()
        })
        .unwrap();
    element
        .set_scroll_position(
            PositionUpdate {
                top: Some(5.9),
                left: None,
            },
            false,
        )
        .unwrap();
    assert_eq!(element.get_scroll_position().top, 5.0);
    element.dispose().unwrap();
}

#[wasm_bindgen_test]
fn owned_model_rejects_frozen_options_before_later_getters() {
    let window = web_sys::window().unwrap();
    let screen: HtmlElement = window
        .document()
        .unwrap()
        .create_element("div")
        .unwrap()
        .unchecked_into();
    let environment = environment(&window);
    let queue = WindowAnimationQueues::default().for_window(&window);
    let fixture = Function::new_no_args(
        "const log=[];return{log,options:Object.freeze({get lazyRender(){log.push('lazy');return false;}})};",
    ).call0(&JsValue::NULL).unwrap();
    let raw = Reflect::get(&fixture, &"options".into()).unwrap();
    let result = SmoothScrollableElement::new_owned(screen, raw, environment, queue);
    assert!(result.err().unwrap().is_instance_of::<js_sys::TypeError>());
    assert_eq!(
        js_sys::Array::from(&Reflect::get(&fixture, &"log".into()).unwrap()).length(),
        0
    );
}

#[wasm_bindgen_test]
fn nested_wheels_stop_only_after_actual_scroll_or_explicit_consumption() {
    let outer = Setup::new(&JsValue::UNDEFINED);
    let inner = Setup::new(&JsValue::UNDEFINED);
    outer.dimensions();
    inner.dimensions();
    outer
        .element
        .get_dom_node()
        .unwrap()
        .append_child(&inner.parent)
        .unwrap();
    let event = wheel();
    inner
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert!(event.default_prevented());
    assert_eq!(inner.element.get_scroll_position().top, 50.0);
    assert_eq!(outer.element.get_scroll_position().top, 0.0);
    inner
        .element
        .set_scroll_position(
            PositionUpdate {
                top: Some(360.0),
                left: None,
            },
            false,
        )
        .unwrap();
    let event = wheel();
    inner
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert!(event.default_prevented());
    assert_eq!(inner.element.get_scroll_position().top, 360.0);
    assert_eq!(outer.element.get_scroll_position().top, 50.0);
}

#[wasm_bindgen_test]
fn smooth_wheels_accumulate_from_future_position_and_disposal_detaches() {
    let setup = Setup::new(&JsValue::UNDEFINED);
    setup.dimensions();
    setup.model.set_smooth_scroll_duration(100.0);
    for expected in [50.0, 100.0] {
        let event = wheel();
        setup
            .element
            .get_dom_node()
            .unwrap()
            .dispatch_event(&event)
            .unwrap();
        assert!(event.default_prevented());
        assert_eq!(setup.model.future_scroll_position().top, expected);
        assert!(setup.model.has_pending_scroll_animation());
    }
    setup.element.dispose().unwrap();
    let event = wheel();
    setup
        .element
        .get_dom_node()
        .unwrap()
        .dispatch_event(&event)
        .unwrap();
    assert!(!event.default_prevented());
    assert_eq!(setup.model.future_scroll_position().top, 100.0);
    setup.model.dispose().unwrap();
}

#[wasm_bindgen_test]
fn update_ignores_creation_only_fields_and_undefined_preserves_values() {
    let setup = Setup::new(&options(&[
        ("mouseWheelScrollSensitivity", 2.into()),
        ("useShadows", false.into()),
    ]));
    setup
        .element
        .update_options(&options(&[
            ("mouseWheelScrollSensitivity", JsValue::UNDEFINED),
            ("flipAxes", true.into()),
            ("useShadows", true.into()),
            ("lazyRender", true.into()),
            ("className", "ignored".into()),
            ("mouseWheelSmoothScroll", false.into()),
        ]))
        .unwrap();
    let options = setup.element.options();
    let options = options.borrow();
    assert_eq!(options.mouse_wheel_scroll_sensitivity, 2.0.into());
    assert_eq!(options.flip_axes, false.into());
    assert_eq!(options.use_shadows, false.into());
    assert_eq!(options.lazy_render, false.into());
    assert_eq!(options.class_name, OptionValue::from(""));
    assert_eq!(options.mouse_wheel_smooth_scroll, true.into());
}
