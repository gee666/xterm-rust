// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::buffer::{CellData, CharData};
pub fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
pub fn cell(attr: u32, text: &str, width: i32) -> CellData {
    CellData::from_char_data(&CharData {
        attr,
        chars: utf16(text),
        width,
        code: None,
    })
}
