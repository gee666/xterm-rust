use xterm_addons::image::numeric::fixed_zero;
use xterm_addons::image::options::ImageOptions;
use xterm_addons::image::policy::{
    GraphicsGeometry, ImageControlAction, ImageControlEffect, ImageControlPolicy,
    ImageParameter as P,
};
use xterm_addons::image::storage::CellSize;
fn policy() -> ImageControlPolicy {
    ImageControlPolicy::new(ImageOptions::default())
}
fn reply(action: &ImageControlAction) -> String {
    let [ImageControlEffect::Reply(units)] = action.effects.as_slice() else {
        panic!("expected one reply: {action:?}")
    };
    String::from_utf16(units).unwrap()
}
fn params(values: &[f64]) -> Vec<P> {
    values.iter().copied().map(P::Number).collect()
}
#[test]
fn runtime_defaults_are_not_stale_typings_and_reset_changes_only_source_fields() {
    let defaults = ImageOptions::default();
    assert!(
        defaults.enable_size_reports
            && defaults.sixel_support
            && defaults.iip_support
            && defaults.kitty_support
            && defaults.show_placeholder
            && defaults.sixel_scrolling
    );
    assert_eq!(defaults.pixel_limit, 16_777_216.0);
    assert_eq!(defaults.sixel_palette_limit, 4096.0);
    assert_eq!(
        (
            defaults.sixel_size_limit,
            defaults.iip_size_limit,
            defaults.kitty_size_limit
        ),
        (33_554_432.0, 33_554_432.0, 33_554_432.0)
    );
    let mut policy = ImageControlPolicy::new(ImageOptions {
        sixel_palette_limit: 16.0,
        sixel_scrolling: false,
        ..defaults
    });
    policy.options.sixel_palette_limit = 256.0;
    policy.options.sixel_scrolling = true;
    policy.options.storage_limit = 32.0;
    policy.options.show_placeholder = false;
    policy.options.pixel_limit = 4.0;
    let action = policy.reset();
    assert!(!action.handled);
    assert_eq!(
        action.effects,
        [
            ImageControlEffect::ClearStorage,
            ImageControlEffect::ResetHandlers
        ]
    );
    assert_eq!(policy.options.sixel_palette_limit, 16.0);
    assert!(!policy.options.sixel_scrolling);
    assert_eq!(policy.options.storage_limit, 32.0);
    assert_eq!(policy.options.pixel_limit, 4.0);
    assert!(!policy.options.show_placeholder);
    assert_eq!(defaults.storage_limit, 128.0);
}
#[test]
fn dec80_polarity_bubbles_and_array_parameters_do_not_match_scalar80() {
    let mut policy = policy();
    assert!(!policy.decset(&params(&[80.0])).handled);
    assert!(!policy.options.sixel_scrolling);
    assert!(!policy.decrst(&[P::Subparams(vec![80.0])]).handled);
    assert!(!policy.options.sixel_scrolling);
    assert!(!policy.decrst(&params(&[80.0])).handled);
    assert!(policy.options.sixel_scrolling);
    assert!(!policy.decset(&params(&[1.0, 80.0, 80.0])).handled);
    assert!(!policy.options.sixel_scrolling);
}
#[test]
fn primary_device_attributes_preserve_truthiness_and_disabled_fallback() {
    let mut policy = policy();
    for parameters in [
        vec![],
        params(&[0.0]),
        params(&[-0.0]),
        params(&[f64::NAN]),
        vec![P::Undefined],
    ] {
        assert_eq!(
            reply(&policy.primary_device_attributes(&parameters)),
            "\x1b[?62;4;9;22c"
        );
    }
    for parameter in [
        P::Number(1.0),
        P::Number(f64::INFINITY),
        P::Subparams(vec![]),
        P::Subparams(vec![0.0]),
    ] {
        let action = policy.primary_device_attributes(&[parameter]);
        assert!(action.handled);
        assert!(action.effects.is_empty());
    }
    policy.options.sixel_support = false;
    assert!(!policy.primary_device_attributes(&[]).handled);
    assert!(policy.primary_device_attributes(&params(&[1.0])).handled);
}
#[test]
fn graphics_colors_preserve_source_scalar_boundaries_and_ordered_reset() {
    let mut policy = ImageControlPolicy::new(ImageOptions {
        sixel_palette_limit: 128.0,
        ..ImageOptions::default()
    });
    let geometry = GraphicsGeometry::default();
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[1.0, 1.0]), geometry)),
        "\x1b[?1;0;128S"
    );
    for value in [4096.0, -1.0, -0.0, 1.5, f64::NEG_INFINITY] {
        let action = policy.graphics_attributes(&params(&[1.0, 3.0, value]), geometry);
        assert!(action.handled);
        assert_eq!(policy.options.sixel_palette_limit, value);
    }
    for third in [
        P::Number(4097.0),
        P::Number(f64::NAN),
        P::Subparams(vec![1.0]),
        P::Undefined,
    ] {
        let previous = policy.options.sixel_palette_limit;
        assert_eq!(
            reply(&policy.graphics_attributes(&[P::Number(1.0), P::Number(3.0), third], geometry)),
            "\x1b[?1;2S"
        );
        assert_eq!(policy.options.sixel_palette_limit, previous);
    }
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[1.0, 3.0]), geometry)),
        "\x1b[?1;2S"
    );
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[1.0, 4.0]), geometry)),
        "\x1b[?1;0;4096S"
    );
    let reset = policy.graphics_attributes(&params(&[1.0, 2.0]), geometry);
    assert_eq!(policy.options.sixel_palette_limit, 128.0);
    assert!(
        matches!(&reset.effects[..],[ImageControlEffect::Reply(units),ImageControlEffect::ResetHandlers] if String::from_utf16(units).unwrap()=="\x1b[?1;0;128S")
    );
}
#[test]
fn graphics_geometry_uses_strict_area_limit_and_js_fixed_zero() {
    let mut policy = policy();
    let measured = GraphicsGeometry {
        canvas: Some(CellSize {
            width: 20.5,
            height: 10.5,
        }),
        columns: 80.0,
        rows: 24.0,
    };
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[2.0, 1.0]), measured)),
        "\x1b[?2;0;21;11S"
    );
    policy.options.pixel_limit = 100.0;
    for value in [10.0, 11.0] {
        let measured = GraphicsGeometry {
            canvas: Some(CellSize {
                width: value,
                height: value,
            }),
            ..measured
        };
        assert_eq!(
            reply(&policy.graphics_attributes(&params(&[2.0, 1.0]), measured)),
            "\x1b[?2;0;10;10S"
        );
    }
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[2.0, 4.0]), measured)),
        "\x1b[?2;0;10;10S"
    );
    policy.options.pixel_limit = -1.0;
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[2.0, 4.0]), measured)),
        "\x1b[?2;0;NaN;NaNS"
    );
}
#[test]
fn graphics_missing_dimensions_fall_back_as_a_pair_and_bad_items_keep_source_text() {
    let mut policy = policy();
    let geometry = GraphicsGeometry {
        canvas: Some(CellSize {
            width: 100.0,
            height: 0.0,
        }),
        columns: 0.0,
        rows: f64::NAN,
    };
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[2.0, 1.0]), geometry)),
        "\x1b[?2;0;560;336S"
    );
    assert_eq!(
        reply(
            &policy.graphics_attributes(&[P::Subparams(vec![1.0, 2.0]), P::Number(1.0)], geometry)
        ),
        "\x1b[?1,2;1S"
    );
    assert_eq!(
        reply(&policy.graphics_attributes(&[P::Undefined, P::Number(1.0)], geometry)),
        "\x1b[?undefined;1S"
    );
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[3.0, 1.0]), geometry)),
        "\x1b[?3;1S"
    );
    assert_eq!(
        reply(&policy.graphics_attributes(&params(&[1.0, 99.0]), geometry)),
        "\x1b[?1;2S"
    );
    let action = policy.graphics_attributes(&params(&[1.0]), geometry);
    assert!(action.handled);
    assert!(action.effects.is_empty());
}
#[test]
fn fixed_zero_keeps_negative_rounded_zero_and_large_number_format() {
    for (value, expected) in [
        (-0.1, "-0"),
        (-0.0, "0"),
        (0.5, "1"),
        (-0.5, "-1"),
        (1e21, "1e+21"),
        (f64::NAN, "NaN"),
    ] {
        assert_eq!(fixed_zero(value), expected);
    }
}
