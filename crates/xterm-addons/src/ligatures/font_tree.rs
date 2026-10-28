// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Arena IDs preserve entry sharing without ownership cycles. IDs are private
//! and created only by this arena, so internal index access is invariant-safe.
use super::{
    font_bytes::FontError,
    font_tables::{Glyph, GlyphSet, GlyphValue},
};
use std::collections::HashMap;
#[path = "font_tree_retain.rs"]
mod retain;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EntryId(usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TreeId(usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LookupId(usize);

#[derive(Clone, Debug, Default)]
pub struct LookupResult {
    pub substitutions: Vec<Option<GlyphValue>>,
    pub length: usize,
    pub index: usize,
    pub sub_index: usize,
    pub context_range: [i64; 2],
}
#[derive(Clone, Debug, Default)]
pub struct Entry {
    pub lookup: Option<LookupId>,
    pub forward: Option<TreeId>,
    pub reverse: Option<TreeId>,
}
#[derive(Clone, Copy, Debug)]
pub struct RangeEntry {
    pub start: Glyph,
    pub end: Glyph,
    pub entry: EntryId,
}
#[derive(Default)]
struct Tree {
    individual: HashMap<Glyph, EntryId>,
    order: Vec<Glyph>,
    ranges: Vec<RangeEntry>,
}
#[derive(Default)]
pub struct Arena {
    entries: Vec<Entry>,
    trees: Vec<Tree>,
    lookups: Vec<LookupResult>,
}
impl Arena {
    pub fn tree(&mut self) -> Result<TreeId, FontError> {
        self.trees.try_reserve(1)?;
        let id = TreeId(self.trees.len());
        self.trees.push(Tree::default());
        Ok(id)
    }
    pub fn entry(&mut self) -> Result<EntryId, FontError> {
        self.entries.try_reserve(1)?;
        let id = EntryId(self.entries.len());
        self.entries.push(Entry::default());
        Ok(id)
    }
    pub fn lookup(&mut self, result: LookupResult) -> Result<LookupId, FontError> {
        self.lookups.try_reserve(1)?;
        let id = LookupId(self.lookups.len());
        self.lookups.push(result);
        Ok(id)
    }
    pub fn result(&self, id: LookupId) -> &LookupResult {
        &self.lookups[id.0]
    }
    fn clone_lookup(&mut self, id: LookupId) -> Result<LookupId, FontError> {
        let source = self.result(id);
        let mut substitutions = Vec::new();
        substitutions.try_reserve_exact(source.substitutions.len())?;
        substitutions.extend_from_slice(&source.substitutions);
        let copy = LookupResult {
            substitutions,
            length: source.length,
            index: source.index,
            sub_index: source.sub_index,
            context_range: source.context_range,
        };
        self.lookup(copy)
    }
    pub fn get(&self, id: EntryId) -> &Entry {
        &self.entries[id.0]
    }
    pub fn get_mut(&mut self, id: EntryId) -> &mut Entry {
        &mut self.entries[id.0]
    }
    pub fn individual(&self, tree: TreeId, glyph: Glyph) -> Option<EntryId> {
        self.trees[tree.0].individual.get(&glyph).copied()
    }
    pub fn set_individual(
        &mut self,
        tree: TreeId,
        glyph: Glyph,
        entry: EntryId,
    ) -> Result<(), FontError> {
        let target = &mut self.trees[tree.0];
        if !target.individual.contains_key(&glyph) {
            target
                .individual
                .try_reserve(1)
                .map_err(|_| FontError::Capacity)?;
            target.order.try_reserve(1)?;
            let position = if is_array_index(glyph) {
                target
                    .order
                    .iter()
                    .position(|stored| !is_array_index(*stored) || *stored > glyph)
                    .unwrap_or(target.order.len())
            } else {
                target.order.len()
            };
            target.order.insert(position, glyph);
        }
        target.individual.insert(glyph, entry);
        Ok(())
    }
    pub fn individuals(&self, tree: TreeId) -> Result<Vec<(Glyph, EntryId)>, FontError> {
        let target = &self.trees[tree.0];
        let mut output = Vec::new();
        output.try_reserve_exact(target.order.len())?;
        for key in &target.order {
            output.push((*key, target.individual[key]));
        }
        Ok(output)
    }
    pub fn range(&self, tree: TreeId, index: usize) -> Option<RangeEntry> {
        self.trees[tree.0].ranges.get(index).copied()
    }
    pub fn push_range(
        &mut self,
        tree: TreeId,
        start: Glyph,
        end: Glyph,
        entry: EntryId,
    ) -> Result<(), FontError> {
        self.trees[tree.0].ranges.try_reserve(1)?;
        self.trees[tree.0]
            .ranges
            .push(RangeEntry { start, end, entry });
        Ok(())
    }
    pub fn splice_range(&mut self, tree: TreeId, index: isize) {
        let ranges = &mut self.trees[tree.0].ranges;
        let position = if index < 0 {
            ranges.len().saturating_sub(index.unsigned_abs())
        } else {
            index.unsigned_abs().min(ranges.len())
        };
        if position < ranges.len() {
            ranges.remove(position);
        }
    }
    pub fn insert_set(
        &mut self,
        tree: TreeId,
        set: GlyphSet,
        entry: EntryId,
    ) -> Result<(), FontError> {
        match set {
            GlyphSet::Individual(glyph) => self.set_individual(tree, glyph, entry),
            GlyphSet::Range(start, end) => self.push_range(tree, start, end, entry),
        }
    }
    pub fn ensure_forward(&mut self, entry: EntryId) -> Result<TreeId, FontError> {
        if let Some(tree) = self.get(entry).forward {
            return Ok(tree);
        }
        let tree = self.tree()?;
        self.get_mut(entry).forward = Some(tree);
        Ok(tree)
    }
    pub fn ensure_reverse(&mut self, entry: EntryId) -> Result<TreeId, FontError> {
        if let Some(tree) = self.get(entry).reverse {
            return Ok(tree);
        }
        let tree = self.tree()?;
        self.get_mut(entry).reverse = Some(tree);
        Ok(tree)
    }
    pub fn clone_entry(&mut self, entry: EntryId) -> Result<EntryId, FontError> {
        let mut visited = HashMap::new();
        let mut work = Vec::new();
        let target = self.copy_entry(entry, &mut visited, &mut work)?;
        self.finish_copies(false, &mut visited, work)?;
        Ok(target)
    }
    pub fn flatten(&mut self, tree: TreeId) -> Result<TreeId, FontError> {
        let mut visited = HashMap::new();
        let mut work = Vec::new();
        let target = self.copy_tree(tree, true, &mut visited, &mut work)?;
        self.finish_copies(true, &mut visited, work)?;
        Ok(target)
    }
    fn copy_entry(
        &mut self,
        entry: EntryId,
        visited: &mut HashMap<EntryId, EntryId>,
        work: &mut Vec<(EntryId, EntryId)>,
    ) -> Result<EntryId, FontError> {
        if let Some(target) = visited.get(&entry) {
            return Ok(*target);
        }
        let target = self.entry()?;
        visited.try_reserve(1).map_err(|_| FontError::Capacity)?;
        work.try_reserve(1)?;
        visited.insert(entry, target);
        work.push((entry, target));
        Ok(target)
    }
    fn copy_tree(
        &mut self,
        tree: TreeId,
        flatten: bool,
        visited: &mut HashMap<EntryId, EntryId>,
        work: &mut Vec<(EntryId, EntryId)>,
    ) -> Result<TreeId, FontError> {
        let target = self.tree()?;
        for (glyph, entry) in self.individuals(tree)? {
            let entry = self.copy_entry(entry, visited, work)?;
            self.set_individual(target, glyph, entry)?;
        }
        let mut index = 0;
        while let Some(range) = self.range(tree, index) {
            let entry = self.copy_entry(range.entry, visited, work)?;
            if flatten {
                for glyph in range.start..range.end {
                    self.set_individual(target, glyph, entry)?;
                }
            } else {
                self.push_range(target, range.start, range.end, entry)?;
            }
            index += 1;
        }
        Ok(target)
    }
    fn finish_copies(
        &mut self,
        flatten: bool,
        visited: &mut HashMap<EntryId, EntryId>,
        mut work: Vec<(EntryId, EntryId)>,
    ) -> Result<(), FontError> {
        // An explicit worklist handles shared/cyclic and deeply nested font
        // contexts without overflowing the native call stack.
        while let Some((source, target)) = work.pop() {
            let source = self.get(source).clone();
            let forward = source
                .forward
                .map(|tree| self.copy_tree(tree, flatten, visited, &mut work))
                .transpose()?;
            let reverse = source
                .reverse
                .map(|tree| self.copy_tree(tree, flatten, visited, &mut work))
                .transpose()?;
            let lookup = if flatten {
                source.lookup
            } else {
                source.lookup.map(|id| self.clone_lookup(id)).transpose()?
            };
            *self.get_mut(target) = Entry {
                lookup,
                forward,
                reverse,
            };
        }
        Ok(())
    }
}
fn is_array_index(value: Glyph) -> bool {
    value >= 0 && value < i64::from(u32::MAX)
}
