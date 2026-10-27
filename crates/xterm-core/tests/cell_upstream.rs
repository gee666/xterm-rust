// Copyright (c) 2018, 2026 The xterm.js authors. All rights reserved. MIT license.
// Ports CellData.test.ts and BufferLine.test.ts AttributeData/CellData describes.
use xterm_core::buffer::{
    constants::{attributes as color, bg_flags, content, fg_flags, underline_style as style},
    AttributeData, CellData, CharData,
};

fn styled_cell(ch: char, underline: u32, underline_color: u32) -> CellData {
    let mut cell = CellData::from_char_data(&CharData {
        attr: color::CM_P256 | 12 | fg_flags::BOLD | fg_flags::UNDERLINE,
        chars: ch.to_string().encode_utf16().collect(),
        width: 1,
        code: Some(ch as u32),
    });
    cell.bg = color::CM_P16 | 2 | bg_flags::ITALIC;
    cell.extended.set_underline_style(underline);
    cell.extended
        .set_underline_color(color::CM_P256 | underline_color);
    cell.update_extended();
    cell
}

#[test]
fn returns_true_for_same_attributes_with_different_chars() {
    assert!(
        styled_cell('A', style::DOUBLE, 45).attributes_equals(&styled_cell('B', style::DOUBLE, 45))
    );
}
#[test]
fn detects_underline_style_changes() {
    assert!(
        !styled_cell('A', style::DOUBLE, 45).attributes_equals(&styled_cell(
            'B',
            style::SINGLE,
            45
        ))
    );
}
#[test]
fn detects_underline_color_changes() {
    assert!(
        !styled_cell('A', style::SINGLE, 45).attributes_equals(&styled_cell(
            'B',
            style::SINGLE,
            46
        ))
    );
}
#[test]
fn ignores_underline_variant_offsets() {
    let mut a = styled_cell('A', style::SINGLE, 45);
    let mut b = styled_cell('B', style::SINGLE, 45);
    a.extended.set_underline_variant_offset(1);
    b.extended.set_underline_variant_offset(3);
    a.update_extended();
    b.update_extended();
    assert!(a.attributes_equals(&b));
}
#[test]
fn ignores_url_ids() {
    let mut a = styled_cell('A', style::SINGLE, 45);
    let mut b = styled_cell('B', style::SINGLE, 45);
    a.extended.set_url_id(1);
    b.extended.set_url_id(2);
    a.update_extended();
    b.update_extended();
    assert!(a.attributes_equals(&b));
}
#[test]
fn has_extended_attrs() {
    let mut attrs = AttributeData::default();
    assert_eq!(attrs.has_extended_attrs(), 0);
    attrs.bg |= bg_flags::HAS_EXTENDED;
    assert_ne!(attrs.has_extended_attrs(), 0);
}
fn underline_color_uses_fg_until_enabled(value: u32, expected: i32) {
    let mut attrs = AttributeData::default();
    attrs.extended.set_underline_color(value);
    assert_eq!(attrs.get_underline_color(), -1);
    attrs.bg |= bg_flags::HAS_EXTENDED;
    assert_eq!(attrs.get_underline_color(), expected);
    attrs.extended.set_underline_color(0);
    attrs.fg |= color::CM_P256 | 123;
    assert_eq!(attrs.get_underline_color(), 123);
}
#[test]
fn get_underline_color_p256() {
    underline_color_uses_fg_until_enabled(color::CM_P256 | 45, 45);
}
#[test]
fn get_underline_color_rgb() {
    underline_color_uses_fg_until_enabled(color::CM_RGB | 0x010203, 0x010203);
}
#[test]
fn underline_color_mode_and_queries() {
    let mut attrs = AttributeData::default();
    let modes = [
        color::CM_DEFAULT,
        color::CM_P16,
        color::CM_P256,
        color::CM_RGB,
    ];
    for mode in modes {
        attrs.extended.set_underline_color(mode);
        assert_eq!(attrs.get_underline_color_mode(), attrs.get_fg_color_mode());
        assert!(attrs.is_underline_color_default());
    }
    attrs.fg = color::CM_RGB;
    for mode in modes {
        attrs.extended.set_underline_color(mode);
        assert_eq!(attrs.get_underline_color_mode(), attrs.get_fg_color_mode());
        assert!(!attrs.is_underline_color_default());
        assert!(attrs.is_underline_color_rgb());
    }
    attrs.bg |= bg_flags::HAS_EXTENDED;
    attrs.extended.set_underline_color(color::CM_DEFAULT);
    assert_eq!(attrs.get_underline_color_mode(), color::CM_DEFAULT);
    attrs.extended.set_underline_color(color::CM_P16);
    assert_eq!(attrs.get_underline_color_mode(), color::CM_P16);
    assert!(attrs.is_underline_color_palette());
    attrs.extended.set_underline_color(color::CM_P256);
    assert_eq!(attrs.get_underline_color_mode(), color::CM_P256);
    assert!(attrs.is_underline_color_palette());
    attrs.extended.set_underline_color(color::CM_RGB);
    assert_eq!(attrs.get_underline_color_mode(), color::CM_RGB);
    assert!(attrs.is_underline_color_rgb());
}
#[test]
fn get_underline_style() {
    let mut attrs = AttributeData::default();
    assert_eq!(attrs.get_underline_style(), style::NONE);
    attrs.extended.set_underline_style(style::CURLY);
    assert_eq!(attrs.get_underline_style(), style::NONE);
    attrs.fg |= fg_flags::UNDERLINE;
    assert_eq!(attrs.get_underline_style(), style::SINGLE);
    attrs.bg |= bg_flags::HAS_EXTENDED;
    assert_eq!(attrs.get_underline_style(), style::CURLY);
    attrs.fg &= !fg_flags::UNDERLINE;
    assert_eq!(attrs.get_underline_style(), style::NONE);
}
#[test]
fn get_underline_variant_offset() {
    let mut attrs = AttributeData::default();
    assert_eq!(attrs.get_underline_variant_offset(), 0);
    for i in 0..8 {
        attrs.extended.set_underline_variant_offset(i);
        assert_eq!(attrs.get_underline_variant_offset(), i);
    }
}
#[test]
fn char_data_cell_data_equality() {
    let mut cell = CellData::default();
    for (chars, width, code, combined) in [
        ("a", 1, 97, false),
        ("e\u{0301}", 1, 0x301, true),
        ("𝄞", 1, 0x1d11e, false),
        ("𓂀\u{0301}", 1, 0x301, true),
        ("１", 2, 0xff11, false),
    ] {
        let data = CharData {
            attr: 123,
            chars: chars.encode_utf16().collect(),
            width,
            code: Some(code),
        };
        cell.set_from_char_data(&data);
        assert_eq!(cell.get_as_char_data(), data);
        assert_eq!(
            cell.is_combined(),
            if combined {
                content::IS_COMBINED_MASK
            } else {
                0
            }
        );
    }
}
