// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT.

//! Kitty graphics protocol parsing and state, separate from browser rendering.

pub mod command;
pub mod geometry;
pub mod handler;
mod handler_types;
pub mod pixels;
pub mod storage;
