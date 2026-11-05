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
    let list = pointer(root, 8)?;
    let count = list.map(|list| list.u16(0)).transpose()?.unwrap_or(0);
    let mut lookups = Vec::new();
    lookups.try_reserve_exact(usize::from(count))?;
    for index in 0..usize::from(count) {
        // Source follows each pointer before reading the next list element.
        // Null pointers survive at their original indices for FontImpl.
        let table = list
            .map(|list| pointer(list, 2 + index * 2))
            .transpose()?
            .flatten();
        let Some(table) = table else {
            lookups.push(None);
            continue;
        };
        let kind = table.u16(0)?;
        if !(1..=9).contains(&kind) {
            // The shared GPOS/GSUB parser rejects the type before flags/count.
            return Err(FontError::GposLookupType(kind));
        }
        let flags = table.u16(2)?;
        let count = usize::from(table.u16(4)?);
        if kind == 9 {
            // Source has no type9 parser. Null pointers skip parseStruct, but a
            // nonnull pointer fails on Object.keys(undefined) before any target
            // bytes or later offsets are read.
            for index in 0..count {
                if table.u16(6 + index * 2)? != 0 {
                    return Err(FontError::Type(
                        "Cannot convert undefined or null to object",
                    ));
                }
            }
            lookups.push(Some(Lookup::NullSubtables(count)));
        } else {
            let lookup = match kind {
                1 => Lookup::Single(subtables(table, count, other::single)?),
                6 => Lookup::Chain(subtables(table, count, chain::parse)?),
                8 => Lookup::Reverse(subtables(table, count, other::reverse)?),
                2..=5 | 7 => Lookup::Other(subtables(table, count, |table| {
                    other::generic(kind, table)
                })?),
                _ => return Err(FontError::UnsupportedFormat("GSUB lookup type", kind)),
            };
            lookups.push(Some(lookup));
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

fn subtables<T>(
    table: Reader<'_>,
    count: usize,
    parse: impl Fn(Reader<'_>) -> Result<T, FontError>,
) -> Result<Vec<Option<T>>, FontError> {
    let mut output = Vec::new();
    output.try_reserve_exact(count)?;
    for index in 0..count {
        // Each pointer is followed before reading the next offset. A null
        // pointer consumes no subtable bytes, including the format word.
        output.push(pointer(table, 6 + index * 2)?.map(&parse).transpose()?);
    }
    Ok(output)
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

fn selected_features(root: Reader<'_>) -> Result<Option<Vec<usize>>, FontError> {
    let mut selected = Some(Vec::new());
    if let Some(features) = pointer(root, 6)? {
        let count = usize::from(features.u16(0)?);
        for index in 0..count {
            let entry = 2 + index * 6;
            let tag = features.bytes(entry, 4)?;
            if let Some(table) = pointer(features, entry + 4)? {
                table.u16(0)?;
                let count = usize::from(table.u16(2)?);
                let indices = table.words(4, count)?;
                if tag == b"calt" {
                    if let Some(selected) = &mut selected {
                        selected.try_reserve(indices.len())?;
                        selected.extend(indices.into_iter().map(usize::from));
                    }
                }
            } else if tag == b"calt" {
                // Do not fail yet. The source parser accepts an absent feature;
                // FontImpl dereferences it only after every constructor table.
                selected = None;
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
