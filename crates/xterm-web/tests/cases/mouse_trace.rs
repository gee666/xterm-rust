use super::{
    getters::{getter, Getter},
    Setup,
};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::Event;
use xterm_core::mouse_state::{MouseEventTypes, MouseProtocol};

fn methods(setup: &Setup, event: &Event) -> Vec<Getter> {
    [("preventDefault", "prevent"), ("stopPropagation", "stop")]
        .into_iter()
        .map(|(name, label)| {
            let function = js_sys::Reflect::get(event, &name.into())
                .unwrap()
                .dyn_into::<js_sys::Function>()
                .unwrap();
            let target = event.clone();
            let trace = setup.host.trace.clone();
            let callback = Closure::wrap(Box::new(move || {
                trace.borrow_mut().push(label.into());
                function.call0(&target)
            })
                as Box<dyn Fn() -> Result<JsValue, JsValue>>);
            assert!(js_sys::Reflect::set(event, &name.into(), callback.as_ref()).unwrap());
            callback
        })
        .collect()
}

#[wasm_bindgen_test]
fn unchanged_source_live_read_and_effect_traces_match() {
    let fixture = include_str!("../fixtures/mouse-live.tsv");
    assert_eq!(fixture.lines().count(), 10);
    for row in fixture.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 3);
        let name = fields[0];
        assert!([
            "wheel",
            "zero",
            "veto",
            "override",
            "override-number",
            "mutate",
            "shift",
            "pixel",
            "fallback",
            "invalid-size"
        ]
        .contains(&name));
        let setup = Setup::new();
        let restricted = setup.host.trace.clone();
        let events = MouseEventTypes::DOWN
            | MouseEventTypes::UP
            | MouseEventTypes::DRAG
            | MouseEventTypes::MOVE
            | MouseEventTypes::WHEEL;
        setup.host.input.mouse.add_protocol(
            "TRACE",
            MouseProtocol {
                events,
                restrict: Rc::new(move |_| {
                    restricted.borrow_mut().push("restrict".into());
                    true
                }),
            },
        );
        let encoded = setup.host.trace.clone();
        setup.host.input.mouse.add_encoding(
            "TRACE",
            Rc::new(move |_| {
                encoded.borrow_mut().push("encode".into());
                vec![33]
            }),
        );
        setup.host.input.mouse.set_active_protocol("TRACE").unwrap();
        setup.host.input.mouse.set_active_encoding("TRACE").unwrap();
        setup.host.veto.set(name == "veto");
        let mut snapshot = setup.host.snapshot.get();
        snapshot.valid_char_size = name != "invalid-size";
        setup.host.snapshot.set(snapshot);
        let fallback = name == "fallback";
        let event = if fallback {
            setup.pointer("mousemove", 0)
        } else {
            setup.wheel()
        };
        let rect = setup.dom.screen.get_bounding_client_rect();
        let delta = Rc::new(Cell::new(match name {
            "zero" => 0.0,
            "pixel" => 30.0,
            _ => 1.0,
        }));
        if name == "mutate" {
            let mutated = delta.clone();
            *setup.host.wheel_hook.borrow_mut() = Some(Rc::new(move |_| mutated.set(-1.0)));
        }
        let entries = [
            ("clientX", (rect.left() + 4.0).into()),
            ("clientY", (rect.top() + 4.0).into()),
            (
                "type",
                if fallback {
                    "mousemove".into()
                } else {
                    "wheel".into()
                },
            ),
            (
                "overrideType",
                match name {
                    "override" => "unknown".into(),
                    "override-number" => 1.into(),
                    _ => JsValue::UNDEFINED,
                },
            ),
            (
                "buttons",
                if fallback {
                    JsValue::UNDEFINED
                } else {
                    0.into()
                },
            ),
            ("button", 0.into()),
            ("ctrlKey", false.into()),
            ("altKey", false.into()),
            ("shiftKey", (name == "shift").into()),
            (
                "deltaMode",
                if name == "pixel" { 0.into() } else { 1.into() },
            ),
        ];
        let mut getters: Vec<Getter> = entries
            .into_iter()
            .map(|(key, value)| {
                let trace = setup.host.trace.clone();
                getter(&event, key, move || {
                    trace.borrow_mut().push(format!("event:{key}"));
                    Ok(value.clone())
                })
            })
            .collect();
        let trace = setup.host.trace.clone();
        getters.push(getter(&event, "deltaY", move || {
            trace.borrow_mut().push("event:deltaY".into());
            Ok(delta.get().into())
        }));
        let _methods = methods(&setup, &event);
        setup.host.trace.borrow_mut().clear();
        setup.dom.element.dispatch_event(&event).unwrap();
        assert_eq!(setup.host.trace.borrow().join(","), fields[1], "{name}");
        assert_eq!(
            setup.host.input.data.borrow().len(),
            fields[2].parse::<usize>().unwrap(),
            "{name}"
        );
        assert!(setup
            .host
            .input
            .data
            .borrow()
            .iter()
            .all(|(data, user)| data == &[33] && *user));
        drop(getters);
    }
}
