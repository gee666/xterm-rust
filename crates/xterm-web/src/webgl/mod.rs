// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT.

//! WebGL renderer components. Portable storage and layout algorithms are tested
//! without a browser; actual GPU integration has separate browser assertions.
//! This module is a partial port and does not yet export a working WebGL addon.

#[cfg(target_arch = "wasm32")]
pub mod atlas_canvas;
pub mod atlas_colors;
#[cfg(target_arch = "wasm32")]
mod atlas_context;
#[cfg(target_arch = "wasm32")]
mod atlas_pack;
#[cfg(target_arch = "wasm32")]
pub mod atlas_page;
pub mod atlas_pixels;
#[cfg(target_arch = "wasm32")]
mod atlas_rasterizer;
#[cfg(target_arch = "wasm32")]
mod atlas_store;
#[cfg(target_arch = "wasm32")]
mod atlas_styles;
#[cfg(target_arch = "wasm32")]
pub mod browser_cache;
pub mod cache;
pub mod cell_colors;
pub mod config;
#[cfg(target_arch = "wasm32")]
pub mod cursor_blink;
pub mod custom_glyphs;
#[cfg(target_arch = "wasm32")]
pub mod device_pixel_observer;
pub mod glyphs;
pub mod math;
pub mod model;
pub mod rectangles;
pub mod renderer_frame;
pub mod renderer_types;
pub mod renderer_update;
#[cfg(target_arch = "wasm32")]
pub mod texture_atlas;
pub mod typed_array;

#[cfg(target_arch = "wasm32")]
pub mod glyph_atlas;
#[cfg(target_arch = "wasm32")]
pub mod glyph_gpu;
#[cfg(target_arch = "wasm32")]
mod glyph_shaders;
#[cfg(target_arch = "wasm32")]
mod gpu_resources;
#[cfg(target_arch = "wasm32")]
pub mod rectangle_gpu;
