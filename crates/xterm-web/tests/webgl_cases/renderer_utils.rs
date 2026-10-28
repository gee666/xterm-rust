// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Shared renderer source branch and numeric boundaries.
use xterm_web::renderer::utils::{
    allow_rescaling, compute_next_variant_offset, create_render_dimensions, is_emoji,
    is_powerline_glyph, is_restricted_powerline_glyph, treat_glyph_as_background_color, FalsyValue,
};

#[test]
fn powerline_and_background_ranges_preserve_every_boundary() {
    for (code, powerline, restricted, background) in [
        (0xe0a3, false, false, false),
        (0xe0a4, true, false, true),
        (0xe0af, true, false, true),
        (0xe0b0, true, true, true),
        (0xe0b7, true, true, true),
        (0xe0b8, true, false, true),
        (0xe0d6, true, false, true),
        (0xe0d7, false, false, false),
        (0x24ff, false, false, false),
        (0x2500, false, false, true),
        (0x259f, false, false, true),
        (0x25a0, false, false, false),
    ] {
        assert_eq!(is_powerline_glyph(code), powerline, "code {code:x}");
        assert_eq!(
            is_restricted_powerline_glyph(code),
            restricted,
            "code {code:x}"
        );
        assert_eq!(
            treat_glyph_as_background_color(code),
            background,
            "code {code:x}"
        );
    }
}

#[test]
fn every_source_emoji_range_endpoint_and_neighbor() {
    for (first, last) in [
        (0x1f600, 0x1f64f),
        (0x1f300, 0x1f5ff),
        (0x1f680, 0x1f6ff),
        (0x2600, 0x26ff),
        (0x2700, 0x27bf),
        (0xfe00, 0xfe0f),
        (0x1f900, 0x1f9ff),
        (0x1f1e6, 0x1f1ff),
    ] {
        assert!(is_emoji(first));
        assert!(is_emoji(last));
    }
    for code in [
        0x25ff,
        0x27c0,
        0xfdff,
        0xfe10,
        0x1f1e5,
        0x1f200,
        0x1f2ff,
        0x1f650,
        0x1f67f,
        0x1f700,
        0x1f8ff,
        0x1fa00,
        u32::MAX,
    ] {
        assert!(!is_emoji(code), "code {code:x}");
    }
}

#[test]
fn rescaling_threshold_width_and_every_exclusion() {
    assert!(!allow_rescaling(Some(0x100), 1.0, 15.0, 10.0));
    assert!(allow_rescaling(Some(0x100), 1.0, 15.01, 10.0));
    assert!(!allow_rescaling(Some(0x100), 1.0, 16.0, 10.5));
    assert!(allow_rescaling(Some(0x100), 1.0, 16.01, 10.5));
    assert!(!allow_rescaling(None, 1.0, 99.0, 10.0));
    for width in [0.0, 2.0, 1.5, f64::NAN] {
        assert!(!allow_rescaling(Some(0x100), width, 99.0, 10.0));
    }
    for code in [0, 0xff, 0x2600, 0x1f600, 0xe000, 0xe0a4, 0xf8ff] {
        assert!(
            !allow_rescaling(Some(code), 1.0, 99.0, 10.0),
            "code {code:x}"
        );
    }
    assert!(allow_rescaling(Some(0xf900), 1.0, 99.0, 10.0));
    // The JS combined marker produces a negative int32 before Uint32 storage.
    let combined = i64::from((xterm_web::webgl::model::COMBINED_CHAR_BIT_MASK | 0x100) as i32);
    assert!(!allow_rescaling(Some(combined), 1.0, 99.0, 10.0));
    assert!(allow_rescaling(Some(i64::MAX), 1.0, 99.0, 10.0));
    assert!(!allow_rescaling(Some(0x100), 1.0, f64::NAN, 10.0));
}

#[test]
fn variant_offsets_preserve_js_round_ties_remainder_and_signed_zero() {
    assert_eq!(compute_next_variant_offset(5.0, 1.5, None), 1.0);
    assert_eq!(compute_next_variant_offset(5.0, 1.5, Some(1.0)), 2.0);
    assert_eq!(compute_next_variant_offset(3.0, -1.5, None), 1.0);
    assert!(compute_next_variant_offset(3.0, -0.5, None).is_nan());
    assert!(compute_next_variant_offset(3.0, 0.0, None).is_nan());
    assert!(compute_next_variant_offset(3.0, f64::NAN, None).is_nan());
    assert_eq!(
        compute_next_variant_offset(0.0, 1.0, None).to_bits(),
        (-0.0_f64).to_bits()
    );
}

#[test]
fn initial_dimensions_and_source_falsy_error_text() {
    let dimensions = create_render_dimensions();
    assert_eq!(dimensions.css.canvas.width, 0.0);
    assert_eq!(dimensions.css.canvas.height, 0.0);
    assert_eq!(dimensions.css.cell.width, 0.0);
    assert_eq!(dimensions.css.cell.height, 0.0);
    assert_eq!(dimensions.device.canvas.width, 0.0);
    assert_eq!(dimensions.device.canvas.height, 0.0);
    assert_eq!(dimensions.device.cell.width, 0.0);
    assert_eq!(dimensions.device.cell.height, 0.0);
    assert_eq!(dimensions.device.character.width, 0.0);
    assert_eq!(dimensions.device.character.height, 0.0);
    assert_eq!(dimensions.device.character.left, 0.0);
    assert_eq!(dimensions.device.character.top, 0.0);
    assert_eq!(FalsyValue.to_string(), "value must not be falsy");
}
