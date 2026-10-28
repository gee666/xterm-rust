// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, Buffer, BufferError},
    options::{CoercionError, OptionObject, OptionValue as V, OptionsService},
};

fn prepared(pty: V) -> Buffer {
    let options = Rc::new(OptionsService::default());
    options.set_raw("windowsPty", pty).unwrap();
    let mut buffer = Buffer::new(true, 5, 3, options).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    for (col, cp) in "abcde".chars().enumerate() {
        buffer
            .lines
            .get(0)
            .unwrap()
            .borrow_mut()
            .set_cell_from_codepoint(col, u32::from(cp), 1, &AttributeData::default());
    }
    buffer.y = 1;
    buffer
}
fn pty(backend: V, build: V) -> V {
    V::property_map([("backend", backend), ("buildNumber", build)])
}

#[test]
fn raw_build_truthiness_precedes_comparison_coercion() {
    for (build, enabled) in [
        (V::Number(0.0), true),
        (V::from("0"), false),
        (V::Number(21375.0), false),
        (V::from("21376"), true),
        (V::Null, true),
        (V::Undefined, true),
    ] {
        let mut buffer = prepared(pty(V::from("conpty"), build));
        buffer.resize(1, 3).unwrap();
        let second = buffer
            .lines
            .get(1)
            .unwrap()
            .borrow()
            .translate_to_string(true, None, None, None);
        assert_eq!(
            second,
            if enabled {
                vec![u16::from(b'b')]
            } else {
                Vec::new()
            }
        );
    }
}
#[test]
fn reflow_preserves_getter_order_and_repeats_the_build_getter() {
    let reads = Rc::new(RefCell::new(Vec::<String>::new()));
    let object = OptionObject::with_properties(
        reads.clone(),
        |_| Ok(V::from("[object Object]")),
        |reads, key| {
            reads.borrow_mut().push(key.to_owned());
            Ok(match key {
                "backend" => V::from("conpty"),
                "buildNumber" => V::from("21376"),
                _ => V::Undefined,
            })
        },
    );
    let mut buffer = prepared(V::Object(object));
    buffer.resize(1, 3).unwrap();
    assert_eq!(*reads.borrow(), ["buildNumber", "backend", "buildNumber"]);
}
#[test]
fn alternate_reflow_does_not_read_backend_or_coerce_truthy_build() {
    let reads = Rc::new(RefCell::new(Vec::<String>::new()));
    let object = OptionObject::with_properties(
        reads.clone(),
        |_| Ok(V::from("[object Object]")),
        |reads, key| {
            reads.borrow_mut().push(key.to_owned());
            if key == "buildNumber" {
                Ok(V::from("not numeric"))
            } else {
                Err(CoercionError::new("backend must not be accessed"))
            }
        },
    );
    let options = Rc::new(OptionsService::default());
    options.set_raw("windowsPty", V::Object(object)).unwrap();
    let mut buffer = Buffer::new(false, 5, 3, options).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    buffer.resize(2, 3).unwrap();
    assert_eq!(*reads.borrow(), ["buildNumber"]);
}
#[test]
fn row_growth_checks_presence_and_short_circuits_build_lookup_each_row() {
    let reads = Rc::new(RefCell::new(Vec::<String>::new()));
    let object = OptionObject::with_properties(
        reads.clone(),
        |_| Ok(V::from("[object Object]")),
        |reads, key| {
            reads.borrow_mut().push(key.to_owned());
            Ok(match key {
                "backend" => V::Null,
                "buildNumber" => V::Undefined,
                _ => V::Undefined,
            })
        },
    );
    let mut buffer = prepared(V::Object(object));
    buffer
        .lines
        .push(buffer.get_blank_line(&AttributeData::default(), false))
        .unwrap();
    buffer.ybase = 1;
    buffer.ydisp = 1;
    buffer.y = 2;
    buffer.resize(5, 4).unwrap();
    assert_eq!(buffer.ybase, 1);
    assert_eq!(buffer.lines.len(), 5);
    assert_eq!(*reads.borrow(), ["backend", "buildNumber"]);
}
#[test]
fn getter_failures_propagate_without_text_or_default_substitution() {
    let object = OptionObject::with_properties(
        (),
        |_| Ok(V::from("[object Object]")),
        |_, _| Err(CoercionError::new("host getter failed")),
    );
    let mut buffer = prepared(V::Object(object));
    let error = buffer.resize(1, 3).unwrap_err();
    assert!(matches!(error, BufferError::Options(_)));
    assert_eq!(error.to_string(), "host getter failed");
}
