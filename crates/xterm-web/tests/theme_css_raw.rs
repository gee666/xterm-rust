use serde_json::Value;
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    color::{css, Color},
    options::{CoercionHint, OptionObject, OptionValue, OptionsService},
};
use xterm_web::{
    renderer::theme_css_utf16::theme_css_live,
    theme::{ColorRef, ColorSet, ThemeService},
};

fn units(value: &Value) -> Vec<u16> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|unit| unit.as_u64().unwrap().try_into().unwrap())
        .collect()
}
fn color(value: &Value) -> ColorRef {
    let color = ColorRef::from(Color {
        css: String::new(),
        rgba: value["rgba"].as_u64().unwrap().try_into().unwrap(),
    });
    color.set_css_units(units(&value["css"]));
    color
}
fn populate(colors: &mut ColorSet, value: &Value) {
    colors.foreground = color(&value["foreground"]);
    colors.background = color(&value["background"]);
    colors.cursor = color(&value["cursor"]);
    colors.cursor_accent = color(&value["cursorAccent"]);
    colors.selection_background_opaque = color(&value["selectionBackgroundOpaque"]);
    colors.selection_inactive_background_opaque =
        color(&value["selectionInactiveBackgroundOpaque"]);
    colors.ansi = value["ansi"]
        .as_array()
        .unwrap()
        .iter()
        .map(color)
        .collect();
}
fn fixtures() -> Value {
    serde_json::from_str(include_str!("fixtures/theme-css-raw.json")).unwrap()
}
fn native_color(css: &str, rgba: u32) -> ColorRef {
    Color {
        css: css.into(),
        rgba,
    }
    .into()
}

#[test]
fn every_raw_stylesheet_unit_matches_the_unchanged_source() {
    let data = fixtures();
    let cases = data["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 30);
    for case in cases {
        let theme = ThemeService::default();
        let colors = theme.colors();
        populate(&mut colors.borrow_mut(), &case["colors"]);
        let options = OptionsService::new(false, [] as [(&str, OptionValue); 0]);
        for (key, value) in case["options"].as_object().unwrap() {
            options
                .set_raw(key, OptionValue::String(units(value)))
                .unwrap();
        }
        let actual = theme_css_live(case["id"].as_u64().unwrap(), &colors, &options).unwrap();
        let expected = units(&case["expected"]);
        assert_eq!(actual, expected, "{} raw UTF-16", case["name"]);
    }
}

#[test]
fn live_callbacks_match_source_hint_order_and_full_text_after_mutations() {
    let data = fixtures();
    let theme = ThemeService::default();
    let colors = theme.colors();
    {
        let mut c = colors.borrow_mut();
        c.foreground = native_color("#ffffff", u32::MAX);
        c.background = native_color("#ffffff", u32::MAX);
        c.cursor = native_color("#ffffff", u32::MAX);
        c.cursor_accent = native_color("#ffffff", u32::MAX);
        c.selection_background_opaque = native_color("#ffffff", u32::MAX);
        c.selection_inactive_background_opaque = native_color("#ffffff", u32::MAX);
        c.ansi.clear();
    }
    let options = Rc::new(OptionsService::new(false, [] as [(&str, OptionValue); 0]));
    let hints = Rc::new(RefCell::new(Vec::<(String, CoercionHint)>::new()));
    let weak_options = Rc::downgrade(&options);
    let changed = colors.clone();
    let observed = hints.clone();
    let family = OptionObject::with_properties(
        (),
        move |hint| {
            observed.borrow_mut().push(("family".into(), hint));
            changed.borrow_mut().foreground = native_color("after-family", 0x00ff00ff);
            let changed = changed.clone();
            let observed = observed.clone();
            let size = OptionObject::with_properties(
                (),
                move |hint| {
                    observed.borrow_mut().push(("size".into(), hint));
                    changed.borrow_mut().foreground = native_color("after-size", 0x0000ffff);
                    Ok(OptionValue::from("15e0"))
                },
                |_, _| Ok(OptionValue::Undefined),
            );
            weak_options
                .upgrade()
                .unwrap()
                .set_raw("fontSize", OptionValue::Object(size))
                .unwrap();
            Ok(OptionValue::String(vec![97, 0xd800]))
        },
        |_, _| Ok(OptionValue::Undefined),
    );
    options
        .set_raw("fontFamily", OptionValue::Object(family))
        .unwrap();
    for (key, label, result, action) in [
        ("fontWeight", "weight", "normal", 0),
        ("fontWeightBold", "weightBold", "bold", 1),
        ("cursorWidth", "cursorWidth", "2.5", 2),
    ] {
        let changed = colors.clone();
        let observed = hints.clone();
        let object = OptionObject::with_properties(
            (),
            move |hint| {
                observed.borrow_mut().push((label.into(), hint));
                match action {
                    0 => {
                        changed.borrow_mut().background =
                            native_color("ignored direct BG", 0x11223344)
                    }
                    1 => changed.borrow_mut().cursor = native_color("before-width", u32::MAX),
                    2 => changed.borrow_mut().cursor = native_color("after-width", u32::MAX),
                    _ => unreachable!(),
                }
                Ok(OptionValue::from(result))
            },
            |_, _| Ok(OptionValue::Undefined),
        );
        options.set_raw(key, OptionValue::Object(object)).unwrap();
    }
    let actual = theme_css_live(99, &colors, &options).unwrap();
    assert_eq!(actual, units(&data["live"]["expected"]));
    let expected: Vec<_> = data["live"]["hints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|pair| (pair[0].as_str().unwrap().to_owned(), CoercionHint::String))
        .collect();
    assert_eq!(*hints.borrow(), expected);
}

#[test]
fn fast_color_css_and_rgba_match_source_without_replacing_lone_surrogates() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/color-utf16-fast.json")).unwrap();
    assert_eq!(cases.len(), 78);
    for (index, case) in cases.iter().enumerate() {
        let actual = css::to_color_utf16(&units(&case["input"]));
        if case["unsupported"].as_bool() == Some(true) {
            assert!(actual.is_err(), "case {index}");
        } else {
            let actual = actual.unwrap();
            assert_eq!(actual.css, units(&case["css"]), "case {index} CSS units");
            assert_eq!(
                u64::from(actual.rgba),
                case["rgba"].as_u64().unwrap(),
                "case {index} RGBA"
            );
        }
    }
}
