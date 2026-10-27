// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/ColorContrastCache.ts.
use crate::theme::ColorRef;
use std::collections::HashMap;

/// Outer Option means missing; an explicitly cached None is upstream's null.
#[derive(Default, Debug)]
pub struct ColorContrastCache {
    colors: HashMap<(u32, u32), Option<ColorRef>>,
    css: HashMap<(u32, u32), Option<String>>,
}
impl ColorContrastCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_css(&mut self, bg: u32, fg: u32, value: Option<String>) {
        self.css.insert((bg, fg), value);
    }
    pub fn get_css(&self, bg: u32, fg: u32) -> Option<&Option<String>> {
        self.css.get(&(bg, fg))
    }
    pub fn set_color(&mut self, bg: u32, fg: u32, value: Option<ColorRef>) {
        self.colors.insert((bg, fg), value);
    }
    pub fn get_color(&self, bg: u32, fg: u32) -> Option<&Option<ColorRef>> {
        self.colors.get(&(bg, fg))
    }
    pub fn clear(&mut self) {
        self.colors.clear();
        self.css.clear();
    }
}
