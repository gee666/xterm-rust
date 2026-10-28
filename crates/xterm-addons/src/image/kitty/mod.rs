// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT.

//! Kitty graphics protocol parsing and state, separate from browser rendering.

pub mod command;
pub mod storage;
pub mod pixels;
pub mod geometry;
mod handler_types;
pub mod handler;
