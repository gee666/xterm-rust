use std::rc::Rc;
use xterm_core::buffer::{
    constants::{bg_flags, content, underline_style},
    AttributeData, CellData, CharData, ExtendedAttrs,
};

fn units(value: &str) -> Vec<u16> {
    if value == "-" {
        Vec::new()
    } else {
        value
            .split(',')
            .map(|s| u16::from_str_radix(s, 16).unwrap())
            .collect()
    }
}
fn number(value: &str) -> u32 {
    value.parse().unwrap()
}
fn snapshot(a: &CellData) -> Vec<i64> {
    vec![
        a.is_inverse().into(),
        a.is_bold().into(),
        a.is_underline().into(),
        a.is_blink().into(),
        a.is_invisible().into(),
        a.is_italic().into(),
        a.is_dim().into(),
        a.is_strikethrough().into(),
        a.is_protected().into(),
        a.is_overline().into(),
        a.get_fg_color_mode().into(),
        a.get_bg_color_mode().into(),
        a.is_fg_rgb().into(),
        a.is_bg_rgb().into(),
        a.is_fg_palette().into(),
        a.is_bg_palette().into(),
        a.is_fg_default().into(),
        a.is_bg_default().into(),
        a.is_attribute_default().into(),
        a.get_fg_color().into(),
        a.get_bg_color().into(),
        a.has_extended_attrs().into(),
        a.get_underline_color().into(),
        a.get_underline_color_mode().into(),
        a.is_underline_color_rgb().into(),
        a.is_underline_color_palette().into(),
        a.is_underline_color_default().into(),
        a.get_underline_style().into(),
        a.get_underline_variant_offset().into(),
        a.extended.ext().into(),
        a.extended.is_empty().into(),
        a.get_width().into(),
        a.is_combined().into(),
        a.get_code().map_or(-1, i64::from),
    ]
}

#[test]
fn seeded_raw_cells_match_javascript() {
    let data = include_str!("fixtures/cell-raw.tsv");
    assert_eq!(data.lines().count(), 2048);
    for (index, line) in data.lines().enumerate() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 13);
        let a = CellData {
            content: number(fields[4]),
            combined_data: units(fields[5]),
            attributes: AttributeData {
                fg: number(fields[0]),
                bg: number(fields[1]),
                extended: ExtendedAttrs::new(number(fields[2]), number(fields[3])),
            },
        };
        let expected: Vec<i64> = fields[6].split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(snapshot(&a), expected, "raw cell {index}");
        assert_eq!(a.get_chars(), units(fields[7]), "chars {index}");
        let b = CellData {
            attributes: AttributeData {
                fg: number(fields[8]),
                bg: number(fields[9]),
                extended: ExtendedAttrs::new(number(fields[10]), number(fields[11])),
            },
            ..CellData::default()
        };
        assert_eq!(
            a.attributes_equals(&b),
            fields[12] == "1",
            "attributes equals {index}"
        );
    }
}

#[test]
fn seeded_legacy_char_data_matches_javascript() {
    let data = include_str!("fixtures/cell-char-data.tsv");
    assert_eq!(data.lines().count(), 1024);
    for (index, line) in data.lines().enumerate() {
        let f: Vec<_> = line.split('\t').collect();
        assert_eq!(f.len(), 12);
        let mut cell = CellData {
            attributes: AttributeData {
                bg: u32::MAX,
                extended: ExtendedAttrs::new(u32::MAX, 42),
                ..AttributeData::default()
            },
            combined_data: "old".encode_utf16().collect(),
            ..CellData::default()
        };
        cell.set_from_char_data(&CharData {
            attr: number(f[0]),
            chars: units(f[2]),
            width: f[1].parse().unwrap(),
            code: Some(123456),
        });
        assert_eq!(cell.fg, number(f[0]), "fg {index}");
        assert_eq!(cell.content, number(f[3]), "content {index}");
        assert_eq!(cell.bg, number(f[4]), "bg {index}");
        assert_eq!(
            cell.get_width(),
            f[5].parse::<i32>().unwrap(),
            "width {index}"
        );
        assert_eq!(
            cell.get_code().map_or(-1, i64::from),
            f[6].parse::<i64>().unwrap(),
            "code {index}"
        );
        assert_eq!(cell.is_combined(), number(f[7]), "combined {index}");
        assert_eq!(cell.get_chars(), units(f[8]), "chars {index}");
        assert_eq!(
            cell.combined_data,
            units(f[9]),
            "stale combined storage {index}"
        );
        assert_eq!(cell.extended.ext(), number(f[10]), "ext {index}");
        assert_eq!(cell.extended.url_id(), number(f[11]), "url {index}");
    }
}

#[test]
fn payload_affects_emptiness_but_is_not_cloned_upstream() {
    let mut attrs = AttributeData::default();
    attrs.extended.set_payload(Some(Rc::new("opaque")));
    attrs.update_extended();
    assert_eq!(attrs.has_extended_attrs(), bg_flags::HAS_EXTENDED);
    let mut cloned = attrs.clone();
    assert!(cloned.extended.payload().is_none());
    assert!(cloned.extended.is_empty());
    // Upstream copies bg without recomputing the HAS_EXTENDED flag on clone.
    assert_eq!(cloned.has_extended_attrs(), bg_flags::HAS_EXTENDED);
    cloned.update_extended();
    assert_eq!(cloned.has_extended_attrs(), 0);
}

#[test]
fn buffer_handles_share_identity_while_explicit_clone_does_not() {
    let mut original = ExtendedAttrs::new(0, 7);
    original.set_payload(Some(Rc::new(42_u32)));
    let mut shared = original.share();
    assert!(original.same_identity(&shared));
    shared.set_url_id(13);
    shared.set_underline_variant_offset(6);
    assert_eq!(original.url_id(), 13);
    assert_eq!(original.underline_variant_offset(), 6);
    assert!(Rc::ptr_eq(
        &original.payload().unwrap(),
        &shared.payload().unwrap()
    ));
    let mut copied = original.clone();
    assert!(!original.same_identity(&copied));
    assert!(copied.payload().is_none());
    copied.set_url_id(99);
    assert_eq!(original.url_id(), 13);
}

#[test]
fn hyperlink_overrides_style_without_destroying_stored_style() {
    let mut ext = ExtendedAttrs::new(0, 0);
    ext.set_underline_style(underline_style::CURLY);
    ext.set_url_id(123);
    assert_eq!(ext.underline_style(), underline_style::DASHED);
    assert_eq!((ext.ext() >> 26) & 7, underline_style::DASHED);
    ext.set_url_id(0);
    assert_eq!(ext.underline_style(), underline_style::CURLY);
}

#[test]
fn lone_surrogates_and_empty_combined_strings_are_not_lost() {
    let mut cell = CellData::from_char_data(&CharData {
        attr: 0,
        chars: vec![0xd800],
        width: 1,
        code: None,
    });
    assert_eq!(cell.get_chars(), [0xd800]);
    cell.content = content::IS_COMBINED_MASK;
    cell.combined_data.clear();
    assert_eq!(cell.get_code(), None);
    assert_eq!(cell.get_chars(), []);
}

#[test]
fn rgb_conversion_masks_each_channel() {
    assert_eq!(
        AttributeData::from_color_rgb([0x101, 0x202, 0x303]),
        0x010203
    );
    assert_eq!(AttributeData::to_color_rgb(0xff010203), [1, 2, 3]);
}
