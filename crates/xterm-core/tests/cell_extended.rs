use std::rc::Rc;
use xterm_core::buffer::{
    constants::{attributes as color, bg_flags, ext_flags, underline_style as style},
    AttributeData, ExtendedAttrs,
};

#[test]
fn setter_masks_preserve_all_other_fields() {
    for style in 0..16 {
        for offset in 0..16 {
            for mode in [
                color::CM_DEFAULT,
                color::CM_P16,
                color::CM_P256,
                color::CM_RGB,
            ] {
                let mut a = ExtendedAttrs::new(u32::MAX, 0);
                a.set_underline_style(style);
                assert_eq!(
                    a.ext() & !ext_flags::UNDERLINE_STYLE,
                    u32::MAX & !ext_flags::UNDERLINE_STYLE
                );
                assert_eq!(a.underline_style(), style & 7);
                a.set_underline_variant_offset(offset);
                assert_eq!(a.underline_style(), style & 7);
                assert_eq!(a.underline_variant_offset(), offset & 7);
                a.set_underline_color(mode | 0x123456);
                assert_eq!(a.underline_color(), mode | 0x123456);
                assert_eq!(a.underline_style(), style & 7);
                assert_eq!(a.underline_variant_offset(), offset & 7);
            }
        }
    }
}

#[test]
fn emptiness_update_and_clone_preserve_unrelated_data() {
    let mut a = AttributeData {
        bg: u32::MAX,
        ..AttributeData::default()
    };
    a.extended.set_underline_color(color::CM_RGB | 0xffffff);
    a.extended.set_underline_variant_offset(7);
    assert!(a.extended.is_empty());
    a.update_extended();
    assert_eq!(a.bg, u32::MAX & !bg_flags::HAS_EXTENDED);
    a.extended.set_underline_style(1);
    a.update_extended();
    assert_eq!(a.bg, u32::MAX);
    a.extended.set_underline_style(0);
    a.extended.set_url_id(1);
    a.update_extended();
    assert_eq!(a.bg, u32::MAX);
    a.extended.set_url_id(0);
    a.extended.set_payload(Some(Rc::new("payload")));
    a.update_extended();
    assert_eq!(a.bg, u32::MAX);
    a.fg = 42;
    let b = a.clone();
    assert_eq!(b.fg, 42);
    assert_eq!(b.bg, u32::MAX);
    assert!(b.extended.is_empty());
    assert!(b.extended.payload().is_none());
    assert!(!a.extended.same_identity(&b.extended));
    assert_eq!(a.extended.ext(), b.extended.ext());
}

#[test]
fn clone_keeps_hidden_style_when_url_is_present_and_cell_clone_drops_payload() {
    let mut a = xterm_core::buffer::CellData::default();
    a.extended.set_underline_style(style::CURLY);
    a.extended.set_url_id(17);
    a.extended.set_payload(Some(Rc::new(42)));
    a.update_extended();
    let mut b = a.clone();
    assert_eq!(b.extended.url_id(), 17);
    assert_eq!(b.extended.underline_style(), style::DASHED);
    assert!(!a.extended.same_identity(&b.extended));
    assert!(b.extended.payload().is_none());
    assert_eq!(a.fg, b.fg);
    assert_eq!(a.bg, b.bg);
    b.extended.set_url_id(0);
    assert_eq!(b.extended.underline_style(), style::CURLY);
    assert_eq!(a.extended.url_id(), 17);
}

#[test]
fn seeded_extended_mutations_match_js_identity_and_payload_behavior() {
    let fixture = include_str!("fixtures/cell-extended.tsv");
    assert_eq!(fixture.lines().count(), 4096);
    let mut objects = std::array::from_fn::<_, 3, _>(|_| ExtendedAttrs::default());
    for (index, line) in fixture.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        let target: usize = fields[0].parse().unwrap();
        let operation: u32 = fields[1].parse().unwrap();
        let value: u32 = fields[2].parse().unwrap();
        match operation {
            0 => objects[target].set_ext(value),
            1 => objects[target].set_underline_style(value),
            2 => objects[target].set_underline_color(value),
            3 => objects[target].set_underline_variant_offset(value),
            4 => objects[target].set_url_id(if value % 3 == 0 { 0 } else { value }),
            5 => objects[target].set_payload(if value % 2 == 0 {
                None
            } else {
                Some(Rc::new(value))
            }),
            6 => objects[target] = objects[(value % 3) as usize].share(),
            7 => objects[target] = objects[(value % 3) as usize].clone(),
            _ => panic!("unexpected fixture operation"),
        }
        let mut actual = Vec::new();
        for a in &objects {
            actual.extend([
                a.ext(),
                a.underline_style(),
                a.underline_color(),
                a.underline_variant_offset(),
                a.url_id(),
                u32::from(a.is_empty()),
                u32::from(a.payload().is_some()),
            ]);
        }
        actual.extend([
            u32::from(objects[0].same_identity(&objects[1])),
            u32::from(objects[0].same_identity(&objects[2])),
            u32::from(objects[1].same_identity(&objects[2])),
        ]);
        let expected: Vec<u32> = fields[3].split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(actual, expected, "extended operation {index}");
    }
}
