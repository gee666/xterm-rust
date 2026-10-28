// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Shared upstream atlas configuration fixture.
use std::rc::Rc;
use xterm_core::color::Color;
use xterm_web::{
    theme::ThemeService,
    webgl::config::{AtlasConfig, AtlasDimensions, AtlasOptions, ConfigFactory, FontWeight},
};

pub fn text(value: &str) -> Rc<[u16]> {
    value.encode_utf16().collect::<Vec<_>>().into()
}
pub fn config() -> AtlasConfig {
    let service = ThemeService::default();
    let mut colors = service.colors().borrow().clone();
    let white = Color {
        css: "#ffffff".into(),
        rgba: 0xffff_ffff,
    }
    .into();
    colors.foreground = white;
    colors.background = colors.foreground.clone();
    colors.ansi = vec![colors.foreground.clone(); 256];
    colors.half_contrast_cache = Rc::clone(&colors.contrast_cache);
    ConfigFactory::new().generate(
        AtlasDimensions {
            device_pixel_ratio: 1.0,
            device_max_texture_size: 4096.0,
            device_cell_width: 10.0,
            device_cell_height: 20.0,
            device_char_width: 8.0,
            device_char_height: 16.0,
        },
        &AtlasOptions {
            letter_spacing: 0.0,
            line_height: 1.0,
            font_family: text("monospace"),
            font_size: 15.0,
            font_weight: FontWeight::Text(text("normal")),
            font_weight_bold: FontWeight::Text(text("bold")),
            allow_transparency: false,
            draw_bold_text_in_bright_colors: true,
            minimum_contrast_ratio: 1.0,
        },
        &colors,
        true,
    )
}
