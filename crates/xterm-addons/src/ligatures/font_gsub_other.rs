// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::*;

pub(super) fn single(table: Reader<'_>) -> Result<SingleSubstitution, FontError> {
    let coverage = coverage(table.child16(2)?)?;
    match table.u16(0)? {
        1 => Ok(SingleSubstitution::Delta {
            coverage,
            delta: table.i16(4)?,
        }),
        2 => Ok(SingleSubstitution::Substitute {
            coverage,
            glyphs: glyphs(table, 6, usize::from(table.u16(4)?))?,
        }),
        format => Err(FontError::UnsupportedFormat(
            "GSUB single substitution",
            format,
        )),
    }
}

pub(super) fn reverse(table: Reader<'_>) -> Result<ReverseTable, FontError> {
    if table.u16(0)? != 1 {
        return Err(FontError::UnsupportedFormat(
            "GSUB reverse chaining",
            table.u16(0)?,
        ));
    }
    let coverage = coverage(table.child16(2)?)?;
    let mut position = 4;
    let backtrack = chain::coverages(table, &mut position)?;
    let lookahead = chain::coverages(table, &mut position)?;
    let substitutes = chain::counted_glyphs(table, &mut position)?;
    Ok(ReverseTable {
        coverage,
        backtrack,
        lookahead,
        substitutes,
    })
}

pub(super) fn generic(kind: u16, table: Reader<'_>) -> Result<GenericTable, FontError> {
    let format = table.u16(0)?;
    let coverage = match kind {
        2..=4 => {
            if format != 1 {
                return Err(FontError::UnsupportedFormat(
                    "GSUB noncontext substitution",
                    format,
                ));
            }
            let coverage = coverage(table.child16(2)?)?;
            let count = usize::from(table.u16(4)?);
            for offset in table.words(6, count)? {
                let set = table.at(usize::from(offset))?;
                let count = usize::from(set.u16(0)?);
                if kind == 4 {
                    for offset in set.words(2, count)? {
                        let ligature = set.at(usize::from(offset))?;
                        ligature.u16(0)?;
                        let count = usize::from(ligature.u16(2)?);
                        if count == 0 {
                            return Err(FontError::Invalid(
                                "GSUB ligature component count is zero",
                            ));
                        }
                        ligature.words(4, count - 1)?;
                    }
                } else {
                    set.words(2, count)?;
                }
            }
            Some(coverage)
        }
        5 => context(table)?,
        7 => {
            if format != 1 {
                return Err(FontError::UnsupportedFormat("GSUB extension", format));
            }
            let target_kind = table.u16(2)?;
            let target = table.at(table.offset32(4)?)?;
            // Extension lookup type7 cannot contain another type7 by spec.
            // Reject it rather than recursing indefinitely on untrusted bytes.
            match target_kind {
                1 => {
                    single(target)?;
                }
                6 => {
                    chain::parse(target)?;
                }
                8 => {
                    reverse(target)?;
                }
                2..=5 => {
                    generic(target_kind, target)?;
                }
                _ => {
                    return Err(FontError::UnsupportedFormat(
                        "GSUB extension lookup",
                        target_kind,
                    ))
                }
            }
            None
        }
        _ => return Err(FontError::UnsupportedFormat("GSUB lookup", kind)),
    };
    Ok(GenericTable { format, coverage })
}

fn context(table: Reader<'_>) -> Result<Option<Coverage>, FontError> {
    match table.u16(0)? {
        1 | 2 => {
            let coverage = coverage(table.child16(2)?)?;
            let format = table.u16(0)?;
            let position = if format == 2 {
                class_def(table.child16(4)?)?;
                6
            } else {
                4
            };
            let count = usize::from(table.u16(position)?);
            for offset in table.words(position + 2, count)? {
                if offset == 0 {
                    continue;
                }
                let set = table.at(usize::from(offset))?;
                let count = usize::from(set.u16(0)?);
                for offset in set.words(2, count)? {
                    context_rule(set.at(usize::from(offset))?)?;
                }
            }
            Ok(Some(coverage))
        }
        3 => {
            let glyph_count = usize::from(table.u16(2)?);
            let lookup_count = usize::from(table.u16(4)?);
            for offset in table.words(6, glyph_count)? {
                coverage(table.at(usize::from(offset))?)?;
            }
            records(table, 6 + glyph_count * 2, lookup_count)?;
            Ok(None)
        }
        format => Err(FontError::UnsupportedFormat("GSUB context", format)),
    }
}
fn context_rule(table: Reader<'_>) -> Result<(), FontError> {
    let glyph_count = usize::from(table.u16(0)?);
    let lookup_count = usize::from(table.u16(2)?);
    if glyph_count == 0 {
        return Err(FontError::Invalid("GSUB context input count is zero"));
    }
    table.words(4, glyph_count - 1)?;
    records(table, 4 + (glyph_count - 1) * 2, lookup_count)?;
    Ok(())
}
