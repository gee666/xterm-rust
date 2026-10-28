// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::*;

enum Job {
    Tree(TreeId, TreeId),
    Entry(EntryId, EntryId),
}
struct Retainer {
    source: Arena,
    target: Arena,
    trees: Vec<Option<TreeId>>,
    entries: Vec<Option<EntryId>>,
    lookups: Vec<Option<LookupId>>,
    work: Vec<Job>,
}
fn map<T: Copy>(count: usize) -> Result<Vec<Option<T>>, FontError> {
    let mut values = Vec::new();
    values.try_reserve_exact(count)?;
    values.resize(count, None);
    Ok(values)
}
impl Arena {
    /// Drop intermediate builder/merge nodes just as source garbage collection
    /// releases them after only the flattened lookup graphs remain reachable.
    pub fn retain(self, roots: &[TreeId]) -> Result<(Self, Vec<TreeId>), FontError> {
        let mut state = Retainer {
            trees: map(self.trees.len())?,
            entries: map(self.entries.len())?,
            lookups: map(self.lookups.len())?,
            source: self,
            target: Self::default(),
            work: Vec::new(),
        };
        let mut retained = Vec::new();
        retained.try_reserve_exact(roots.len())?;
        for root in roots {
            retained.push(state.tree(*root)?);
        }
        while let Some(job) = state.work.pop() {
            state.run(job)?;
        }
        Ok((state.target, retained))
    }
}
impl Retainer {
    fn tree(&mut self, source: TreeId) -> Result<TreeId, FontError> {
        if let Some(target) = self.trees[source.0] {
            return Ok(target);
        }
        let target = self.target.tree()?;
        self.work.try_reserve(1)?;
        self.trees[source.0] = Some(target);
        self.work.push(Job::Tree(source, target));
        Ok(target)
    }
    fn entry(&mut self, source: EntryId) -> Result<EntryId, FontError> {
        if let Some(target) = self.entries[source.0] {
            return Ok(target);
        }
        let target = self.target.entry()?;
        self.work.try_reserve(1)?;
        self.entries[source.0] = Some(target);
        self.work.push(Job::Entry(source, target));
        Ok(target)
    }
    fn lookup(&mut self, source: LookupId) -> Result<LookupId, FontError> {
        if let Some(target) = self.lookups[source.0] {
            return Ok(target);
        }
        // This consumes the original arena. Each payload is moved once, and
        // every later alias uses the stored map instead of its emptied slot.
        let value = std::mem::take(&mut self.source.lookups[source.0]);
        let target = self.target.lookup(value)?;
        self.lookups[source.0] = Some(target);
        Ok(target)
    }
    fn run(&mut self, job: Job) -> Result<(), FontError> {
        match job {
            Job::Tree(source, target) => {
                for (glyph, entry) in self.source.individuals(source)? {
                    let entry = self.entry(entry)?;
                    self.target.set_individual(target, glyph, entry)?;
                }
                let mut index = 0;
                while let Some(range) = self.source.range(source, index) {
                    let entry = self.entry(range.entry)?;
                    self.target
                        .push_range(target, range.start, range.end, entry)?;
                    index += 1;
                }
            }
            Job::Entry(source, target) => {
                let source = self.source.get(source).clone();
                let forward = source.forward.map(|tree| self.tree(tree)).transpose()?;
                let reverse = source.reverse.map(|tree| self.tree(tree)).transpose()?;
                let lookup = source
                    .lookup
                    .map(|lookup| self.lookup(lookup))
                    .transpose()?;
                *self.target.get_mut(target) = Entry {
                    forward,
                    reverse,
                    lookup,
                };
            }
        }
        Ok(())
    }
}
