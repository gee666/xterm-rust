use xterm_core::buffer::{
    constants::{
        attributes as color, bg_flags as bg, content, fg_flags as fg, underline_style as style,
    },
    AttributeData, CellData, CharData,
};

fn cell(fg: u32, bg: u32) -> CellData {
    CellData {
        attributes: AttributeData {
            fg,
            bg,
            ..AttributeData::default()
        },
        ..CellData::default()
    }
}

#[test]
fn legacy_utf16_conversion_branches_and_signed_width_boundaries() {
    let cases: &[(&[u16], &[u16], u32, bool)] = &[
        (&[], &[], 0, false),
        (&[0], &[], 0, false),
        (&[97], &[97], 97, false),
        (&[0xd800], &[0xd800], 0xd800, false),
        (&[0xdc00], &[0xdc00], 0xdc00, false),
        (&[0xd800, 0xdc00], &[0xd800, 0xdc00], 0x10000, false),
        (&[0xdbff, 0xdfff], &[0xdbff, 0xdfff], 0x10ffff, false),
        (&[0xdc00, 0xd800], &[0xdc00, 0xd800], 0xd800, true),
        (&[0xd800, 97], &[0xd800, 97], 97, true),
        (&[97, 0xdc00], &[97, 0xdc00], 0xdc00, true),
        (&[97, 98], &[97, 98], 98, true),
        (
            &[0xd800, 0xdc00, 0xd800, 0xdc00],
            &[0xd800, 0xdc00, 0xd800, 0xdc00],
            0xdc00,
            true,
        ),
    ];
    for &(input, chars, code, combined) in cases {
        for width in [0, 1, 2, 3, 456, -1, i32::MIN, i32::MAX] {
            let mut a = cell(0, u32::MAX);
            a.extended.set_url_id(3);
            a.combined_data = vec![123];
            let identity = a.extended.share();
            a.set_from_char_data(&CharData {
                attr: 42,
                chars: input.to_vec(),
                width,
                code: Some(999),
            });
            assert_eq!(a.fg, 42);
            assert_eq!(a.bg, 0);
            assert_eq!(a.get_code(), Some(code));
            assert_eq!(a.get_chars(), chars);
            assert_eq!(
                a.get_as_char_data(),
                CharData {
                    attr: 42,
                    chars: chars.to_vec(),
                    width: a.get_width(),
                    code: Some(code),
                }
            );
            assert_eq!(a.is_combined() != 0, combined);
            assert_eq!(
                a.get_width(),
                ((width as u32) << content::WIDTH_SHIFT) as i32 >> content::WIDTH_SHIFT
            );
            assert!(a.extended.same_identity(&identity));
            assert_eq!(a.extended.url_id(), 3);
            if !combined {
                assert_eq!(a.combined_data, [123]);
            }
        }
    }
    let a = CellData::default();
    assert_eq!(a.content, 0);
    assert_eq!(a.get_width(), 0);
    assert!(a.get_chars().is_empty());
    assert_eq!(a.get_code(), Some(0));
}

#[test]
fn foreground_and_background_modes_payloads_and_rgb_masking() {
    for mode in [
        color::CM_DEFAULT,
        color::CM_P16,
        color::CM_P256,
        color::CM_RGB,
    ] {
        let a = cell(mode | 0x123456, mode | 0xabcdef);
        let expected_fg = match mode {
            color::CM_DEFAULT => -1,
            color::CM_RGB => 0x123456,
            _ => 0x56,
        };
        let expected_bg = match mode {
            color::CM_DEFAULT => -1,
            color::CM_RGB => 0xabcdef,
            _ => 0xef,
        };
        assert_eq!(a.get_fg_color(), expected_fg);
        assert_eq!(a.get_bg_color(), expected_bg);
        assert_eq!(a.get_fg_color_mode(), mode);
        assert_eq!(a.get_bg_color_mode(), mode);
        assert_eq!(a.is_fg_rgb(), mode == color::CM_RGB);
        assert_eq!(a.is_bg_rgb(), mode == color::CM_RGB);
        assert_eq!(
            a.is_fg_palette(),
            matches!(mode, color::CM_P16 | color::CM_P256)
        );
        assert_eq!(a.is_bg_palette(), a.is_fg_palette());
        assert_eq!(a.is_fg_default(), mode == 0);
        assert_eq!(a.is_bg_default(), mode == 0);
    }
    let mut defaults = cell(0, 0);
    defaults.extended.set_url_id(9);
    assert!(defaults.is_attribute_default());
    assert!(!cell(1, 0).is_attribute_default());
    assert!(!cell(0, 1).is_attribute_default());
    for r in [0, 1, 255, 256, u32::MAX] {
        for g in [0, 1, 255, 256, u32::MAX] {
            for b in [0, 1, 255, 256, u32::MAX] {
                let packed = AttributeData::from_color_rgb([r, g, b]);
                assert_eq!(
                    AttributeData::to_color_rgb(packed | 0xff000000),
                    [r as u8, g as u8, b as u8]
                );
            }
        }
    }
}

#[test]
fn every_individual_flag_has_exact_mask_and_no_other_flag() {
    let foreground = [
        fg::INVERSE,
        fg::BOLD,
        fg::UNDERLINE,
        fg::BLINK,
        fg::INVISIBLE,
        fg::STRIKETHROUGH,
    ];
    let background = [bg::ITALIC, bg::DIM, bg::PROTECTED, bg::OVERLINE];
    for (index, &flag) in foreground.iter().enumerate() {
        let a = cell(flag, 0);
        let values = [
            a.is_inverse(),
            a.is_bold(),
            a.is_underline(),
            a.is_blink(),
            a.is_invisible(),
            a.is_strikethrough(),
        ];
        for (query, value) in values.into_iter().enumerate() {
            assert_eq!(value, if query == index { flag } else { 0 });
        }
    }
    for (index, &flag) in background.iter().enumerate() {
        let a = cell(0, flag);
        let values = [a.is_italic(), a.is_dim(), a.is_protected(), a.is_overline()];
        for (query, value) in values.into_iter().enumerate() {
            assert_eq!(value, if query == index { flag } else { 0 });
        }
    }
}

#[test]
fn underline_presence_style_and_default_color_are_distinct_queries() {
    let mut a = cell(color::CM_P256 | 42, 0);
    a.extended.set_underline_style(style::CURLY);
    a.extended.set_underline_color(color::CM_RGB | 0x123456);
    assert_eq!(a.is_underline(), 0);
    assert_eq!(a.get_underline_color(), 42);
    assert_eq!(a.get_underline_color_mode(), color::CM_P256);
    a.bg |= bg::HAS_EXTENDED;
    assert_eq!(a.is_underline(), 1);
    assert_eq!(a.get_underline_style(), style::NONE);
    assert_eq!(a.get_underline_color(), 0x123456);
    a.fg |= fg::UNDERLINE;
    assert_eq!(a.get_underline_style(), style::CURLY);
    a.extended.set_underline_color(0x123456);
    assert!(a.is_underline_color_default());
    assert_eq!(a.get_underline_color(), 42);
    a.extended.set_underline_style(style::NONE);
    assert_eq!(a.is_underline(), fg::UNDERLINE);
    assert_eq!(a.get_underline_style(), style::NONE);
}

#[test]
fn equality_checks_each_visible_flag_but_excludes_protection() {
    let a = cell(0, 0);
    for flag in [
        fg::INVERSE,
        fg::BOLD,
        fg::UNDERLINE,
        fg::BLINK,
        fg::INVISIBLE,
        fg::STRIKETHROUGH,
    ] {
        let b = cell(flag, 0);
        assert!(!a.attributes_equals(&b));
        assert!(!b.attributes_equals(&a));
    }
    for flag in [bg::ITALIC, bg::DIM, bg::OVERLINE] {
        let b = cell(0, flag);
        assert!(!a.attributes_equals(&b));
        assert!(!b.attributes_equals(&a));
    }
    assert!(a.attributes_equals(&cell(0, bg::PROTECTED)));
    let mut b = cell(0, 0);
    b.content = (2 << content::WIDTH_SHIFT) | 97;
    assert!(a.attributes_equals(&b));
}

#[test]
fn equality_decodes_modes_colors_and_default_underline_branches() {
    let a = cell(color::CM_P16 | 1, color::CM_P256 | 2);
    assert!(!a.attributes_equals(&cell(color::CM_P256 | 1, a.bg)));
    assert!(!a.attributes_equals(&cell(color::CM_P16 | 2, a.bg)));
    assert!(!a.attributes_equals(&cell(a.fg, color::CM_P16 | 2)));
    assert!(!a.attributes_equals(&cell(a.fg, color::CM_P256 | 3)));
    assert!(a.attributes_equals(&cell(a.fg | 0x100, a.bg | 0x100)));
    let mut a = cell(fg::UNDERLINE | color::CM_P256 | 12, bg::HAS_EXTENDED);
    a.extended.set_underline_color(45);
    let mut b = a.clone();
    b.extended.set_underline_color(46);
    assert!(a.attributes_equals(&b)); // Both default modes ignore payload differences.
    b.extended.set_underline_color(color::CM_P16 | 12);
    assert!(!a.attributes_equals(&b)); // Same decoded color, but exactly one default mode.
    a.extended.set_underline_color(color::CM_P256 | 12);
    assert!(!a.attributes_equals(&b)); // Explicit color mode mismatch.
    b.extended.set_underline_color(color::CM_P256 | 13);
    assert!(!a.attributes_equals(&b)); // Explicit payload mismatch.
    b.extended.set_underline_color(color::CM_P256 | 12);
    assert!(a.attributes_equals(&b));
    for item in [&a, &b] {
        assert!(item.attributes_equals(item));
    }
}

#[test]
fn equality_uses_numeric_underline_and_not_a_rendering_guess() {
    let mut a = cell(fg::UNDERLINE, 0);
    let mut b = a.clone();
    b.bg = bg::HAS_EXTENDED;
    b.extended.set_underline_style(style::SINGLE);
    assert_eq!(a.get_underline_style(), b.get_underline_style());
    assert_eq!(a.is_underline(), fg::UNDERLINE);
    assert_eq!(b.is_underline(), 1);
    assert!(!a.attributes_equals(&b));
    assert!(!b.attributes_equals(&a));

    // A nonzero extended style reports underline even without the foreground flag.
    a.fg = 0;
    b.fg = 0;
    assert!(!a.attributes_equals(&b));
    a.bg = bg::HAS_EXTENDED;
    a.extended.set_underline_style(style::CURLY);
    assert!(a.attributes_equals(&b)); // Both report style NONE and numeric underline 1.
    b.extended.set_underline_color(color::CM_P16 | 7);
    assert!(!a.attributes_equals(&b));

    // With no numeric underline, extended colors and unenabled styles are ignored.
    a.bg = 0;
    b.bg = 0;
    assert!(a.attributes_equals(&b));
    a.extended.set_url_id(99);
    b.extended.set_url_id(100);
    assert!(a.attributes_equals(&b));
}

#[test]
fn raw_content_masks_and_out_of_range_codepoints_keep_js_truncation() {
    assert_eq!(content::CODEPOINT_MASK, 0x1fffff);
    assert_eq!(content::IS_COMBINED_MASK, 0x200000);
    assert_eq!(content::WIDTH_SHIFT, 22);
    for code in [0x110000, 0x1fffff] {
        let mut a = cell(0, 0);
        a.content = code;
        let adjusted = code - 0x10000;
        assert_eq!(
            a.get_chars(),
            [
                (0xd800 + (adjusted >> 10)) as u16,
                (0xdc00 + (adjusted & 0x3ff)) as u16
            ]
        );
        assert_eq!(a.get_code(), Some(code));
    }
}
