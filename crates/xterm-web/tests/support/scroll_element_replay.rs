// Browser source traces retain the original inputs, per-engine outputs and hash.
use super::*;
use serde_json::{json, Value};

fn js(value: &Value) -> JsValue {
    js_sys::JSON::parse(&value.to_string()).unwrap()
}
fn snapshot(setup: &Setup, prevented: Option<bool>) -> Value {
    let node = setup.element.get_dom_node().unwrap();
    let shadows = node.query_selector_all(".xterm-shadow").unwrap();
    let shadows: Vec<_> = (0..shadows.length())
        .map(|index| {
            shadows
                .item(index)
                .unwrap()
                .unchecked_into::<HtmlElement>()
                .class_name()
        })
        .collect();
    let position = setup.element.get_scroll_position();
    json!({"top":position.top,"left":position.left,"prevented":prevented,
        "className":node.class_name(),"shadows":shadows})
}

#[wasm_bindgen_test]
fn unchanged_source_scenarios_match_rust_element() {
    let reference: Value = serde_json::from_str(include_str!(
        "../../../../results/scroll-element/source.json"
    ))
    .unwrap();
    let agent = web_sys::window().unwrap().navigator().user_agent().unwrap();
    let browser = if agent.contains("Firefox/") {
        "firefox"
    } else if agent.contains("Chrome/") {
        "chromium"
    } else {
        "webkit"
    };
    let expected = reference["browsers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == browser)
        .unwrap();
    let cases = reference["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 38);
    let construct_wheel = Function::new_with_args(
        "values",
        "const {modern,legacyY,legacyX,prevented:pre,...init}=values;
         const e=new WheelEvent('wheel',{view:window,deltaY:40,deltaMode:0,bubbles:true,cancelable:true,...init});
         if(modern)Object.defineProperties(e,{wheelDeltaY:{value:undefined},wheelDeltaX:{value:undefined},wheelDelta:{value:undefined}});
         if(legacyY!==undefined)Object.defineProperty(e,'wheelDeltaY',{value:legacyY});
         if(legacyX!==undefined)Object.defineProperty(e,'wheelDeltaX',{value:legacyX});
         if(pre)e.preventDefault();return e;",
    );
    for (index, case) in cases.iter().enumerate() {
        let raw_options = js(&case["options"]);
        let external = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .create_element("div")
            .unwrap();
        if case["external"] == true {
            Reflect::set(&raw_options, &"listenOnDomNode".into(), &external).unwrap();
        }
        let setup = Setup::new(&raw_options);
        setup.parent.append_child(&external).unwrap();
        setup
            .element
            .set_scroll_dimensions(DimensionUpdate {
                width: Some(100.0),
                height: Some(40.0),
                scroll_width: Some(case["scrollWidth"].as_f64().unwrap_or(400.0)),
                scroll_height: Some(case["scrollHeight"].as_f64().unwrap_or(400.0)),
            })
            .unwrap();
        setup
            .element
            .set_scroll_position(
                PositionUpdate {
                    top: Some(case["top"].as_f64().unwrap_or(0.0)),
                    left: None,
                },
                false,
            )
            .unwrap();
        for (step, action) in case["actions"].as_array().unwrap().iter().enumerate() {
            let mut prevented = None;
            match action["kind"].as_str().unwrap() {
                "wheel" => {
                    let event: Event = construct_wheel
                        .call1(&JsValue::NULL, &js(&action["values"]))
                        .unwrap()
                        .unchecked_into();
                    if case["external"] == true {
                        external.dispatch_event(&event).unwrap();
                    } else {
                        setup
                            .element
                            .get_dom_node()
                            .unwrap()
                            .dispatch_event(&event)
                            .unwrap();
                    }
                    prevented = Some(event.default_prevented());
                }
                "update" => setup
                    .element
                    .update_options(&js(&action["values"]))
                    .unwrap(),
                "class" => setup
                    .element
                    .update_class_name(action["value"].as_str().unwrap().into())
                    .unwrap(),
                "render" => setup.element.render_now().unwrap(),
                "dispose" => setup.element.dispose().unwrap(),
                other => panic!("unknown retained action {other}"),
            }
            let mut expected_step = expected["results"][index]["steps"][step].clone();
            // JSON has no integer/float distinction; serde_json equality does.
            for key in ["top", "left"] {
                expected_step[key] = json!(expected_step[key].as_f64().unwrap());
            }
            assert_eq!(
                snapshot(&setup, prevented),
                expected_step,
                "{} step {step} in {browser}",
                case["name"]
            );
        }
    }
}
