// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Portable WebglUtils and Constants boundary assertions.
use std::rc::Rc;
use xterm_web::webgl::math::{
    expand_float32_array, text_baseline, FloatArrayError, GlTexture, TextBaseline, DIM_OPACITY,
    PROJECTION_MATRIX,
};

#[test]
fn projection_maps_source_unit_coordinates_to_clip_space() {
    for (x, y, expected) in [
        (0.0, 0.0, [-1.0, 1.0]),
        (1.0, 1.0, [1.0, -1.0]),
        (0.5, 0.5, [0.0, 0.0]),
    ] {
        let output = [
            PROJECTION_MATRIX[0] * x + PROJECTION_MATRIX[4] * y + PROJECTION_MATRIX[12],
            PROJECTION_MATRIX[1] * x + PROJECTION_MATRIX[5] * y + PROJECTION_MATRIX[13],
        ];
        assert_eq!(output, expected);
    }
    assert_eq!(PROJECTION_MATRIX[10], 1.0);
    assert_eq!(PROJECTION_MATRIX[15], 1.0);
}

#[test]
fn growth_doubles_caps_and_truncates_without_aliasing() {
    let source = [1.0_f32, -0.0, 3.0];
    assert_eq!(
        expand_float32_array(&source, 6.0).unwrap(),
        [1.0, -0.0, 3.0, 0.0, 0.0, 0.0]
    );
    assert_eq!(
        expand_float32_array(&source, 4.8).unwrap(),
        [1.0, -0.0, 3.0, 0.0]
    );
    assert_eq!(expand_float32_array(&source, 2.0).unwrap(), [1.0, -0.0]);
    let mut result = expand_float32_array(&source, 100.0).unwrap();
    assert_eq!(result.len(), 6);
    assert_eq!(result[1].to_bits(), (-0.0_f32).to_bits());
    assert_eq!(result[3].to_bits(), 0.0_f32.to_bits());
    result[0] = 99.0;
    assert_eq!(source[0], 1.0);
}

#[test]
fn numeric_constructor_edges_match_typed_array_to_index() {
    let source = [1.0_f32, 2.0];
    assert!(expand_float32_array(&source, f64::NAN).unwrap().is_empty());
    assert!(expand_float32_array(&source, -0.5).unwrap().is_empty());
    assert_eq!(
        expand_float32_array(&source, f64::INFINITY).unwrap(),
        [1.0, 2.0, 0.0, 0.0]
    );
    assert!(
        matches!(expand_float32_array(&source,-1.0),Err(FloatArrayError::InvalidLength(value)) if value == -1.0)
    );
    assert!(
        matches!(expand_float32_array(&source,f64::NEG_INFINITY),Err(FloatArrayError::InvalidLength(value)) if value.is_infinite() && value.is_sign_negative())
    );
}

#[test]
fn texture_version_retains_handle_identity_and_starts_dirty() {
    let handle = Rc::new(17);
    let texture = GlTexture::new(Rc::clone(&handle));
    assert!(Rc::ptr_eq(&handle, &texture.texture));
    assert_eq!(texture.version, -1.0);
}

#[test]
fn source_browser_baselines_and_dim_opacity() {
    assert_eq!(text_baseline(false, false), TextBaseline::Ideographic);
    assert_eq!(text_baseline(true, false), TextBaseline::Bottom);
    assert_eq!(text_baseline(false, true), TextBaseline::Bottom);
    assert_eq!(text_baseline(true, true), TextBaseline::Bottom);
    assert_eq!(TextBaseline::Bottom.as_str(), "bottom");
    assert_eq!(TextBaseline::Ideographic.as_str(), "ideographic");
    assert_eq!(DIM_OPACITY, 0.5);
}
