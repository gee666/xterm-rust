// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::font_tables::{interval, Glyph, GlyphSet};
use std::cmp::Ordering;

pub struct Overlap {
    pub first: Vec<GlyphSet>,
    pub second: Vec<GlyphSet>,
    pub both: Option<GlyphSet>,
}
pub fn ranges(first: [Glyph; 2], second: [Glyph; 2]) -> Overlap {
    let mut result = Overlap {
        first: Vec::new(),
        second: Vec::new(),
        both: None,
    };
    if first[0] < second[1] && second[0] < first[1] {
        result.both = Some(interval(first[0].max(second[0]), first[1].min(second[1])));
    }
    match first[0].cmp(&second[0]) {
        Ordering::Less => result
            .first
            .push(interval(first[0], second[0].min(first[1]))),
        Ordering::Greater => result
            .second
            .push(interval(second[0], first[0].min(second[1]))),
        Ordering::Equal => {}
    }
    match first[1].cmp(&second[1]) {
        Ordering::Greater => result
            .first
            .push(interval(first[0].max(second[1]), first[1])),
        Ordering::Less => result
            .second
            .push(interval(first[1].max(second[0]), second[1])),
        Ordering::Equal => {}
    }
    result
}
pub fn individual(first: Glyph, second: [Glyph; 2]) -> Overlap {
    // The strict '>' at the right edge is an observable source quirk.
    if first < second[0] || first > second[1] {
        return Overlap {
            first: vec![GlyphSet::Individual(first)],
            second: vec![GlyphSet::Range(second[0], second[1])],
            both: None,
        };
    }
    let mut result = Overlap {
        first: Vec::new(),
        second: Vec::new(),
        both: Some(GlyphSet::Individual(first)),
    };
    if second[0] < first {
        result.second.push(interval(second[0], first));
    }
    if second[1] > first {
        result.second.push(interval(first + 1, second[1]));
    }
    result
}
