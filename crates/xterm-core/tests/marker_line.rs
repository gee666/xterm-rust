// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::Cell, error::Error as _, fmt::Write as _, rc::Rc};
use xterm_core::{
    buffer::marker_line::{MarkerLine, MarkerLineError},
    event::EmitError,
    options::{OptionError, OptionObject, OptionValue},
};
fn text(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}
fn hex(units: &[u16]) -> String {
    if units.is_empty() {
        return "_".into();
    }
    let mut value = String::new();
    for unit in units {
        write!(&mut value, "{unit:04x}").unwrap();
    }
    value
}
fn encode(value: &MarkerLine) -> String {
    if let Some(number) = value.as_number() {
        if number == 0.0 && number.is_sign_negative() {
            return "n-0".into();
        }
        format!("n{}", String::from_utf16(&value.to_utf16()).unwrap())
    } else {
        format!("s{}", hex(value.as_string().unwrap()))
    }
}
fn decode(value: &str) -> MarkerLine {
    match value.as_bytes()[0] {
        b'n' => MarkerLine::number(match &value[1..] {
            "NaN" => f64::NAN,
            "Infinity" => f64::INFINITY,
            "-Infinity" => f64::NEG_INFINITY,
            number => number.parse().unwrap(),
        }),
        b's' if &value[1..] == "_" => MarkerLine::string(Vec::new()),
        b's' => MarkerLine::string(
            (1..value.len())
                .step_by(4)
                .map(|i| u16::from_str_radix(&value[i..i + 4], 16).unwrap())
                .collect(),
        ),
        _ => panic!("invalid scalar fixture"),
    }
}
fn snapshot(left: &MarkerLine, right: &MarkerLine) -> String {
    let numeric = MarkerLine::number(left.to_number());
    let added = left.add_value(right.as_value()).unwrap();
    let subtracted = left.subtract_number(right.to_number());
    format!(
        "{}|{}|{}|{}|{}|{}{}{}{}{}{}|{}",
        encode(&numeric),
        hex(&left.to_utf16()),
        encode(&added),
        encode(&subtracted),
        encode(&left.increment()),
        u8::from(left.strict_eq(right)),
        u8::from(left.rel_lt(right)),
        u8::from(left.rel_gt(right)),
        u8::from(left.rel_le(right)),
        u8::from(left.rel_ge(right)),
        u8::from(left.map_key() == right.map_key()),
        encode(&left.map_key().to_line())
    )
}

#[test]
fn every_scalar_trace_matches_javascript_operators() {
    for (index, line) in include_str!("fixtures/marker-line.tsv").lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(
            snapshot(&decode(fields[0]), &decode(fields[1])),
            fields[2],
            "scalar case{index}"
        );
    }
}
#[test]
fn string_addition_subtraction_increment_and_relational_types_are_distinct() {
    let line = MarkerLine::from("01");
    assert_eq!(
        line.add_number(2.0).unwrap().as_string(),
        Some(text("012").as_slice())
    );
    assert_eq!(line.subtract_number(2.0).as_number(), Some(-1.0));
    assert_eq!(line.increment().as_number(), Some(2.0));
    assert!(!line.strict_eq_number(1.0));
    assert!(!MarkerLine::from("2").rel_lt(&MarkerLine::from("10")));
    assert!(MarkerLine::from("2").rel_lt(&MarkerLine::number(10.0)));
    assert!(MarkerLine::string(vec![0xd800]).rel_lt(&MarkerLine::string(vec![0xe000])));
}
#[test]
fn map_keys_use_same_value_zero_but_equality_is_strict() {
    let nan = MarkerLine::number(f64::NAN);
    assert!(!nan.strict_eq(&nan));
    assert!(!nan.rel_lt(&nan));
    assert!(!nan.rel_ge(&nan));
    assert_eq!(nan.map_key(), nan.map_key());
    assert_eq!(
        MarkerLine::number(-0.0).map_key(),
        MarkerLine::number(0.0).map_key()
    );
    assert!(!MarkerLine::number(0.0)
        .map_key()
        .to_line()
        .to_number()
        .is_sign_negative());
    assert_ne!(
        MarkerLine::from("0").map_key(),
        MarkerLine::number(0.0).map_key()
    );
}
#[test]
fn invalid_primitives_are_rejected_without_object_coercion() {
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    let object = OptionValue::Object(OptionObject::new((), move |_| {
        observed.set(observed.get() + 1);
        Ok(1.0.into())
    }));
    for value in [
        OptionValue::Undefined,
        OptionValue::Null,
        OptionValue::Bool(false),
        object,
    ] {
        assert!(matches!(
            MarkerLine::try_from(value),
            Err(MarkerLineError::NotNumberOrString)
        ));
    }
    assert_eq!(calls.get(), 0);
}
#[test]
fn general_addition_keeps_boolean_null_and_undefined_concatenation() {
    let line = MarkerLine::from("0");
    for (value, expected) in [
        (OptionValue::Bool(false), "0false"),
        (OptionValue::Null, "0null"),
        (OptionValue::Undefined, "0undefined"),
    ] {
        assert_eq!(line.add_value(&value).unwrap().to_utf16(), text(expected));
    }
    assert!(MarkerLine::number(0.0)
        .add_value(&OptionValue::Undefined)
        .unwrap()
        .to_number()
        .is_nan());
}
#[test]
fn host_coercion_errors_keep_the_original_object_and_source_chain() {
    let original = EmitError::callback(std::io::Error::other("line coercion sentinel"));
    let retained = original.clone();
    let object = OptionValue::Object(OptionObject::with_fallible_properties(
        (),
        move |_| Err(OptionError::Dispatch(retained.clone())),
        |_, _| Ok(OptionValue::Undefined),
    ));
    let error = MarkerLine::from("0").add_value(&object).unwrap_err();
    let source = error.source().unwrap().source().unwrap().source().unwrap();
    assert_eq!(source.to_string(), "line coercion sentinel");
    assert!(
        matches!(error, MarkerLineError::Coercion(OptionError::Dispatch(actual)) if actual == original)
    );
}
#[test]
fn large_number_increment_matches_source_precision_without_integer_iteration() {
    let start = MarkerLine::number(2_f64.powi(53));
    let end = start.add_number(1.0).unwrap();
    assert!(start.strict_eq(&end));
    assert!(!start.rel_lt(&end));
    assert!(start.strict_eq(&start.increment()));
}
