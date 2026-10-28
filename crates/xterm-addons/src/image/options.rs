// Copyright (c) 2020 The xterm.js authors. MIT license.
//! Native image configuration. Raw JS option/coercion boundaries are task205.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageOptions {
    pub enable_size_reports: bool,
    pub pixel_limit: f64,
    pub storage_limit: f64,
    pub show_placeholder: bool,
    pub sixel_support: bool,
    pub sixel_scrolling: bool,
    pub sixel_palette_limit: f64,
    pub sixel_size_limit: f64,
    pub iip_support: bool,
    pub iip_size_limit: f64,
    pub kitty_support: bool,
    pub kitty_size_limit: f64,
}
impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            enable_size_reports: true,
            pixel_limit: 16_777_216.0,
            storage_limit: 128.0,
            show_placeholder: true,
            sixel_support: true,
            sixel_scrolling: true,
            sixel_palette_limit: 4096.0,
            sixel_size_limit: 33_554_432.0,
            iip_support: true,
            iip_size_limit: 33_554_432.0,
            kitty_support: true,
            kitty_size_limit: 33_554_432.0,
        }
    }
}
