use super::*;
use serde_json::{json, Value};
use xterm_web::scroll::{mouse_event::StandardMouseEvent, wheel_event::StandardWheelEvent};

fn json_value(raw: &JsValue) -> Value {
    serde_json::from_str(&js_sys::JSON::stringify(raw).unwrap().as_string().unwrap()).unwrap()
}
fn numeric_equal(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.as_f64(), b.as_f64()),
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len());
            for (key, value) in a {
                numeric_equal(value, &b[key]);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}

#[wasm_bindgen_test]
fn source_wheel_and_mouse_constructor_traces_match() {
    let reference: Value = serde_json::from_str(include_str!(
        "../../../../results/scroll-element/mouse-source.json"
    ))
    .unwrap();
    let window = web_sys::window().unwrap();
    let environment = environment(&window);
    let browser = if environment.platform.is_firefox {
        "firefox"
    } else if environment.platform.is_chrome {
        "chromium"
    } else {
        "webkit"
    };
    let expected = reference["browsers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == browser)
        .unwrap();
    let make_wheel = Function::new_with_args("input", "return {...input,view:window};");
    let wheels = reference["wheelCases"].as_array().unwrap();
    assert_eq!(wheels.len(), 13);
    for (index, input) in wheels.iter().enumerate() {
        let raw = js_sys::JSON::parse(&input.to_string()).unwrap();
        let event: Event = make_wheel
            .call1(&JsValue::NULL, &raw)
            .unwrap()
            .unchecked_into();
        let result = StandardWheelEvent::new(Some(&event), 0.0, 0.0, &environment).unwrap();
        let record = Object::new();
        for (key, value) in [
            ("deltaX", result.delta_x),
            ("deltaY", result.delta_y),
            ("target", result.target),
        ] {
            Reflect::set(&record, &key.into(), &value).unwrap();
        }
        numeric_equal(&json_value(record.as_ref()), &expected["wheels"][index]);
    }
    let make_mouse=Function::new_with_args("input",
        "return {button:0,buttons:1,detail:1,type:'mousedown',pageX:10,pageY:20,ctrlKey:false,shiftKey:false,altKey:false,metaKey:false,...input,target:document.body,view:window};");
    let mice = reference["mouseCases"].as_array().unwrap();
    assert_eq!(mice.len(), 8);
    for (index, input) in mice.iter().enumerate() {
        let raw = js_sys::JSON::parse(&input.to_string()).unwrap();
        let event: Event = make_mouse
            .call1(&JsValue::NULL, &raw)
            .unwrap()
            .unchecked_into();
        let result = StandardMouseEvent::new(&window, &event, &environment.iframe).unwrap();
        let record = json!({"left":result.left_button,"middle":result.middle_button,"right":result.right_button,
            "buttons":json_value(&result.buttons),"detail":json_value(&result.detail),"x":result.posx,"y":result.posy,
            "ctrl":json_value(&result.ctrl_key),"shift":json_value(&result.shift_key),"alt":json_value(&result.alt_key),"meta":json_value(&result.meta_key)});
        numeric_equal(&record, &expected["mice"][index]);
        assert!(result.timestamp.as_f64().unwrap().is_finite());
        assert_eq!(
            result.target,
            JsValue::from(window.document().unwrap().body().unwrap())
        );
    }
}
