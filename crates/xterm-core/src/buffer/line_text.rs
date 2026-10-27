// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::BufferLine;
pub(super) use crate::input::text_decoder::string_from_code_point as codepoint;

impl BufferLine {
    /// Translate cells to UTF-16. `out_columns` has one entry per code unit,
    /// plus the column after the result. A wide character may advance past end.
    /// Only full-line requests without a column map use the canonical cache.
    pub fn translate_to_string(
        &self,
        trim_right: bool,
        start_col: Option<usize>,
        end_col: Option<usize>,
        mut out_columns: Option<&mut Vec<usize>>,
    ) -> Vec<u16> {
        let canonical = start_col.unwrap_or(0) == 0 && end_col.is_none() && out_columns.is_none();
        if canonical {
            if let Some((value, trimmed)) = self.cache.borrow().as_ref() {
                if trim_right {
                    if *trimmed {
                        return value.clone();
                    }
                    let end = value
                        .iter()
                        .rposition(|&c| !js_whitespace(c))
                        .map_or(0, |i| i + 1);
                    return value[..end].to_vec();
                }
                if !trimmed {
                    return value.clone();
                }
            }
        }
        let mut col = start_col.unwrap_or(0);
        let mut end = end_col.unwrap_or(self.length);
        if trim_right {
            end = end.min(self.get_trimmed_length());
        }
        if let Some(columns) = &mut out_columns {
            columns.clear();
        }
        let mut result = Vec::new();
        while col < end {
            // JS out-of-bounds typed-array reads become zero under bitwise
            // operations, yielding a blank column rather than panicking.
            let chars = match self.get_string(col) {
                Some(s) if !s.is_empty() || self.is_combined(col).is_some_and(|flag| flag != 0) => {
                    s
                }
                _ => vec![32],
            };
            if let Some(columns) = &mut out_columns {
                columns.extend(std::iter::repeat_n(col, chars.len()));
            }
            result.extend(chars);
            col = col.saturating_add(self.get_width(col).unwrap_or(0).max(1) as usize);
        }
        if let Some(columns) = out_columns {
            columns.push(col);
        }
        if canonical {
            *self.cache.borrow_mut() = Some((result.clone(), trim_right));
        }
        result
    }
}

// ECMAScript WhiteSpace + LineTerminator, not Rust's Unicode White_Space set.
fn js_whitespace(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000..=0x200a
        | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{CellData, CharData};
    fn cell(s: &str) -> CellData {
        CellData::from_char_data(&CharData {
            attr: 1,
            chars: s.encode_utf16().collect(),
            width: 1,
            code: None,
        })
    }
    fn utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
    #[test]
    fn upstream_canonical_cache_contract() {
        let mut line = BufferLine::new(5, None, false);
        for (i, s) in ["a", "b", "c"].iter().enumerate() {
            line.set_cell(i, &cell(s));
        }
        assert_eq!(
            line.translate_to_string(true, None, None, None),
            utf16("abc")
        );
        assert_eq!(*line.cache.borrow(), Some((utf16("abc"), true)));
        assert_eq!(
            line.translate_to_string(false, None, None, None),
            utf16("abc  ")
        );
        assert_eq!(*line.cache.borrow(), Some((utf16("abc  "), false)));
        assert_eq!(
            line.translate_to_string(true, None, None, None),
            utf16("abc")
        );
        assert_eq!(*line.cache.borrow(), Some((utf16("abc  "), false)));
        *line.cache.borrow_mut() = Some((utf16("cached-non-trimmed  "), false));
        assert_eq!(
            line.translate_to_string(false, None, None, None),
            utf16("cached-non-trimmed  ")
        );
        assert_eq!(
            line.translate_to_string(true, None, None, None),
            utf16("cached-non-trimmed")
        );
        *line.cache.borrow_mut() = Some((utf16("cached-trimmed"), true));
        assert_eq!(
            line.translate_to_string(true, None, None, None),
            utf16("cached-trimmed")
        );
        assert_eq!(
            line.translate_to_string(false, None, None, None),
            utf16("abc  ")
        );
        for trim in [true, false] {
            assert_eq!(
                line.translate_to_string(trim, Some(0), Some(2), None),
                utf16("ab")
            );
        }
    }
    #[test]
    fn upstream_all_mutations_invalidate_cache() {
        for mutation in 0..11 {
            let fill = cell("b");
            let mut line = BufferLine::new(5, Some(&cell("a")), false);
            line.translate_to_string(true, None, None, None);
            assert_eq!(*line.cache.borrow(), Some((utf16("aaaaa"), true)));
            line.translate_to_string(false, None, None, None);
            assert_eq!(*line.cache.borrow(), Some((utf16("aaaaa"), false)));
            let source = BufferLine::new(5, Some(&cell("x")), false);
            match mutation {
                0 => line.set(0, &fill.get_as_char_data()),
                1 => line.set_cell(0, &fill),
                2 => line.set_cell_from_codepoint(0, 98, 1, &fill),
                3 => line.add_codepoint_to_cell(0, 0x301, 0),
                4 => line.insert_cells(1, 1, &fill),
                5 => line.delete_cells(1, 1, &fill),
                6 => line.replace_cells(1, 3, &fill, false),
                7 => {
                    line.resize(6, &fill);
                }
                8 => line.fill(&fill, false),
                9 => line.copy_from(&source, false),
                10 => line.copy_cells_from(&source, 0, 0, 2, false),
                _ => unreachable!(),
            }
            assert!(line.cache.borrow().is_none(), "mutation {mutation}");
        }
    }
    #[test]
    fn cached_trim_uses_javascript_whitespace() {
        for (cp, trim) in [(0xfeff, true), (0x85, false), (0xa0, true), (0x200b, false)] {
            let mut line = BufferLine::new(1, None, false);
            line.set_cell_from_codepoint(0, cp, 1, &cell("a"));
            let full = line.translate_to_string(false, None, None, None);
            assert_eq!(
                line.translate_to_string(true, None, None, None),
                if trim { vec![] } else { full }
            );
        }
    }
}
