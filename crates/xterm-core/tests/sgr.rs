use std::rc::Rc;
use xterm_core::buffer::{
    constants::{attributes as color, bg_flags as bg, fg_flags as fg},
    AttributeData, ExtendedAttrs,
};
use xterm_core::parser::params::Params;
use xterm_core::terminal::sgr::{apply, apply_with_options};

fn params(body: &str) -> Params {
    let mut result = Params::default();
    result.add_param(0).unwrap();
    for byte in body.bytes() {
        match byte {
            b';' => result.add_param(0).unwrap(),
            b':' => result.add_sub_param(-1).unwrap(),
            b'0'..=b'9' => result.add_digit(byte - b'0').unwrap(),
            _ => panic!("invalid fixture parameter byte"),
        }
    }
    result
}

fn raw_extended(attr: &AttributeData) -> u32 {
    let mut copy = attr.extended.clone();
    copy.set_url_id(0);
    copy.ext()
}

#[test]
fn unchanged_source_assertion_states_and_seeded_corpus() {
    let fixture = include_str!("fixtures/sgr-oracle.tsv");
    assert_eq!(fixture.lines().count(), 13378);
    for (line, row) in fixture.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 14, "line {line}");
        let value = |index: usize| fields[index].parse::<u32>().unwrap();
        let mut attr = AttributeData {
            fg: value(0),
            bg: value(1),
            extended: ExtendedAttrs::new(value(2), u64::from(value(3))),
        };
        if value(4) != 0 {
            attr.extended.set_payload(Some(Rc::new(())));
        }
        let previous = attr.extended.share();
        let unknown = apply_with_options(&mut attr, &params(fields[6]), value(5) != 0);
        let expected = [value(7), value(8), value(9), value(10), value(11)];
        let actual = [
            attr.fg,
            attr.bg,
            raw_extended(&attr),
            u32::try_from(attr.extended.url_id()).unwrap(),
            u32::from(attr.extended.payload().is_some()),
        ];
        assert_eq!(actual, expected, "packed state line {line}: {}", fields[6]);
        assert_eq!(
            attr.extended.same_identity(&previous),
            value(12) != 0,
            "identity line {line}"
        );
        let unknown = unknown
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        assert_eq!(unknown, fields[13], "diagnostics line {line}");
    }
}

#[test]
fn flag_set_reset_and_kitty_controls_preserve_other_bits() {
    let mut attr = AttributeData::default();
    assert!(apply(&mut attr, &params("1;2;3;4;5;7;8;9;53")).is_empty());
    assert_ne!(attr.is_bold(), 0);
    assert_ne!(attr.is_dim(), 0);
    assert_ne!(attr.is_italic(), 0);
    assert_ne!(attr.is_underline(), 0);
    assert_ne!(attr.is_blink(), 0);
    assert_ne!(attr.is_inverse(), 0);
    assert_ne!(attr.is_invisible(), 0);
    assert_ne!(attr.is_strikethrough(), 0);
    assert_ne!(attr.is_overline(), 0);
    assert_eq!(
        apply_with_options(&mut attr, &params("221;222"), false),
        [221, 222]
    );
    assert_ne!(attr.is_bold(), 0);
    assert_ne!(attr.is_dim(), 0);
    apply(&mut attr, &params("221"));
    assert_eq!(attr.is_bold(), 0);
    assert_ne!(attr.is_dim(), 0);
    apply(&mut attr, &params("1;222"));
    assert_ne!(attr.is_bold(), 0);
    assert_eq!(attr.is_dim(), 0);
    apply(&mut attr, &params("22;23;24;25;27;28;29;55"));
    assert_eq!((attr.fg, attr.bg), (0, 0));
}

#[test]
fn palette_and_rgb_transitions_retain_flags_and_protection() {
    let mut attr = AttributeData {
        fg: fg::BOLD,
        bg: bg::PROTECTED,
        ..AttributeData::default()
    };
    for index in 0..256 {
        apply(&mut attr, &params(&format!("38;5;{index};48;5;{index}")));
        assert_eq!(attr.fg, fg::BOLD | color::CM_P256 | index);
        assert_eq!(attr.bg, bg::PROTECTED | color::CM_P256 | index);
    }
    apply(&mut attr, &params("38;2;1;2;3;48;2;4;5;6"));
    assert_eq!(attr.fg, fg::BOLD | color::CM_RGB | 0x010203);
    assert_eq!(attr.bg, bg::PROTECTED | color::CM_RGB | 0x040506);
    apply(&mut attr, &params("97;107"));
    assert_eq!(attr.fg, fg::BOLD | color::CM_P16 | 15);
    assert_eq!(attr.bg, bg::PROTECTED | color::CM_P16 | 15);
    apply(&mut attr, &params("39;49"));
    assert_eq!((attr.fg, attr.bg), (fg::BOLD, bg::PROTECTED));
}

#[test]
fn reset_clones_extended_drops_payload_and_preserves_url_and_variant() {
    let mut attr = AttributeData {
        fg: u32::MAX,
        bg: u32::MAX,
        extended: ExtendedAttrs::new(u32::MAX, 9),
    };
    attr.extended.set_payload(Some(Rc::new(())));
    let previous = attr.extended.share();
    apply(&mut attr, &params("0"));
    assert_eq!((attr.fg, attr.bg), (0, bg::HAS_EXTENDED));
    assert_eq!(attr.extended.url_id(), 9);
    assert_eq!(raw_extended(&attr), 0xe0000000);
    assert!(attr.extended.payload().is_none());
    assert!(!attr.extended.same_identity(&previous));
    assert_eq!(previous.ext(), u32::MAX & !0x1c000000 | (5 << 26));
}

#[test]
fn underline_color_clone_and_packed_flag_quirks_are_preserved() {
    let mut attr = AttributeData::default();
    let previous = attr.extended.share();
    apply(&mut attr, &params("58;2;1;2;3"));
    assert_eq!(attr.extended.underline_color(), color::CM_RGB | 0x010203);
    assert_eq!(attr.has_extended_attrs(), 0);
    assert!(!attr.extended.same_identity(&previous));
    apply(&mut attr, &params("4;59"));
    assert_eq!(attr.get_underline_style(), 1);
    assert_eq!(
        attr.extended.underline_color(),
        color::CM_MASK | color::RGB_MASK
    );
    apply(&mut attr, &params("4:0"));
    assert_eq!(attr.has_extended_attrs(), 0);
    assert_eq!(attr.get_underline_style(), 0);
}
