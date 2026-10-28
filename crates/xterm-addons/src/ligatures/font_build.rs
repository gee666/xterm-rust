// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{
    font_build_helpers::{context_position, input_position, input_tree, EntryMeta},
    font_bytes::FontError,
    font_merge::merge,
    font_tables::*,
    font_tree::{Arena, LookupResult, TreeId},
};

struct Pattern {
    first: GlyphSet,
    input: Vec<Vec<GlyphSet>>,
    lookahead: Vec<Vec<GlyphSet>>,
    backtrack: Vec<Vec<GlyphSet>>,
}
struct ResultSpec {
    length: usize,
    backtrack: usize,
    lookahead: usize,
    index: usize,
    sub_index: usize,
}
struct RuleContext<'a> {
    records: &'a [LookupRecord],
    lookups: &'a [Lookup],
    index: usize,
    sub_index: usize,
}
fn from_glyphs(glyphs: &[Glyph]) -> Vec<Vec<GlyphSet>> {
    glyphs
        .iter()
        .map(|glyph| vec![GlyphSet::Individual(*glyph)])
        .collect()
}
fn from_classes(glyphs: &[Glyph], definition: &ClassDef) -> Result<Vec<Vec<GlyphSet>>, FontError> {
    let mut output = Vec::new();
    output.try_reserve_exact(glyphs.len())?;
    for glyph in glyphs {
        output.push(definition.members(
            usize::try_from(*glyph).map_err(|_| FontError::Invalid("Negative GSUB class"))?,
        ));
    }
    Ok(output)
}
fn from_coverage(coverages: &[Coverage]) -> Vec<Vec<GlyphSet>> {
    coverages
        .iter()
        .map(|coverage| {
            coverage
                .entries()
                .into_iter()
                .map(|(glyph, _)| glyph)
                .collect()
        })
        .collect()
}
fn build_rule(
    arena: &mut Arena,
    tree: TreeId,
    pattern: Pattern,
    context: RuleContext<'_>,
) -> Result<(), FontError> {
    let mut entries = input_tree(
        arena,
        tree,
        context.records,
        context.lookups,
        0,
        pattern.first,
    )?;
    for (index, glyphs) in pattern.input.iter().enumerate() {
        entries = input_position(
            arena,
            glyphs,
            index + 1,
            entries,
            context.records,
            context.lookups,
        )?;
    }
    for glyphs in &pattern.lookahead {
        entries = context_position(arena, glyphs, entries, false)?;
    }
    for glyphs in &pattern.backtrack {
        entries = context_position(arena, glyphs, entries, true)?;
    }
    finish(
        arena,
        entries,
        ResultSpec {
            length: pattern.input.len() + 1,
            backtrack: pattern.backtrack.len(),
            lookahead: pattern.lookahead.len(),
            index: context.index,
            sub_index: context.sub_index,
        },
    )
}
fn finish(arena: &mut Arena, entries: Vec<EntryMeta>, spec: ResultSpec) -> Result<(), FontError> {
    let begin = -i64::try_from(spec.backtrack).map_err(|_| FontError::Bounds)?;
    let end = i64::try_from(
        spec.length
            .checked_add(spec.lookahead)
            .ok_or(FontError::Bounds)?,
    )
    .map_err(|_| FontError::Bounds)?;
    for entry in entries {
        let lookup = arena.lookup(LookupResult {
            substitutions: entry.substitutions,
            length: spec.length,
            index: spec.index,
            sub_index: spec.sub_index,
            context_range: [begin, end],
        })?;
        arena.get_mut(entry.entry).lookup = Some(lookup);
    }
    Ok(())
}

pub fn chain(
    arena: &mut Arena,
    table: &ChainTable,
    lookups: &[Lookup],
    index: usize,
) -> Result<TreeId, FontError> {
    match table {
        ChainTable::Glyphs { coverage, rules } => {
            let tree = arena.tree()?;
            for (first, coverage_index) in coverage.entries() {
                let Some(Some(rules)) = rules.get(coverage_index) else {
                    continue;
                };
                for (sub_index, rule) in rules.iter().enumerate() {
                    let pattern = Pattern {
                        first,
                        input: from_glyphs(&rule.input),
                        lookahead: from_glyphs(&rule.lookahead),
                        backtrack: from_glyphs(&rule.backtrack),
                    };
                    build_rule(
                        arena,
                        tree,
                        pattern,
                        RuleContext {
                            records: &rule.lookups,
                            lookups,
                            index,
                            sub_index,
                        },
                    )?;
                }
            }
            Ok(tree)
        }
        ChainTable::Classes {
            coverage,
            backtrack,
            input,
            lookahead,
            rules,
        } => {
            let mut trees = Vec::new();
            for (first, _) in coverage.entries() {
                for (first, class) in input.classes(first)? {
                    let Some(class) = class else {
                        continue;
                    };
                    let Some(Some(rules)) = rules.get(class) else {
                        continue;
                    };
                    for (sub_index, rule) in rules.iter().enumerate() {
                        let tree = arena.tree()?;
                        let pattern = Pattern {
                            first,
                            input: from_classes(&rule.input, input)?,
                            lookahead: from_classes(&rule.lookahead, lookahead)?,
                            backtrack: from_classes(&rule.backtrack, backtrack)?,
                        };
                        build_rule(
                            arena,
                            tree,
                            pattern,
                            RuleContext {
                                records: &rule.lookups,
                                lookups,
                                index,
                                sub_index,
                            },
                        )?;
                        trees.try_reserve(1)?;
                        trees.push(tree);
                    }
                }
            }
            merge(arena, &trees)
        }
        ChainTable::Coverage {
            backtrack,
            input,
            lookahead,
            lookups: records,
        } => {
            let first = input
                .first()
                .ok_or(FontError::Invalid("GSUB chain has no input coverage"))?;
            let tree = arena.tree()?;
            for (first, _) in first.entries() {
                let pattern = Pattern {
                    first,
                    input: from_coverage(&input[1..]),
                    lookahead: from_coverage(lookahead),
                    backtrack: from_coverage(backtrack),
                };
                build_rule(
                    arena,
                    tree,
                    pattern,
                    RuleContext {
                        records,
                        lookups,
                        index,
                        sub_index: 0,
                    },
                )?;
            }
            Ok(tree)
        }
    }
}

pub fn reverse(arena: &mut Arena, table: &ReverseTable, index: usize) -> Result<TreeId, FontError> {
    let tree = arena.tree()?;
    for (glyphs, coverage_index) in table.coverage.entries() {
        let initial = arena.entry()?;
        arena.insert_set(tree, glyphs, initial)?;
        let substitution = table
            .substitutes
            .get(coverage_index)
            .copied()
            .map_or(GlyphValue::Undefined, GlyphValue::Number);
        let mut entries = vec![EntryMeta {
            entry: initial,
            substitutions: vec![Some(substitution)],
        }];
        for glyphs in from_coverage(&table.lookahead) {
            entries = context_position(arena, &glyphs, entries, false)?;
        }
        for glyphs in from_coverage(&table.backtrack) {
            entries = context_position(arena, &glyphs, entries, true)?;
        }
        finish(
            arena,
            entries,
            ResultSpec {
                length: 1,
                backtrack: table.backtrack.len(),
                lookahead: table.lookahead.len(),
                index,
                sub_index: 0,
            },
        )?;
    }
    Ok(tree)
}
