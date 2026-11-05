// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT.

//! WebGL renderer components. Portable storage and layout algorithms are tested
//! without a browser; actual GPU integration has separate browser assertions.
//! This module is a partial port and does not yet export a working WebGL addon.

pub mod cache;
pub mod config;
pub mod math;
pub mod model;
pub mod rectangles;
pub mod typed_array;

#[cfg(target_arch = "wasm32")]
mod gpu_resources;
#[cfg(target_arch = "wasm32")]
pub mod rectangle_gpu;
