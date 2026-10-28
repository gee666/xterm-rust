// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! UTF-16 fallback joining and CSS font-family parsing used by browser addons.

pub mod font;
pub mod parse;

use std::fmt;

pub const DEFAULT_FALLBACK_LIGATURES: &[&str] = &[
    "<--", "<---", "<<-", "<-", "->", "->>", "-->", "--->", "<==", "<===", "<<=", "<=", "=>",
    "=>>", "==>", "===>", ">=", ">>=", "<->", "<-->", "<--->", "<---->", "<=>", "<==>", "<===>",
    "<====>", "::", ":::", "<~~", "</", "</>", "/>", "~~>", "==", "!=", "/=", "~=", "<>", "===",
    "!==", "!===", "<:", ":=", "*=", "*+", "<*", "<*>", "*>", "<|", "<|>", "|>", "+*", "=*", "=:",
    ":>", "/*", "*/", "+++", "<!--", "<!---",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmptyLigature;

impl fmt::Display for EmptyLigature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Empty fallback ligatures would prevent the source matcher from advancing")
    }
}

impl std::error::Error for EmptyLigature {}

#[derive(Clone, Debug)]
pub struct FallbackLigatures {
    patterns: Vec<Vec<u16>>,
}

impl Default for FallbackLigatures {
    fn default() -> Self {
        let mut patterns: Vec<_> = DEFAULT_FALLBACK_LIGATURES
            .iter()
            .map(|pattern| pattern.encode_utf16().collect::<Vec<_>>())
            .collect();
        patterns.sort_by_key(|pattern| std::cmp::Reverse(pattern.len()));
        Self { patterns }
    }
}

impl FallbackLigatures {
    /// Empty lists disable fallback. Empty strings are rejected because upstream
    /// loops forever on a match of length zero, which cannot be a safe API.
    pub fn new(mut patterns: Vec<Vec<u16>>) -> Result<Self, EmptyLigature> {
        if patterns.iter().any(Vec::is_empty) {
            return Err(EmptyLigature);
        }
        patterns.sort_by_key(|pattern| std::cmp::Reverse(pattern.len()));
        Ok(Self { patterns })
    }

    /// Returned ranges are half-open UTF-16 offsets and never overlap.
    pub fn ranges(&self, text: &[u16]) -> Vec<[usize; 2]> {
        let mut ranges = Vec::new();
        let mut offset = 0;
        while offset < text.len() {
            if let Some(pattern) = self.patterns.iter().find(|p| text[offset..].starts_with(p)) {
                ranges.push([offset, offset + pattern.len()]);
                offset += pattern.len();
            } else {
                offset += 1;
            }
        }
        ranges
    }
}
