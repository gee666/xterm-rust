#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, JsString, Reflect};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::{
    buffer::{
        constants::{attributes, bg_flags, content, fg_flags},
        AttributeData, CellData, ExtendedAttrs,
    },
    headless::Terminal,
};
use xterm_web::{terminal::buffer_api::WebBufferNamespace, utf16::from_js_string};
wasm_bindgen_test_configure!(run_in_browser);

#[path = "cases/public_buffer_number_callbacks.rs"]
mod callbacks;

fn call(object: &JsValue, name: &str, args: &[JsValue]) -> Result<JsValue, JsValue> {
    let function = Reflect::get(object, &name.into())?.dyn_into::<Function>()?;
    let args: Array = args.iter().cloned().collect();
    function.apply(object, &args)
}
fn line() -> (Rc<Terminal>, JsValue, JsValue) {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    terminal
        .with_state(|state| {
            let buffer = state.active_buffer();
            let line = buffer.borrow().lines.get(0).unwrap();
            for (index, cp, width, fg, bg) in [
                (0, 65, 1, content::IS_COMBINED_MASK | 65, 0),
                (
                    1,
                    content::IS_COMBINED_MASK,
                    1,
                    fg_flags::UNDERLINE | attributes::CM_RGB | 0x112233,
                    bg_flags::HAS_EXTENDED,
                ),
                (2, 0x6587, 2, 0, 0),
                (3, 0, 0, 0, 0),
                (4, 0, 1, 0, 0),
            ] {
                line.borrow_mut().set_cell(
                    index,
                    &CellData {
                        content: cp | (width << content::WIDTH_SHIFT),
                        attributes: AttributeData {
                            fg,
                            bg,
                            extended: ExtendedAttrs::new(
                                (3 << 26) | attributes::CM_RGB | 0xabcdef,
                                17,
                            ),
                        },
                        combined_data: if index == 1 {
                            vec![101, 0x301]
                        } else {
                            Vec::new()
                        },
                    },
                );
            }
        })
        .unwrap();
    let namespace = WebBufferNamespace::new(Rc::clone(&terminal)).unwrap();
    let normal = namespace.normal();
    let line = call(&normal, "getLine", &[0.into()]).unwrap();
    (terminal, normal, line)
}
fn scalar(value: Result<JsValue, JsValue>) -> String {
    match value {
        Err(error) => format!(
            "error:{}",
            Reflect::get(&error, &"name".into())
                .unwrap()
                .as_string()
                .unwrap()
        ),
        Ok(value) if value.is_undefined() => "u".into(),
        Ok(value) if value.as_bool().is_some() => {
            if value.as_bool().unwrap() {
                "t".into()
            } else {
                "f".into()
            }
        }
        Ok(value) => {
            let number = value.as_f64().unwrap();
            if number.is_nan() {
                "nan".into()
            } else {
                number.to_string()
            }
        }
    }
}
fn chars(value: JsValue) -> String {
    if value.is_undefined() {
        return "u".into();
    }
    let text = from_js_string(&value.dyn_into::<JsString>().unwrap());
    if text.is_empty() {
        "-".into()
    } else {
        text.iter()
            .map(|unit| format!("{unit:x}"))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[wasm_bindgen_test]
fn numeric_cell_public_getters_match_2071_original_snapshots_in_javascript() {
    let (_terminal, normal, line) = line();
    let default_cell = call(&normal, "getNullCell", &[]).unwrap();
    let methods = [
        "getWidth",
        "getFgColorMode",
        "getBgColorMode",
        "getFgColor",
        "getBgColor",
        "isBold",
        "isItalic",
        "isDim",
        "isUnderline",
        "isBlink",
        "isInverse",
        "isInvisible",
        "isStrikethrough",
        "isOverline",
        "isFgRGB",
        "isFgPalette",
        "isFgDefault",
        "isBgRGB",
        "isBgPalette",
        "isBgDefault",
        "isAttributeDefault",
        "getUnderlineStyle",
        "getUnderlineColor",
        "getUnderlineColorMode",
        "isUnderlineColorRGB",
        "isUnderlineColorPalette",
        "isUnderlineColorDefault",
    ];
    for row in include_str!("../../xterm-core/tests/fixtures/public-cell-numbers.tsv").lines() {
        let (bits, expected) = row.split_once('\t').unwrap();
        let index = f64::from_bits(u64::from_str_radix(bits, 16).unwrap());
        let cell = call(&line, "getCell", &[index.into()]).unwrap();
        if expected == "none" {
            assert!(cell.is_undefined(), "{bits}");
            continue;
        }
        let fields: Vec<_> = expected.split('|').collect();
        assert_eq!(
            chars(call(&cell, "getChars", &[]).unwrap()),
            fields[4],
            "chars {bits}"
        );
        assert_eq!(
            scalar(call(&cell, "getCode", &[])),
            fields[5],
            "code {bits}"
        );
        for (offset, method) in methods.iter().enumerate() {
            assert_eq!(
                scalar(call(&cell, method, &[])),
                fields[6 + offset],
                "{method} {bits}"
            );
        }
        assert_eq!(
            scalar(call(&cell, "attributesEquals", &[cell.clone()])),
            fields[33],
            "equals {bits}"
        );
        assert_eq!(
            scalar(call(&cell, "attributesEquals", &[default_cell.clone()])),
            fields[34],
            "default equals {bits}"
        );
    }
}

#[wasm_bindgen_test]
fn numeric_translation_crosses_js_boundary_without_usize_coercion() {
    let (_terminal, _normal, line) = line();
    for row in
        include_str!("../../xterm-core/tests/fixtures/public-line-number-translations.tsv").lines()
    {
        let fields: Vec<_> = row.split('\t').collect();
        if fields[3] == "true" {
            continue;
        }
        let number = |value: &str| {
            if value == "u" {
                JsValue::UNDEFINED
            } else {
                f64::from_bits(u64::from_str_radix(value, 16).unwrap()).into()
            }
        };
        let result = call(
            &line,
            "translateToString",
            &[
                JsValue::from_bool(fields[2] == "true"),
                number(fields[0]),
                number(fields[1]),
            ],
        );
        let actual = match result {
            Ok(value) => format!("{}|-", chars(value)),
            Err(error) => scalar(Err(error)),
        };
        assert_eq!(actual, fields[4], "{row}");
    }
}

#[wasm_bindgen_test]
fn shrunk_view_keeps_retained_words_outside_public_cell_reads() {
    let (terminal, _normal, line) = line();
    terminal
        .with_state(|state| {
            let buffer = state.active_buffer();
            let row = buffer.borrow().lines.get(0).unwrap();
            row.borrow_mut().resize(3, &CellData::default());
        })
        .unwrap();
    let cell = call(&line, "getCell", &[(2.0 + 2.0 / 3.0).into()]).unwrap();
    assert!(Reflect::get(&cell, &"fg".into()).unwrap().is_undefined());
    assert!(Reflect::get(&cell, &"bg".into()).unwrap().is_undefined());
    assert_eq!(
        call(&cell, "isAttributeDefault", &[]).unwrap().as_bool(),
        Some(false)
    );
}

#[wasm_bindgen_test]
fn numeric_reads_reuse_managed_targets_and_ignore_falsey_targets() {
    let (_terminal, normal, line) = line();
    let target = call(&normal, "getNullCell", &[]).unwrap();
    let result = call(&line, "getCell", &[f64::NAN.into(), target.clone()]).unwrap();
    assert_eq!(result, target);
    assert_eq!(call(&target, "getWidth", &[]).unwrap().as_f64(), Some(0.0));
    assert_eq!(
        call(&target, "isAttributeDefault", &[]).unwrap().as_bool(),
        Some(false)
    );
    for falsey in [
        JsValue::FALSE,
        0.into(),
        "".into(),
        JsValue::NULL,
        JsValue::UNDEFINED,
        f64::NAN.into(),
    ] {
        assert!(!call(&line, "getCell", &[0.into(), falsey])
            .unwrap()
            .is_undefined());
    }
}
