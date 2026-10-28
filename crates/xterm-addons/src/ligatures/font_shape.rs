// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{font_merge_range, font_tree::LookupResult, font_walk, Font, FontError, GlyphValue};

pub(super) struct Shaped {
    pub sequence: Vec<GlyphValue>,
    pub ranges: Vec<[i64; 2]>,
}
struct Next {
    index: usize,
    first: usize,
    last: usize,
}
impl Font {
    pub(super) fn shape(&self, mut sequence: Vec<GlyphValue>) -> Result<Shaped, FontError> {
        let mut ranges = Vec::new();
        let mut next = self.next_lookup(&sequence, 0);
        while let Some(current) = next {
            let lookup = &self.groups[current.index];
            if lookup.forward {
                let mut last = current.last;
                let mut position = current.first;
                while position < last {
                    if let Some(id) = font_walk::walk(&self.arena, lookup.tree, &sequence, position)
                    {
                        let result = self.arena.result(id);
                        apply(result, &mut sequence, position, &mut ranges)?;
                        let end = position
                            .checked_add(result.length)
                            .ok_or(FontError::Bounds)?;
                        if end >= last {
                            last = end.checked_add(1).ok_or(FontError::Bounds)?;
                        }
                        position = end;
                    } else {
                        position += 1;
                    }
                }
            } else {
                let first = i64::try_from(current.first).map_err(|_| FontError::Bounds)?;
                let mut position = i64::try_from(current.last).map_err(|_| FontError::Bounds)? - 1;
                while position >= first {
                    let index = usize::try_from(position).map_err(|_| FontError::Bounds)?;
                    if let Some(id) = font_walk::walk(&self.arena, lookup.tree, &sequence, index) {
                        let result = self.arena.result(id);
                        apply(result, &mut sequence, index, &mut ranges)?;
                        position = position
                            .checked_sub(
                                i64::try_from(result.length).map_err(|_| FontError::Bounds)?,
                            )
                            .ok_or(FontError::Bounds)?;
                    } else {
                        position -= 1;
                    }
                }
            }
            next = self.next_lookup(&sequence, current.index + 1);
        }
        Ok(Shaped { sequence, ranges })
    }
    fn next_lookup(&self, sequence: &[GlyphValue], start: usize) -> Option<Next> {
        let mut result: Option<Next> = None;
        for (position, glyph) in sequence.iter().enumerate() {
            let Some(lookups) = glyph
                .number()
                .and_then(|glyph| self.glyph_lookups.get(&glyph))
            else {
                continue;
            };
            for index in lookups {
                if *index >= start {
                    if let Some(current) = &mut result {
                        if *index <= current.index {
                            current.index = *index;
                            // Do not reset first when a lower-index group is
                            // discovered later. The source retains that value.
                            current.first = current.first.min(position);
                            current.last = position + 1;
                        }
                    } else {
                        result = Some(Next {
                            index: *index,
                            first: position,
                            last: position + 1,
                        });
                    }
                    break;
                }
            }
        }
        result
    }
}
fn apply(
    result: &LookupResult,
    sequence: &mut Vec<GlyphValue>,
    position: usize,
    ranges: &mut Vec<[i64; 2]>,
) -> Result<(), FontError> {
    if result.length == 0 {
        return Err(FontError::Invalid(
            "Zero-length ligature lookup cannot advance",
        ));
    }
    for (offset, substitution) in result.substitutions.iter().enumerate() {
        if let Some(substitution) = substitution {
            let target = position.checked_add(offset).ok_or(FontError::Bounds)?;
            if target >= sequence.len() {
                let required = target.checked_add(1).ok_or(FontError::Bounds)?;
                sequence.try_reserve(required - sequence.len())?;
                sequence.resize(required, GlyphValue::Undefined);
            }
            sequence[target] = *substitution;
        }
    }
    let position = i64::try_from(position).map_err(|_| FontError::Bounds)?;
    let begin = position
        .checked_add(result.context_range[0])
        .ok_or(FontError::Bounds)?;
    let end = position
        .checked_add(result.context_range[1])
        .ok_or(FontError::Bounds)?;
    font_merge_range::merge(ranges, begin, end)
}
