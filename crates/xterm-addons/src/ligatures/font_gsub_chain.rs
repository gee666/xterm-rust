// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::*;

pub(super) fn parse(table: Reader<'_>) -> Result<ChainTable, FontError> {
    match table.u16(0)? {
        1 => {
            let coverage = coverage(table.child16(2)?)?;
            let count = usize::from(table.u16(4)?);
            let rules = rule_sets(table, 6, count)?;
            Ok(ChainTable::Glyphs { coverage, rules })
        }
        2 => {
            let coverage = coverage(table.child16(2)?)?;
            let backtrack = class_def(table.child16(4)?)?;
            let input = class_def(table.child16(6)?)?;
            let lookahead = class_def(table.child16(8)?)?;
            let count = usize::from(table.u16(10)?);
            let rules = rule_sets(table, 12, count)?;
            Ok(ChainTable::Classes {
                coverage,
                backtrack,
                input,
                lookahead,
                rules,
            })
        }
        3 => {
            let mut position = 2;
            let backtrack = coverages(table, &mut position)?;
            let input = coverages(table, &mut position)?;
            let lookahead = coverages(table, &mut position)?;
            let count = usize::from(table.u16(position)?);
            let lookups = records(table, position + 2, count)?;
            Ok(ChainTable::Coverage {
                backtrack,
                input,
                lookahead,
                lookups,
            })
        }
        format => Err(FontError::UnsupportedFormat("GSUB chaining", format)),
    }
}

fn rule_sets(
    table: Reader<'_>,
    position: usize,
    count: usize,
) -> Result<Vec<Option<Vec<ChainRule>>>, FontError> {
    let offsets = table.words(position, count)?;
    let mut sets = Vec::new();
    sets.try_reserve_exact(count)?;
    for offset in offsets {
        if offset == 0 {
            sets.push(None);
            continue;
        }
        let set = table.at(usize::from(offset))?;
        let count = usize::from(set.u16(0)?);
        let offsets = set.words(2, count)?;
        let mut rules = Vec::new();
        rules.try_reserve_exact(count)?;
        for offset in offsets {
            let rule = set.at(usize::from(offset))?;
            let mut position = 0;
            let backtrack = counted_glyphs(rule, &mut position)?;
            let input_count = usize::from(rule.u16(position)?);
            if input_count == 0 {
                return Err(FontError::Invalid("GSUB chaining input count is zero"));
            }
            position += 2;
            let input = glyphs(rule, position, input_count - 1)?;
            position = position
                .checked_add((input_count - 1) * 2)
                .ok_or(FontError::Bounds)?;
            let lookahead = counted_glyphs(rule, &mut position)?;
            let count = usize::from(rule.u16(position)?);
            let lookups = records(rule, position + 2, count)?;
            rules.push(ChainRule {
                backtrack,
                input,
                lookahead,
                lookups,
            });
        }
        sets.push(Some(rules));
    }
    Ok(sets)
}

pub(super) fn counted_glyphs(
    table: Reader<'_>,
    position: &mut usize,
) -> Result<Vec<Glyph>, FontError> {
    let count = usize::from(table.u16(*position)?);
    *position += 2;
    let result = glyphs(table, *position, count)?;
    *position = position.checked_add(count * 2).ok_or(FontError::Bounds)?;
    Ok(result)
}

pub(super) fn coverages(
    table: Reader<'_>,
    position: &mut usize,
) -> Result<Vec<Coverage>, FontError> {
    let count = usize::from(table.u16(*position)?);
    *position += 2;
    let offsets = table.words(*position, count)?;
    *position = position.checked_add(count * 2).ok_or(FontError::Bounds)?;
    let mut result = Vec::new();
    result.try_reserve_exact(count)?;
    for offset in offsets {
        result.push(coverage(table.at(usize::from(offset))?)?);
    }
    Ok(result)
}
