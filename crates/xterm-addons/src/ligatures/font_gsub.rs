// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Binary layout parsing also follows opentype.js, see licenses/opentype-js-MIT.txt.
use super::{
    font_bytes::{FontError, Reader},
    font_tables::*,
};
#[path = "font_gsub_chain.rs"]
mod chain;
#[path = "font_gsub_other.rs"]
mod other;

pub fn parse(bytes: &[u8]) -> Result<Gsub, FontError> {
    let root = Reader::new(bytes);
    let major = root.u16(0)?;
    let minor = root.u16(2)?;
    if major != 1 || minor > 1 {
        return Err(FontError::UnsupportedFormat("GSUB version", major));
    }
    validate_scripts(root.child16(4)?)?;
    let features = root.child16(6)?;
    let feature_count = usize::from(features.u16(0)?);
    features.bytes(2, feature_count * 6)?;
    let mut selected = Vec::new();
    for index in 0..feature_count {
        let entry = 2 + index * 6;
        let table = features.child16(entry + 4)?;
        let count = usize::from(table.u16(2)?);
        let indices = table.words(4, count)?;
        if features.bytes(entry, 4)? == b"calt" {
            selected.try_reserve(indices.len())?;
            selected.extend(indices.into_iter().map(usize::from));
        }
    }
    let list = root.child16(8)?;
    let count = usize::from(list.u16(0)?);
    let offsets = list.words(2, count)?;
    let mut lookups = Vec::new();
    lookups.try_reserve_exact(count)?;
    for offset in offsets {
        let table = list.at(usize::from(offset))?;
        let kind = table.u16(0)?;
        let flags = table.u16(2)?;
        let count = usize::from(table.u16(4)?);
        let offsets = table.words(6, count)?;
        if flags & 0x0010 != 0 {
            table.u16(6 + count * 2)?;
        }
        match kind {
            1 => {
                let mut subtables = Vec::new();
                subtables.try_reserve_exact(count)?;
                for offset in offsets {
                    subtables.push(other::single(table.at(usize::from(offset))?)?);
                }
                lookups.push(Lookup::Single(subtables));
            }
            6 => {
                let mut subtables = Vec::new();
                subtables.try_reserve_exact(count)?;
                for offset in offsets {
                    subtables.push(chain::parse(table.at(usize::from(offset))?)?);
                }
                lookups.push(Lookup::Chain(subtables));
            }
            8 => {
                let mut subtables = Vec::new();
                subtables.try_reserve_exact(count)?;
                for offset in offsets {
                    subtables.push(other::reverse(table.at(usize::from(offset))?)?);
                }
                lookups.push(Lookup::Reverse(subtables));
            }
            2..=5 | 7 => {
                let mut subtables = Vec::new();
                subtables.try_reserve_exact(count)?;
                for offset in offsets {
                    subtables.push(other::generic(kind, table.at(usize::from(offset))?)?);
                }
                lookups.push(Lookup::Other(subtables));
            }
            _ => return Err(FontError::UnsupportedFormat("GSUB lookup type", kind)),
        }
    }
    if minor == 1 {
        let variations = root.offset32(10)?;
        if variations != 0 {
            validate_feature_variations(root.at(variations)?)?;
        }
    }
    Ok(Gsub { selected, lookups })
}

pub(super) fn glyphs(
    table: Reader<'_>,
    offset: usize,
    count: usize,
) -> Result<Vec<Glyph>, FontError> {
    let input = table.words(offset, count)?;
    let mut output = Vec::new();
    output.try_reserve_exact(count)?;
    output.extend(input.into_iter().map(Glyph::from));
    Ok(output)
}

pub(super) fn coverage(table: Reader<'_>) -> Result<Coverage, FontError> {
    let count = usize::from(table.u16(2)?);
    match table.u16(0)? {
        1 => Ok(Coverage::Glyphs(glyphs(table, 4, count)?)),
        2 => {
            table.bytes(4, count * 6)?;
            let mut ranges = Vec::new();
            ranges.try_reserve_exact(count)?;
            for index in 0..count {
                let offset = 4 + index * 6;
                ranges.push(CoverageRange {
                    start: Glyph::from(table.u16(offset)?),
                    end: Glyph::from(table.u16(offset + 2)?),
                    index: usize::from(table.u16(offset + 4)?),
                });
            }
            Ok(Coverage::Ranges(ranges))
        }
        format => Err(FontError::UnsupportedFormat("GSUB coverage", format)),
    }
}

pub(super) fn class_def(table: Reader<'_>) -> Result<ClassDef, FontError> {
    match table.u16(0)? {
        1 => {
            let count = usize::from(table.u16(4)?);
            table.words(6, count)?;
            Ok(ClassDef::Other)
        }
        2 => {
            let count = usize::from(table.u16(2)?);
            table.bytes(4, count * 6)?;
            let mut ranges = Vec::new();
            ranges.try_reserve_exact(count)?;
            for index in 0..count {
                let offset = 4 + index * 6;
                ranges.push(ClassRange {
                    start: Glyph::from(table.u16(offset)?),
                    end: Glyph::from(table.u16(offset + 2)?),
                    class: usize::from(table.u16(offset + 4)?),
                });
            }
            Ok(ClassDef::Ranges(ranges))
        }
        format => Err(FontError::UnsupportedFormat(
            "GSUB class definition",
            format,
        )),
    }
}

pub(super) fn records(
    table: Reader<'_>,
    offset: usize,
    count: usize,
) -> Result<Vec<LookupRecord>, FontError> {
    table.bytes(offset, count * 4)?;
    let mut records = Vec::new();
    records.try_reserve_exact(count)?;
    for index in 0..count {
        records.push(LookupRecord {
            sequence_index: usize::from(table.u16(offset + index * 4)?),
            lookup_index: usize::from(table.u16(offset + index * 4 + 2)?),
        });
    }
    Ok(records)
}

fn validate_scripts(table: Reader<'_>) -> Result<(), FontError> {
    let count = usize::from(table.u16(0)?);
    table.bytes(2, count * 6)?;
    for index in 0..count {
        let script = table.child16(6 + index * 6)?;
        let default = script.u16(0)?;
        if default != 0 {
            validate_language(script.at(usize::from(default))?)?;
        }
        let count = usize::from(script.u16(2)?);
        script.bytes(4, count * 6)?;
        for index in 0..count {
            validate_language(script.child16(8 + index * 6)?)?;
        }
    }
    Ok(())
}
fn validate_language(table: Reader<'_>) -> Result<(), FontError> {
    table.u16(0)?;
    table.u16(2)?;
    table.words(6, usize::from(table.u16(4)?))?;
    Ok(())
}
fn validate_feature_variations(table: Reader<'_>) -> Result<(), FontError> {
    // The ligature engine does not apply variation records, but the source
    // parser still reads their offsets, conditions and feature substitutions.
    table.u16(0)?;
    table.u16(2)?;
    let count = table.offset32(4)?;
    table.bytes(8, count.checked_mul(8).ok_or(FontError::Bounds)?)?;
    for index in 0..count {
        let conditions = table.at(table.offset32(8 + index * 8)?)?;
        let condition_count = usize::from(conditions.u16(0)?);
        conditions.bytes(2, condition_count * 4)?;
        for index in 0..condition_count {
            let condition = conditions.at(conditions.offset32(2 + index * 4)?)?;
            if condition.u16(0)? != 1 {
                return Err(FontError::Invalid("Unsupported GSUB variation condition"));
            }
            condition.bytes(2, 6)?;
        }
        let substitutions = table.at(table.offset32(12 + index * 8)?)?;
        substitutions.u16(0)?;
        substitutions.u16(2)?;
        let count = usize::from(substitutions.u16(4)?);
        substitutions.bytes(6, count * 6)?;
        for index in 0..count {
            let feature = substitutions.at(substitutions.offset32(8 + index * 6)?)?;
            feature.words(4, usize::from(feature.u16(2)?))?;
        }
    }
    Ok(())
}
