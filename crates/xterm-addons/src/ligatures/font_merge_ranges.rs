// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::Job as MergeJob;
use super::*;

pub(super) struct Scan {
    main: TreeId,
    incoming: EntryId,
    remaining: Vec<GlyphSet>,
    index: isize,
}
pub(super) struct Inner {
    scan: Scan,
    range: super::super::font_tree::RangeEntry,
    snapshot: Vec<GlyphSet>,
    cursor: usize,
}
pub(super) struct Keys {
    main: TreeId,
    incoming: EntryId,
    remaining: Vec<GlyphSet>,
    keys: Vec<Glyph>,
    next: usize,
}
pub(super) enum Job {
    Outer {
        main: TreeId,
        incoming: TreeId,
        index: usize,
    },
    Scan(Scan),
    Inner(Inner),
    AfterRange {
        state: Inner,
        first: Vec<GlyphSet>,
        second: Vec<GlyphSet>,
        original: EntryId,
    },
    AfterIndividual {
        state: Scan,
        remaining_index: usize,
        first: Vec<GlyphSet>,
        second: Vec<GlyphSet>,
        original: EntryId,
    },
    Keys(Keys),
    AfterKey {
        state: Keys,
        remaining_index: usize,
        second: Vec<GlyphSet>,
    },
}
impl Merger<'_> {
    pub(super) fn range_job(&mut self, job: Job) -> Result<(), FontError> {
        match job {
            Job::Outer {
                main,
                incoming,
                index,
            } => {
                if let Some(range) = self.arena.range(incoming, index) {
                    self.push(MergeJob::Ranges(Job::Outer {
                        main,
                        incoming,
                        index: index + 1,
                    }))?;
                    self.push(MergeJob::Ranges(Job::Scan(Scan {
                        main,
                        incoming: range.entry,
                        remaining: vec![GlyphSet::Range(range.start, range.end)],
                        index: 0,
                    })))?;
                }
                Ok(())
            }
            Job::Scan(state) => self.scan_ranges(state),
            Job::Inner(state) => self.scan_remaining(state),
            Job::AfterRange {
                mut state,
                first,
                second,
                original,
            } => {
                self.append_clones(state.scan.main, original, &second)?;
                state.scan.remaining = first;
                self.push(MergeJob::Ranges(Job::Inner(state)))
            }
            Job::AfterIndividual {
                mut state,
                remaining_index,
                first,
                second,
                original,
            } => {
                self.arena.splice_range(state.main, state.index);
                state.index = state.index.checked_sub(1).ok_or(FontError::Bounds)?;
                self.append_clones(state.main, original, &second)?;
                replace_remaining(&mut state.remaining, remaining_index, &first)?;
                state.index = state.index.checked_add(1).ok_or(FontError::Bounds)?;
                self.push(MergeJob::Ranges(Job::Scan(state)))
            }
            Job::Keys(state) => self.scan_keys(state),
            Job::AfterKey {
                mut state,
                remaining_index,
                second,
            } => {
                replace_remaining(&mut state.remaining, remaining_index, &second)?;
                self.push(MergeJob::Ranges(Job::Keys(state)))
            }
        }
    }
    fn scan_ranges(&mut self, state: Scan) -> Result<(), FontError> {
        if state.index < 0 {
            return Err(FontError::Invalid("Missing range during GSUB tree merge"));
        }
        if let Some(range) = self.arena.range(state.main, state.index.unsigned_abs()) {
            // Object variable reassignment inside the source loop does not
            // replace the array already held by its entries iterator.
            let mut snapshot = Vec::new();
            snapshot.try_reserve_exact(state.remaining.len())?;
            snapshot.extend_from_slice(&state.remaining);
            self.push(MergeJob::Ranges(Job::Inner(Inner {
                scan: state,
                range,
                snapshot,
                cursor: 0,
            })))
        } else {
            let keys = self
                .arena
                .individuals(state.main)?
                .into_iter()
                .map(|(key, _)| key)
                .collect();
            self.push(MergeJob::Ranges(Job::Keys(Keys {
                main: state.main,
                incoming: state.incoming,
                remaining: state.remaining,
                keys,
                next: 0,
            })))
        }
    }
    fn scan_remaining(&mut self, mut state: Inner) -> Result<(), FontError> {
        let Some(remaining) = state.snapshot.get(state.cursor).copied() else {
            state.scan.index = state.scan.index.checked_add(1).ok_or(FontError::Bounds)?;
            return self.push(MergeJob::Ranges(Job::Scan(state.scan)));
        };
        let current = [state.range.start, state.range.end];
        match remaining {
            GlyphSet::Range(start, end) => {
                let overlap = font_overlap::ranges([start, end], current);
                let Some(both) = overlap.both else {
                    state.cursor += 1;
                    return self.push(MergeJob::Ranges(Job::Inner(state)));
                };
                self.arena.splice_range(state.scan.main, state.scan.index);
                state.scan.index = state.scan.index.checked_sub(1).ok_or(FontError::Bounds)?;
                let target = self.arena.clone_entry(state.range.entry)?;
                self.arena.insert_set(state.scan.main, both, target)?;
                let incoming = self.arena.clone_entry(state.scan.incoming)?;
                let original = state.range.entry;
                state.cursor += 1;
                self.push(MergeJob::Ranges(Job::AfterRange {
                    state,
                    first: overlap.first,
                    second: overlap.second,
                    original,
                }))?;
                self.push(MergeJob::Entry(target, incoming))
            }
            GlyphSet::Individual(glyph) => {
                let overlap = font_overlap::individual(glyph, current);
                if overlap.both.is_none() {
                    state.cursor += 1;
                    return self.push(MergeJob::Ranges(Job::Inner(state)));
                }
                let target = self.arena.clone_entry(state.scan.incoming)?;
                self.arena.set_individual(state.scan.main, glyph, target)?;
                let prior = self.arena.clone_entry(state.range.entry)?;
                self.push(MergeJob::Ranges(Job::AfterIndividual {
                    state: state.scan,
                    remaining_index: state.cursor,
                    first: overlap.first,
                    second: overlap.second,
                    original: state.range.entry,
                }))?;
                self.push(MergeJob::Entry(target, prior))
            }
        }
    }
    fn scan_keys(&mut self, mut state: Keys) -> Result<(), FontError> {
        while let Some(glyph) = state.keys.get(state.next).copied() {
            state.next += 1;
            for index in 0..state.remaining.len() {
                let rest = match state.remaining[index] {
                    GlyphSet::Range(start, end) => {
                        let overlap = font_overlap::individual(glyph, [start, end]);
                        if overlap.both.is_none() {
                            continue;
                        }
                        Some(overlap.second)
                    }
                    GlyphSet::Individual(value) => {
                        if value != glyph {
                            continue;
                        }
                        None
                    }
                };
                let target = self
                    .arena
                    .individual(state.main, glyph)
                    .ok_or(FontError::Invalid("Missing individual during GSUB merge"))?;
                let incoming = self.arena.clone_entry(state.incoming)?;
                match rest {
                    Some(second) => self.push(MergeJob::Ranges(Job::AfterKey {
                        state,
                        remaining_index: index,
                        second,
                    }))?,
                    None => self.push(MergeJob::Ranges(Job::Keys(state)))?,
                }
                return self.push(MergeJob::Entry(target, incoming));
            }
        }
        self.append_clones(state.main, state.incoming, &state.remaining)
    }
}
fn replace_remaining(
    values: &mut Vec<GlyphSet>,
    index: usize,
    replacements: &[GlyphSet],
) -> Result<(), FontError> {
    let index = index.min(values.len());
    let end = index + usize::from(index < values.len());
    values.try_reserve(replacements.len())?;
    values.splice(index..end, replacements.iter().copied());
    Ok(())
}
