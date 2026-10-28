// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{
    font_bytes::FontError,
    font_tables::*,
    font_tree::{Arena, EntryId, TreeId},
};
use std::collections::HashSet;

pub struct EntryMeta {
    pub entry: EntryId,
    pub substitutions: Vec<Option<GlyphValue>>,
}

enum Replacement {
    MissingValues,
    NaN,
    Undefined,
}
enum SubView<'a> {
    Single(&'a SingleSubstitution),
    Other {
        coverage: Option<&'a Coverage>,
        replacement: Replacement,
    },
}
impl SubView<'_> {
    fn glyph(&self, glyph: Glyph) -> Result<Option<GlyphValue>, FontError> {
        match self {
            Self::Single(table) => Ok(table.glyph(glyph)),
            Self::Other {
                coverage,
                replacement,
            } => {
                let coverage =
                    coverage.ok_or(FontError::Invalid("Substitution has no coverage table"))?;
                if coverage.index(glyph).is_none() {
                    return Ok(None);
                }
                match replacement {
                    Replacement::MissingValues => {
                        Err(FontError::Invalid("Substitution has no substitute array"))
                    }
                    Replacement::NaN => Ok(Some(GlyphValue::NaN)),
                    Replacement::Undefined => Ok(Some(GlyphValue::Undefined)),
                }
            }
        }
    }
    fn range(
        &self,
        start: Glyph,
        end: Glyph,
    ) -> Result<Vec<(GlyphSet, Option<GlyphValue>)>, FontError> {
        match self {
            Self::Single(table) => table.range(start, end),
            Self::Other { .. } => source_segments(start, end, |glyph| self.glyph(glyph)),
        }
    }
}
fn subtable_count(lookup: &Lookup) -> usize {
    match lookup {
        Lookup::Single(tables) => tables.len(),
        Lookup::Chain(tables) => tables.len(),
        Lookup::Reverse(tables) => tables.len(),
        Lookup::Other(tables) => tables.len(),
    }
}
fn view(lookup: &Lookup, index: usize) -> SubView<'_> {
    // The caller obtains this index from the same lookup's subtable count.
    match lookup {
        Lookup::Single(tables) => SubView::Single(&tables[index]),
        Lookup::Chain(tables) => match &tables[index] {
            ChainTable::Glyphs { coverage, .. } => SubView::Other {
                coverage: Some(coverage),
                replacement: Replacement::NaN,
            },
            ChainTable::Classes { coverage, .. } => SubView::Other {
                coverage: Some(coverage),
                replacement: Replacement::MissingValues,
            },
            ChainTable::Coverage { .. } => SubView::Other {
                coverage: None,
                replacement: Replacement::Undefined,
            },
        },
        Lookup::Reverse(tables) => SubView::Other {
            coverage: Some(&tables[index].coverage),
            replacement: Replacement::NaN,
        },
        Lookup::Other(tables) => {
            let table = &tables[index];
            SubView::Other {
                coverage: table.coverage.as_ref(),
                replacement: match table.format {
                    1 => Replacement::NaN,
                    2 => Replacement::MissingValues,
                    _ => Replacement::Undefined,
                },
            }
        }
    }
}
fn substitution(
    records: &[LookupRecord],
    lookups: &[Lookup],
    position: usize,
    glyph: Glyph,
) -> Result<Option<GlyphValue>, FontError> {
    for record in records
        .iter()
        .filter(|record| record.sequence_index == position)
    {
        let lookup = lookups
            .get(record.lookup_index)
            .ok_or(FontError::Invalid("Missing GSUB substitution lookup"))?;
        for index in 0..subtable_count(lookup) {
            if let Some(value) = view(lookup, index).glyph(glyph)? {
                return Ok(Some(value));
            }
        }
    }
    Ok(None)
}
fn range_substitutions(
    records: &[LookupRecord],
    lookups: &[Lookup],
    position: usize,
    start: Glyph,
    end: Glyph,
) -> Result<Vec<(GlyphSet, Option<GlyphValue>)>, FontError> {
    for record in records
        .iter()
        .filter(|record| record.sequence_index == position)
    {
        let lookup = lookups
            .get(record.lookup_index)
            .ok_or(FontError::Invalid("Missing GSUB substitution lookup"))?;
        for index in 0..subtable_count(lookup) {
            let table = view(lookup, index);
            let replacements = table.range(start, end)?;
            // This intentionally matches the source's !every(val !== null),
            // not the more intuitive 'any substitution found' condition.
            if replacements.iter().any(|(_, value)| value.is_none()) {
                return Ok(replacements);
            }
        }
    }
    Ok(vec![(GlyphSet::Range(start, end), None)])
}
pub fn input_tree(
    arena: &mut Arena,
    tree: TreeId,
    records: &[LookupRecord],
    lookups: &[Lookup],
    position: usize,
    glyphs: GlyphSet,
) -> Result<Vec<EntryMeta>, FontError> {
    let mut output = Vec::new();
    match glyphs {
        GlyphSet::Individual(glyph) => {
            let entry = arena.entry()?;
            arena.set_individual(tree, glyph, entry)?;
            output.push(EntryMeta {
                entry,
                substitutions: vec![substitution(records, lookups, position, glyph)?],
            });
        }
        GlyphSet::Range(start, end) => {
            for (set, substitution) in range_substitutions(records, lookups, position, start, end)?
            {
                let entry = arena.entry()?;
                match set {
                    GlyphSet::Range(start, end) => arena.push_range(tree, start, end, entry)?,
                    GlyphSet::Individual(glyph) => {
                        // Source installs a separate empty object here, not the
                        // entry returned to the caller. Preserve that distinction.
                        let installed = arena.entry()?;
                        arena.set_individual(tree, glyph, installed)?;
                    }
                }
                output.try_reserve(1)?;
                output.push(EntryMeta {
                    entry,
                    substitutions: vec![substitution],
                });
            }
        }
    }
    Ok(output)
}
pub fn input_position(
    arena: &mut Arena,
    glyphs: &[GlyphSet],
    position: usize,
    entries: Vec<EntryMeta>,
    records: &[LookupRecord],
    lookups: &[Lookup],
) -> Result<Vec<EntryMeta>, FontError> {
    let mut next = Vec::new();
    for current in entries {
        let tree = arena.tree()?;
        arena.get_mut(current.entry).forward = Some(tree);
        for glyph in glyphs {
            for mut entry in input_tree(arena, tree, records, lookups, position, *glyph)? {
                let substitution = entry.substitutions.pop().flatten();
                entry
                    .substitutions
                    .try_reserve_exact(current.substitutions.len() + 1)?;
                entry
                    .substitutions
                    .extend_from_slice(&current.substitutions);
                entry.substitutions.push(substitution);
                next.try_reserve(1)?;
                next.push(entry);
            }
        }
    }
    Ok(next)
}
pub fn context_position(
    arena: &mut Arena,
    glyphs: &[GlyphSet],
    entries: Vec<EntryMeta>,
    reverse: bool,
) -> Result<Vec<EntryMeta>, FontError> {
    let mut next = Vec::new();
    let mut visited = HashSet::new();
    for current in entries {
        if visited.contains(&current.entry) {
            continue;
        }
        visited.try_reserve(1).map_err(|_| FontError::Capacity)?;
        visited.insert(current.entry);
        let tree = if reverse {
            arena.ensure_reverse(current.entry)?
        } else {
            arena.ensure_forward(current.entry)?
        };
        let shared = arena.entry()?;
        for glyph in glyphs {
            arena.insert_set(tree, *glyph, shared)?;
        }
        next.try_reserve(1)?;
        next.push(EntryMeta {
            entry: shared,
            substitutions: current.substitutions,
        });
    }
    Ok(next)
}
