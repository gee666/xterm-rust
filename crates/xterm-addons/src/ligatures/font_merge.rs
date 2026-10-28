// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Ordered merge continuations preserve recursive source ordering without
//! recursively consuming the native stack on deeply nested font contexts.
use super::{
    font_bytes::FontError,
    font_overlap,
    font_tables::{Glyph, GlyphSet},
    font_tree::{Arena, EntryId, LookupId, TreeId},
};
use std::collections::HashSet;
#[path = "font_merge_ranges.rs"]
mod ranges;

struct Individuals {
    main: TreeId,
    incoming: TreeId,
    entries: Vec<(Glyph, EntryId)>,
    next: usize,
}
struct IndividualRanges {
    main: TreeId,
    glyph: Glyph,
    value: EntryId,
    index: usize,
    matched: bool,
}
enum Job {
    Subtree(TreeId, TreeId),
    Individuals(Individuals),
    IndividualRanges(IndividualRanges),
    AfterIndividualRange {
        state: IndividualRanges,
        original: EntryId,
        remaining: Vec<GlyphSet>,
    },
    Entry(EntryId, EntryId),
    Reverse(EntryId, EntryId),
    Ranges(ranges::Job),
}
struct Merger<'a> {
    arena: &'a mut Arena,
    seen: HashSet<(EntryId, EntryId)>,
    work: Vec<Job>,
}
pub fn merge(arena: &mut Arena, trees: &[TreeId]) -> Result<TreeId, FontError> {
    let root = arena.tree()?;
    let mut merger = Merger {
        arena,
        seen: HashSet::new(),
        work: Vec::new(),
    };
    for tree in trees.iter().rev() {
        merger.push(Job::Subtree(root, *tree))?;
    }
    while let Some(job) = merger.work.pop() {
        merger.run(job)?;
    }
    Ok(root)
}
impl Merger<'_> {
    fn push(&mut self, job: Job) -> Result<(), FontError> {
        self.work.try_reserve(1)?;
        self.work.push(job);
        Ok(())
    }
    fn run(&mut self, job: Job) -> Result<(), FontError> {
        match job {
            Job::Subtree(main, incoming) => {
                let entries = self.arena.individuals(incoming)?;
                self.push(Job::Individuals(Individuals {
                    main,
                    incoming,
                    entries,
                    next: 0,
                }))
            }
            Job::Individuals(mut state) => {
                let Some((glyph, value)) = state.entries.get(state.next).copied() else {
                    return self.push(Job::Ranges(ranges::Job::Outer {
                        main: state.main,
                        incoming: state.incoming,
                        index: 0,
                    }));
                };
                state.next += 1;
                let main = state.main;
                self.push(Job::Individuals(state))?;
                if let Some(existing) = self.arena.individual(main, glyph) {
                    self.push(Job::Entry(existing, value))
                } else {
                    self.push(Job::IndividualRanges(IndividualRanges {
                        main,
                        glyph,
                        value,
                        index: 0,
                        matched: false,
                    }))
                }
            }
            Job::IndividualRanges(mut state) => {
                let Some(range) = self.arena.range(state.main, state.index) else {
                    if !state.matched {
                        self.arena
                            .set_individual(state.main, state.glyph, state.value)?;
                    }
                    return Ok(());
                };
                let overlap = font_overlap::individual(state.glyph, [range.start, range.end]);
                if overlap.both.is_none() {
                    state.index += 1;
                    return self.push(Job::IndividualRanges(state));
                }
                state.matched = true;
                self.arena
                    .set_individual(state.main, state.glyph, state.value)?;
                let prior = self.arena.clone_entry(range.entry)?;
                let value = state.value;
                self.push(Job::AfterIndividualRange {
                    state,
                    original: range.entry,
                    remaining: overlap.second,
                })?;
                self.push(Job::Entry(value, prior))
            }
            Job::AfterIndividualRange {
                mut state,
                original,
                remaining,
            } => {
                self.arena.splice_range(
                    state.main,
                    isize::try_from(state.index).map_err(|_| FontError::Bounds)?,
                );
                self.append_clones(state.main, original, &remaining)?;
                state.index += 1;
                self.push(Job::IndividualRanges(state))
            }
            Job::Entry(main, incoming) => self.entry(main, incoming),
            Job::Reverse(main, incoming) => {
                if let Some(reverse) = self.arena.get(incoming).reverse {
                    if let Some(existing) = self.arena.get(main).reverse {
                        self.push(Job::Subtree(existing, reverse))?;
                    } else {
                        self.arena.get_mut(main).reverse = Some(reverse);
                    }
                }
                Ok(())
            }
            Job::Ranges(job) => self.range_job(job),
        }
    }
    fn entry(&mut self, main: EntryId, incoming: EntryId) -> Result<(), FontError> {
        if self.seen.contains(&(main, incoming)) {
            return Ok(());
        }
        self.seen.try_reserve(1).map_err(|_| FontError::Capacity)?;
        self.seen.insert((main, incoming));
        if let Some(candidate) = self.arena.get(incoming).lookup {
            if self
                .arena
                .get(main)
                .lookup
                .is_none_or(|current| self.better(candidate, current))
            {
                self.arena.get_mut(main).lookup = Some(candidate);
            }
        }
        self.push(Job::Reverse(main, incoming))?;
        if let Some(forward) = self.arena.get(incoming).forward {
            if let Some(existing) = self.arena.get(main).forward {
                self.push(Job::Subtree(existing, forward))?;
            } else {
                self.arena.get_mut(main).forward = Some(forward);
            }
        }
        Ok(())
    }
    fn better(&self, candidate: LookupId, current: LookupId) -> bool {
        let candidate = self.arena.result(candidate);
        let current = self.arena.result(current);
        (candidate.index, candidate.sub_index) < (current.index, current.sub_index)
    }
    fn append_clones(
        &mut self,
        tree: TreeId,
        original: EntryId,
        glyphs: &[GlyphSet],
    ) -> Result<(), FontError> {
        for glyphs in glyphs {
            let entry = self.arena.clone_entry(original)?;
            self.arena.insert_set(tree, *glyphs, entry)?;
        }
        Ok(())
    }
}
