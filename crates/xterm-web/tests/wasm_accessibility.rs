#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Real DOM components with an explicitly controlled clock.
mod accessibility_support;
#[path = "accessibility_cases/announcements.rs"]
mod announcements;
#[path = "accessibility_cases/failures.rs"]
mod failures;
#[path = "accessibility_cases/focus.rs"]
mod focus;
#[path = "accessibility_cases/iframe.rs"]
mod iframe;
#[path = "accessibility_cases/selection.rs"]
mod selection;
#[path = "accessibility_cases/tree.rs"]
mod tree;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);
