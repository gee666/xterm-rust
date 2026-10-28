// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Tables consumed by the source ligature algorithm, not a general shaper.

use super::font_bytes::FontError;

pub type Glyph = i64;

/// Malformed or non-type1 referenced substitutions can expose these JavaScript
/// values through the source's unchecked table cast. Keep them observable.
#[derive(Clone, Copy, Debug)]
pub enum GlyphValue {
    Number(Glyph),
    NaN,
    Undefined,
}
impl PartialEq for GlyphValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Undefined, Self::Undefined) => true,
            _ => false,
        }
    }
}
impl GlyphValue {
    pub fn number(self) -> Option<Glyph> {
        if let Self::Number(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlyphSet {
    Individual(Glyph),
    Range(Glyph, Glyph),
}

#[derive(Clone, Debug)]
pub struct CoverageRange {
    pub start: Glyph,
    pub end: Glyph,
    pub index: usize,
}

#[derive(Clone, Debug)]
pub enum Coverage {
    Glyphs(Vec<Glyph>),
    Ranges(Vec<CoverageRange>),
}

impl Coverage {
    pub fn index(&self, glyph: Glyph) -> Option<usize> {
        match self {
            Self::Glyphs(glyphs) => glyphs.iter().position(|value| *value == glyph),
            // The reference returns range.index, not an offset within the range.
            Self::Ranges(ranges) => ranges
                .iter()
                .find(|range| range.start <= glyph && glyph <= range.end)
                .map(|range| range.index),
        }
    }

    pub fn entries(&self) -> Vec<(GlyphSet, usize)> {
        match self {
            Self::Glyphs(glyphs) => glyphs
                .iter()
                .copied()
                .enumerate()
                .map(|(index, glyph)| (GlyphSet::Individual(glyph), index))
                .collect(),
            Self::Ranges(ranges) => ranges
                .iter()
                .enumerate()
                .map(|(index, range)| (interval(range.start, range.end + 1), index))
                .collect(),
        }
    }
}

pub fn interval(start: Glyph, end: Glyph) -> GlyphSet {
    if end - start == 1 {
        GlyphSet::Individual(start)
    } else {
        GlyphSet::Range(start, end)
    }
}

#[derive(Clone, Debug)]
pub struct ClassRange {
    pub start: Glyph,
    pub end: Glyph,
    pub class: usize,
}

#[derive(Clone, Debug)]
pub enum ClassDef {
    Other,
    Ranges(Vec<ClassRange>),
}

impl ClassDef {
    pub fn class(&self, glyph: Glyph) -> Option<usize> {
        match self {
            Self::Other => None,
            Self::Ranges(ranges) => ranges
                .iter()
                .find(|range| range.start <= glyph && glyph <= range.end)
                .map(|range| range.class),
        }
    }

    pub fn members(&self, class: usize) -> Vec<GlyphSet> {
        match self {
            Self::Other => Vec::new(),
            Self::Ranges(ranges) => ranges
                .iter()
                .filter(|range| range.class == class)
                .map(|range| interval(range.start, range.end + 1))
                .collect(),
        }
    }

    pub fn classes(&self, glyphs: GlyphSet) -> Result<Vec<(GlyphSet, Option<usize>)>, FontError> {
        match (self, glyphs) {
            (Self::Other, _) => Ok(vec![(glyphs, None)]),
            (_, GlyphSet::Individual(glyph)) => Ok(vec![(glyphs, self.class(glyph))]),
            (_, GlyphSet::Range(start, end)) => {
                source_segments(start, end, |glyph| Ok(self.class(glyph)))
            }
        }
    }
}

/// Source Maps distinguish array keys by identity, but replace numeric keys.
/// Its range segmentation deliberately keeps the initial start and value.
pub(super) fn source_segments<T: Copy + PartialEq>(
    start: Glyph,
    end: Glyph,
    mut value: impl FnMut(Glyph) -> Result<T, FontError>,
) -> Result<Vec<(GlyphSet, T)>, FontError> {
    let initial = value(start)?;
    let mut output = Vec::new();
    let mut search = start + 1;
    while search < end {
        if value(search)? != initial {
            put_segment(&mut output, interval(start, search), initial)?;
        }
        search += 1;
    }
    put_segment(&mut output, interval(start, search), initial)?;
    Ok(output)
}

fn put_segment<T>(
    output: &mut Vec<(GlyphSet, T)>,
    key: GlyphSet,
    value: T,
) -> Result<(), FontError> {
    if matches!(key, GlyphSet::Individual(_)) {
        if let Some((_, existing)) = output.iter_mut().find(|(stored, _)| *stored == key) {
            *existing = value;
            return Ok(());
        }
    }
    output.try_reserve(1)?;
    output.push((key, value));
    Ok(())
}

#[derive(Clone, Debug)]
pub enum SingleSubstitution {
    Delta {
        coverage: Coverage,
        delta: i16,
    },
    Substitute {
        coverage: Coverage,
        glyphs: Vec<Glyph>,
    },
}

impl SingleSubstitution {
    pub fn glyph(&self, glyph: Glyph) -> Option<GlyphValue> {
        match self {
            Self::Delta { coverage, delta } => coverage
                .index(glyph)
                .map(|_| GlyphValue::Number((glyph + i64::from(*delta)) % 65536)),
            Self::Substitute { coverage, glyphs } => coverage
                .index(glyph)
                .and_then(|index| glyphs.get(index).copied())
                .map(GlyphValue::Number),
        }
    }

    pub fn range(
        &self,
        start: Glyph,
        end: Glyph,
    ) -> Result<Vec<(GlyphSet, Option<GlyphValue>)>, FontError> {
        source_segments(start, end, |glyph| Ok(self.glyph(glyph)))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct LookupRecord {
    pub sequence_index: usize,
    pub lookup_index: usize,
}

#[derive(Clone, Debug)]
pub struct ChainRule {
    pub backtrack: Vec<Glyph>,
    pub input: Vec<Glyph>,
    pub lookahead: Vec<Glyph>,
    pub lookups: Vec<LookupRecord>,
}

#[derive(Clone, Debug)]
pub enum ChainTable {
    Glyphs {
        coverage: Coverage,
        rules: Vec<Option<Vec<ChainRule>>>,
    },
    Classes {
        coverage: Coverage,
        backtrack: ClassDef,
        input: ClassDef,
        lookahead: ClassDef,
        rules: Vec<Option<Vec<ChainRule>>>,
    },
    Coverage {
        backtrack: Vec<Coverage>,
        input: Vec<Coverage>,
        lookahead: Vec<Coverage>,
        lookups: Vec<LookupRecord>,
    },
}

#[derive(Clone, Debug)]
pub struct ReverseTable {
    pub coverage: Coverage,
    pub backtrack: Vec<Coverage>,
    pub lookahead: Vec<Coverage>,
    pub substitutes: Vec<Glyph>,
}

#[derive(Clone, Debug)]
pub enum Lookup {
    Single(Vec<SingleSubstitution>),
    Chain(Vec<ChainTable>),
    Reverse(Vec<ReverseTable>),
    Other(Vec<GenericTable>),
}

#[derive(Clone, Debug)]
pub struct GenericTable {
    pub format: u16,
    pub coverage: Option<Coverage>,
}

#[derive(Clone, Debug, Default)]
pub struct Gsub {
    pub selected: Vec<usize>,
    pub lookups: Vec<Lookup>,
}
