use std::cell::Cell;
use xterm_core::color::{channels, css, rgba, value, Color};

#[test]
fn canvas_fallback_preserves_input_and_rejects_alpha() {
    assert_eq!(css::to_color("red"), Err(css::UnsupportedCss));
    assert_eq!(
        css::to_color_with_canvas("red", |text| {
            assert_eq!(text, "red");
            Some([255, 0, 0, 255])
        }),
        Ok(Color {
            css: "red".into(),
            rgba: 0xff0000ff
        })
    );
    assert_eq!(
        css::to_color_with_canvas("invalid", |_| None),
        Err(css::UnsupportedCss)
    );
    assert_eq!(
        css::to_color_with_canvas("color", |_| Some([1, 2, 3, 254])),
        Err(css::UnsupportedCss)
    );
    let called = Cell::new(false);
    assert!(css::to_color_with_canvas("#abc", |_| {
        called.set(true);
        None
    })
    .is_ok());
    assert!(!called.get());
}

#[test]
fn source_unanchored_patterns_and_channel_overflow_are_preserved() {
    let result = css::to_color("x#abc").unwrap();
    assert_eq!(result.css, "#NaNaabbcc");
    assert_eq!(result.rgba, 0x00aabbcc);
    // Independent review cases exercise parseInt's non-digit prefix handling.
    for (input, expected) in [
        ("a-1#abc", 0xffffffff),
        ("a+1#abc", 511),
        ("a f#abc", 4095),
        ("a 1#abcde", 1),
        ("a0x1#abcd", 1),
    ] {
        assert_eq!(css::to_color(input).unwrap().rgba, expected, "{input}");
    }
    assert_eq!(
        css::to_color("prefix rgb(1,2,3,.5) suffix").unwrap(),
        channels::to_color(1, 2, 3, Some(128))
    );
    assert_eq!(
        css::to_color("rgb(999,999,999)").unwrap(),
        channels::to_color(999, 999, 999, Some(255))
    );
}

#[test]
fn full_color_operations_preserve_css_and_optional_results() {
    let bg = channels::to_color(0, 0, 0, None);
    let fg = channels::to_color(96, 96, 96, Some(128));
    assert_eq!(value::ensure_contrast_ratio(&bg, &fg, 1.0), None);
    assert_eq!(
        value::ensure_contrast_ratio(&bg, &fg, 4.0),
        Some(channels::to_color(112, 112, 112, None))
    );
    assert_eq!(
        value::multiply_opacity(&fg, 0.5),
        channels::to_color(96, 96, 96, Some(64))
    );
    assert_eq!(value::to_color_rgb(&fg), [96, 96, 96]);
    assert_eq!(Color::null().rgba, 0);
    assert_eq!(Color::null().css, "#00000000");
    assert_eq!(rgba::ensure_contrast_ratio(0, 0, f64::NAN), None);
}

#[test]
fn opacity_keeps_javascript_nonfinite_and_wide_number_behavior() {
    let color = channels::to_color(0, 0, 0, None);
    assert_eq!(value::opacity(&color, f64::NAN).css, "#000000NaN");
    assert_eq!(value::opacity(&color, f64::INFINITY).css, "#000000Infinity");
    assert_eq!(value::opacity(&color, f64::NEG_INFINITY).rgba, 0);
    assert_eq!(
        value::opacity(&color, 4503599627370497.0 / 255.0).css,
        "#00000010000000000001"
    );
    assert_eq!(value::opacity(&color, -1.0).css, "#000000-ff");
}
