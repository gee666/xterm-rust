// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. CharAtlasUtils source assertions and branch boundaries.
use crate::webgl_support::{config, text};
use std::rc::Rc;
use xterm_core::{buffer::constants::attributes, color::Color};
use xterm_web::webgl::config::{
    config_equals, is_256_color, AtlasConfig, ConfigFactory, FontWeight, MissingAnsiColor,
};

#[test]
fn original_identical_configs() {
    assert!(config_equals(&config(), &config()).unwrap());
}
#[test]
fn original_max_texture_size_differs() {
    let a = config();
    let mut b = config();
    b.dimensions.device_max_texture_size = 8192.0;
    assert!(!config_equals(&a, &b).unwrap());
}
#[test]
fn original_cell_width_differs() {
    let a = config();
    let mut b = config();
    b.dimensions.device_cell_width = 11.0;
    assert!(!config_equals(&a, &b).unwrap());
}
#[test]
fn original_cell_height_differs() {
    let a = config();
    let mut b = config();
    b.dimensions.device_cell_height = 21.0;
    assert!(!config_equals(&a, &b).unwrap());
}

#[test]
fn each_remaining_config_field_participates_in_equality() {
    type Change = fn(&mut AtlasConfig);
    let changes: &[Change] = &[
        |b| b.custom_glyphs = false,
        |b| b.dimensions.device_pixel_ratio = 1.25,
        |b| b.dimensions.device_char_width = 9.0,
        |b| b.dimensions.device_char_height = 17.0,
        |b| b.options.line_height = 1.2,
        |b| b.options.letter_spacing = 1.0,
        |b| b.options.font_family = text("other"),
        |b| b.options.font_size = 16.0,
        |b| b.options.font_weight = FontWeight::Number(400.0),
        |b| b.options.font_weight_bold = FontWeight::Number(700.0),
        |b| b.options.allow_transparency = true,
        |b| b.options.draw_bold_text_in_bright_colors = false,
        |b| b.options.minimum_contrast_ratio = 2.0,
        |b| b.colors.foreground = Color::null().into(),
        |b| b.colors.background = Color::null().into(),
    ];
    for (index, change) in changes.iter().enumerate() {
        let a = config();
        let mut b = config();
        change(&mut b);
        assert!(!config_equals(&a, &b).unwrap(), "field mutation {index}");
    }
}

#[test]
fn every_ansi_entry_is_checked_and_css_is_not_compared() {
    let a = config();
    for index in 0..256 {
        let mut b = config();
        b.colors.ansi[index] = Color::null().into();
        assert!(!config_equals(&a, &b).unwrap(), "ANSI index {index}");
    }
    let b = config();
    b.colors.foreground.borrow_mut().css = "different css".into();
    assert!(config_equals(&a, &b).unwrap());
}

#[test]
fn ansi_length_comparison_preserves_source_asymmetry_and_error() {
    let a = config();
    let mut b = config();
    b.colors.ansi.push(Color::null().into());
    assert!(config_equals(&a, &b).unwrap());
    assert_eq!(config_equals(&b, &a), Err(MissingAnsiColor { index: 256 }));
}

#[test]
fn generated_colors_share_objects_and_caches_but_not_ansi_vector_slots() {
    let a = config();
    let factory = ConfigFactory::new();
    let b = factory.generate(a.dimensions, &a.options, &a.colors, true);
    let mut c = factory.generate(a.dimensions, &a.options, &a.colors, true);
    assert!(b.colors.foreground.same_object(&a.colors.foreground));
    assert!(b.colors.ansi[0].same_object(&a.colors.ansi[0]));
    assert!(Rc::ptr_eq(
        &b.colors.contrast_cache,
        &a.colors.contrast_cache
    ));
    assert!(Rc::ptr_eq(
        &b.colors.half_contrast_cache,
        &a.colors.half_contrast_cache
    ));
    assert!(b.colors.cursor.same_object(&c.colors.cursor));
    assert!(b
        .colors
        .cursor
        .same_object(b.colors.selection_foreground.as_ref().unwrap()));
    c.colors.ansi[0] = Color::null().into();
    assert_eq!(b.colors.ansi[0].borrow().rgba, 0xffff_ffff);
    assert_eq!(a.colors.ansi[0].borrow().rgba, 0xffff_ffff);
}

#[test]
fn strict_number_and_string_semantics_and_packed_color_modes() {
    let mut a = config();
    let mut b = config();
    a.options.font_weight = FontWeight::Number(400.0);
    b.options.font_weight = FontWeight::Text(text("400"));
    assert!(!config_equals(&a, &b).unwrap());
    a.options.font_size = f64::NAN;
    assert!(!config_equals(&a, &a).unwrap());
    assert!(is_256_color(attributes::CM_P16 | 15));
    assert!(is_256_color(attributes::CM_P256 | 255));
    assert!(!is_256_color(attributes::CM_DEFAULT));
    assert!(!is_256_color(attributes::CM_RGB | 0xffff_ffff));
}
