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
    // parseVersion(1) uses major + minor / 10, not a fixed-point version.
    let version = u32::from(major) * 10 + u32::from(minor);
    if !matches!(version, 10 | 11) {
        return Err(FontError::Invalid("Unsupported GSUB table version."));
    }
    if let Some(scripts) = pointer(root, 4)? {
        validate_scripts(scripts)?;
    }
    let selected = selected_features(root)?;
    let offsets = if let Some(list) = pointer(root, 8)? {
        let count = usize::from(list.u16(0)?);
        list.words(2, count)?
    } else {
        Vec::new()
    };
    let list = root.child16(8)?;
    let count = offsets.len();
    let mut lookups = Vec::new();
    lookups.try_reserve_exact(count)?;
    for offset in offsets {
        let table = list.at(usize::from(offset))?;
        let kind = table.u16(0)?;
        if !(1..=9).contains(&kind) {
            // The shared GPOS/GSUB parser rejects the type before flags/count.
            return Err(FontError::GposLookupType(kind));
        }
        let flags = table.u16(2)?;
        let count = usize::from(table.u16(4)?);
        let offsets = table.words(6, count)?;
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
        // Subtable errors precede the optional trailing mark-filter word.
        if flags & 0x0010 != 0 {
            table.u16(6 + count * 2)?;
        }
    }
    if version == 11 {
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

fn pointer(table: Reader<'_>, field: usize) -> Result<Option<Reader<'_>>, FontError> {
    let offset = table.u16(field)?;
    (offset != 0)
        .then(|| table.at(usize::from(offset)))
        .transpose()
}

fn selected_features(root: Reader<'_>) -> Result<Vec<usize>, FontError> {
    let mut selected = Vec::new();
    if let Some(features) = pointer(root, 6)? {
        let count = usize::from(features.u16(0)?);
        for index in 0..count {
            let entry = 2 + index * 6;
            let tag = features.bytes(entry, 4)?;
            let table = features.child16(entry + 4)?;
            table.u16(0)?;
            let count = usize::from(table.u16(2)?);
            let indices = table.words(4, count)?;
            if tag == b"calt" {
                selected.try_reserve(indices.len())?;
                selected.extend(indices.into_iter().map(usize::from));
            }
        }
    }
    Ok(selected)
}

fn validate_scripts(table: Reader<'_>) -> Result<(), FontError> {
    let count = usize::from(table.u16(0)?);
    table.bytes(2, count * 6)?;
    for index in 0..count {
        if let Some(script) = pointer(table, 6 + index * 6)? {
            if let Some(default) = pointer(script, 0)? {
                validate_language(default)?;
            }
            let count = usize::from(script.u16(2)?);
            script.bytes(4, count * 6)?;
            for index in 0..count {
                if let Some(language) = pointer(script, 8 + index * 6)? {
                    validate_language(language)?;
                }
            }
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
    let major = table.u16(0)?;
    let minor = table.u16(2)?;
    if major != 1 || minor != 0 {
        return Err(FontError::Invalid(
            "GPOS/GSUB feature variations table unknown.",
        ));
    }
    let count = table.offset32(4)?;
    // opentype retains these offsets but never follows conditions or feature
    // substitutions. Invalid targets must not reject an otherwise valid font.
    table.bytes(8, count.checked_mul(8).ok_or(FontError::Bounds)?)?;
    Ok(())
}
