// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{
    font_tables::GlyphValue,
    font_tree::{Arena, LookupId, TreeId},
};

pub fn walk(
    arena: &Arena,
    mut tree: TreeId,
    sequence: &[GlyphValue],
    start: usize,
) -> Option<LookupId> {
    let mut position = start;
    let mut best = None;
    while let Some(glyph) = sequence.get(position).and_then(|value| value.number()) {
        let Some(entry) = arena.individual(tree, glyph) else {
            break;
        };
        let entry = arena.get(entry);
        let mut lookup = entry.lookup;
        if let Some(reverse) = entry.reverse {
            lookup = prefer(arena, lookup, walk_reverse(arena, reverse, sequence, start));
        }
        best = prefer(arena, best, lookup);
        position += 1;
        let Some(forward) = entry.forward else {
            break;
        };
        tree = forward;
    }
    best
}
fn prefer(
    arena: &Arena,
    current: Option<LookupId>,
    candidate: Option<LookupId>,
) -> Option<LookupId> {
    match (current, candidate) {
        (Some(current), Some(candidate)) => {
            let left = arena.result(current);
            let right = arena.result(candidate);
            if (right.index, right.sub_index) < (left.index, left.sub_index) {
                Some(candidate)
            } else {
                Some(current)
            }
        }
        (Some(current), None) => Some(current),
        (None, candidate) => candidate,
    }
}
fn walk_reverse(
    arena: &Arena,
    mut tree: TreeId,
    sequence: &[GlyphValue],
    start: usize,
) -> Option<LookupId> {
    let mut position = start.checked_sub(1)?;
    let mut best = None;
    loop {
        let Some(glyph) = sequence.get(position).and_then(|value| value.number()) else {
            break;
        };
        let Some(entry) = arena.individual(tree, glyph) else {
            break;
        };
        let entry = arena.get(entry);
        if let Some(candidate) = entry.lookup {
            // Deeper reverse matches compare only index in the source.
            if best
                .is_none_or(|current| arena.result(current).index > arena.result(candidate).index)
            {
                best = Some(candidate);
            }
        }
        let Some(previous) = position.checked_sub(1) else {
            break;
        };
        let Some(reverse) = entry.reverse else {
            break;
        };
        position = previous;
        tree = reverse;
    }
    best
}
