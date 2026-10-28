use super::super::width_cache::{WidthCache, WidthMeasurer};
use super::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::options::{CoercionHint, OptionObject};
struct RawMeasure(Rc<RefCell<Vec<Vec<u16>>>>);
impl WidthMeasurer for RawMeasure {
    type Font = RawFont;
    type Error = OptionError;
    fn configure_variant(&self, font: &RawFont, variant: FontVariant) -> Result<(), OptionError> {
        self.0.borrow_mut().push(font.variant_units(variant)?);
        Ok(())
    }
    fn measure(&self, _: &[u16], _: FontVariant) -> Result<f64, OptionError> {
        Ok(5.0)
    }
}
fn value(label: &'static str, text: &'static str, log: &Rc<RefCell<Vec<String>>>) -> OptionValue {
    let log = Rc::clone(log);
    OptionValue::Object(OptionObject::new((), move |hint| {
        assert_eq!(hint, CoercionHint::String);
        log.borrow_mut().push(label.to_owned());
        Ok(text.into())
    }))
}
#[test]
fn captured_arguments_convert_in_source_order_for_each_variant() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let family = value("family", "Face", &log);
    let size = value("size", "15", &log);
    let weight = value("weight", "normal", &log);
    let bold = value("bold", "bold", &log);
    let raw = RawFont::read_with(|key| {
        log.borrow_mut().push(format!("read:{key}"));
        Ok(match key {
            "fontFamily" => family.clone(),
            "fontSize" => size.clone(),
            "fontWeight" => weight.clone(),
            _ => bold.clone(),
        })
    })
    .unwrap();
    assert_eq!(
        *log.borrow(),
        [
            "read:fontFamily",
            "read:fontSize",
            "read:fontWeight",
            "read:fontWeightBold"
        ]
    );
    let output = Rc::new(RefCell::new(Vec::new()));
    let cache = WidthCache::new(RawMeasure(Rc::clone(&output)));
    cache.set_font(raw).unwrap();
    assert_eq!(
        &log.borrow()[4..],
        [
            "weight", "size", "family", "bold", "size", "family", "weight", "size", "family",
            "bold", "size", "family"
        ]
    );
    let expected: Vec<Vec<u16>> = [
        "normal 15px Face",
        "bold 15px Face",
        "italic normal 15px Face",
        "italic bold 15px Face",
    ]
    .into_iter()
    .map(|s| s.encode_utf16().collect())
    .collect();
    assert_eq!(*output.borrow(), expected);
}
#[test]
fn object_identity_is_the_key_not_its_later_string_value() {
    let calls = Rc::new(Cell::new(0));
    let current = Rc::new(RefCell::new("Face".to_owned()));
    let c = Rc::clone(&calls);
    let v = Rc::clone(&current);
    let family = OptionValue::Object(OptionObject::new((), move |_| {
        c.set(c.get() + 1);
        Ok(v.borrow().as_str().into())
    }));
    let raw = RawFont {
        family: family.clone(),
        size: 15.0.into(),
        ..RawFont::default()
    };
    let output = Rc::new(RefCell::new(Vec::new()));
    let cache = WidthCache::new(RawMeasure(output));
    cache.set_font(raw.clone()).unwrap();
    assert_eq!(calls.get(), 4);
    *current.borrow_mut() = "Changed".into();
    cache.set_font(raw).unwrap();
    assert_eq!(calls.get(), 4);
    let c = Rc::clone(&calls);
    let second = OptionValue::Object(OptionObject::new((), move |_| {
        c.set(c.get() + 1);
        Ok("Changed".into())
    }));
    cache
        .set_font(RawFont {
            family: second,
            size: 15.0.into(),
            ..RawFont::default()
        })
        .unwrap();
    assert_eq!(calls.get(), 8);
}
#[test]
fn exact_utf16_and_ecma_trim_preserve_nel_surrogate_and_nul() {
    let raw = RawFont {
        family: OptionValue::String(vec![0xd800, 0, 0x85, 0xfeff]),
        size: (-0.0).into(),
        weight: OptionValue::String(vec![0xfeff, 110, 111, 114, 109, 97, 108]),
        ..RawFont::default()
    };
    let mut expected: Vec<u16> = "normal 0px ".encode_utf16().collect();
    expected.extend([0xd800, 0, 0x85]);
    assert_eq!(raw.variant_units(FontVariant::Regular).unwrap(), expected);
}
#[test]
fn initial_defaults_skip_and_nan_reconfigures_while_signed_zero_does_not() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let cache = WidthCache::new(RawMeasure(Rc::clone(&output)));
    cache.set_font(RawFont::default()).unwrap();
    cache
        .set_font(RawFont {
            size: (-0.0).into(),
            ..RawFont::default()
        })
        .unwrap();
    assert!(output.borrow().is_empty());
    for _ in 0..2 {
        cache
            .set_font(RawFont {
                size: f64::NAN.into(),
                ..RawFont::default()
            })
            .unwrap();
    }
    assert_eq!(output.borrow().len(), 8);
}
#[test]
fn text_metrics_converts_size_before_reading_family_and_does_not_trim() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let family = Rc::new(RefCell::new("old".to_owned()));
    let target = Rc::clone(&family);
    let trace = Rc::clone(&log);
    let size = OptionValue::Object(OptionObject::new((), move |hint| {
        assert_eq!(hint, CoercionHint::String);
        trace.borrow_mut().push("size:string".to_owned());
        *target.borrow_mut() = "new".into();
        Ok("15".into())
    }));
    let units = text_metrics_font(|key| {
        log.borrow_mut().push(format!("read:{key}"));
        Ok(if key == "fontSize" {
            size.clone()
        } else {
            family.borrow().as_str().into()
        })
    })
    .unwrap();
    assert_eq!(
        *log.borrow(),
        ["read:fontSize", "size:string", "read:fontFamily"]
    );
    assert_eq!(String::from_utf16(&units).unwrap(), "15px new");
}
