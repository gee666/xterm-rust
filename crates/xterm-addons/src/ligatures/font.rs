// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Contextual ligature lookup over raw font bytes. Full equivalence remains
//! unproven until the source differential and browser integration gates pass.
#[path = "font_build.rs"]
mod font_build;
#[path = "font_build_helpers.rs"]
mod font_build_helpers;
#[path = "font_bytes.rs"]
mod font_bytes;
#[path = "font_cache.rs"]
mod font_cache;
#[path = "font_cmap.rs"]
mod font_cmap;
#[path = "font_gsub.rs"]
mod font_gsub;
#[path = "font_kern.rs"]
mod font_kern;
#[path = "font_merge.rs"]
mod font_merge;
#[path = "font_merge_range.rs"]
mod font_merge_range;
#[path = "font_metadata.rs"]
mod font_metadata;
#[path = "font_overlap.rs"]
mod font_overlap;
#[path = "font_tables.rs"]
mod font_tables;
#[path = "font_tree.rs"]
mod font_tree;
#[path = "font_walk.rs"]
mod font_walk;
#[path = "font_warning.rs"]
mod font_warning;
#[path = "font_shape.rs"]
mod shape;

#[cfg(test)]
#[path = "font_graph_tests.rs"]
mod graph_tests;
#[cfg(test)]
#[path = "font_tests.rs"]
mod tests;

pub use font_bytes::FontError;
use font_bytes::FontTables;
use font_cache::{Cache, Cached};
pub use font_cache::{GlyphArray, LigatureData, LigatureResult, RangeArray};
use font_cmap::Cmap;
pub use font_tables::{Glyph, GlyphValue};
use font_tables::{Gsub, Lookup};
use font_tree::{Arena, TreeId};
pub use font_warning::{FontLoadError, FontWarning};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, Debug, Default)]
pub struct FontOptions {
    pub cache_size: usize,
}
struct Group {
    tree: TreeId,
    forward: bool,
}
pub struct Font {
    cmap: Cmap,
    arena: Arena,
    groups: Vec<Group>,
    glyph_lookups: HashMap<Glyph, Vec<usize>>,
    cache: Option<Cache>,
}
impl Font {
    /// Loads a snapshot without a diagnostic sink. Browser hosts should use
    /// `load_buffer_with_warnings` to preserve observable console callbacks.
    pub fn load_buffer(bytes: &[u8], options: FontOptions) -> Result<Self, FontError> {
        match Self::load_buffer_with_warnings(bytes, options, |_| {
            Ok::<(), std::convert::Infallible>(())
        }) {
            Ok(font) => Ok(font),
            Err(FontLoadError::Font(error)) => Err(error),
            Err(FontLoadError::Warning(never)) => match never {},
        }
    }

    /// Calls `warning` synchronously before later constructor reads. A callback
    /// failure aborts construction unchanged, including before a later bounds
    /// error. The callback may reenter font loading; no shared state is borrowed.
    /// Input bytes are an immutable snapshot throughout construction.
    pub fn load_buffer_with_warnings<E>(
        bytes: &[u8],
        options: FontOptions,
        mut warning: impl FnMut(FontWarning) -> Result<(), E>,
    ) -> Result<Self, FontLoadError<E>> {
        let tables = FontTables::parse(bytes)?;
        let cmap = Cmap::parse(&tables.cmap)?;
        if tables
            .glyph_count
            .is_some_and(|count| !cmap.valid_glyph_indices(count))
        {
            return Err(FontError::Type(
                "Cannot read properties of undefined (reading 'addUnicode')",
            )
            .into());
        }
        // opentype assigns cmap glyph names before it reads deferred kern/GSUB
        // data, so those later errors must not replace an invalid-glyph error.
        if let Some(table) = tables.kern {
            font_kern::validate(&table.load()?, &mut warning)?;
        }
        let gsub = tables
            .gsub
            .map(|table| font_gsub::parse(&table.load()?))
            .transpose()?;
        if let Some(table) = tables.meta {
            font_metadata::meta(&table.load()?)?;
        }
        // FontImpl builds lookup trees only after opentype finishes all
        // constructor tables. A tree error must not hide a later meta error.
        Self::from_gsub(gsub, cmap, options).map_err(FontLoadError::Font)
    }

    fn from_gsub(gsub: Option<Gsub>, cmap: Cmap, options: FontOptions) -> Result<Self, FontError> {
        let mut arena = Arena::default();
        let mut groups = Vec::new();
        if let Some(gsub) = gsub {
            for (lookup_index, lookup) in gsub.lookups.iter().enumerate() {
                if !gsub.selected.contains(&lookup_index) {
                    continue;
                }
                let mut trees = Vec::new();
                match lookup {
                    Lookup::Chain(tables) => {
                        trees.try_reserve_exact(tables.len())?;
                        for (index, table) in tables.iter().enumerate() {
                            trees.push(font_build::chain(&mut arena, table, &gsub.lookups, index)?);
                        }
                    }
                    Lookup::Reverse(tables) => {
                        trees.try_reserve_exact(tables.len())?;
                        for (index, table) in tables.iter().enumerate() {
                            trees.push(font_build::reverse(&mut arena, table, index)?);
                        }
                    }
                    // Source FontImpl retains calt groups of every type, but
                    // only types6/8 build lookup trees. Empty groups still
                    // affect its cache path, so do not discard them here.
                    Lookup::Single(_) | Lookup::Other(_) => {}
                }
                let merged = font_merge::merge(&mut arena, &trees)?;
                let tree = arena.flatten(merged)?;
                groups.try_reserve(1)?;
                groups.push(Group {
                    tree,
                    forward: !matches!(lookup, Lookup::Reverse(_)),
                });
            }
        }
        let mut roots = Vec::new();
        roots.try_reserve_exact(groups.len())?;
        roots.extend(groups.iter().map(|group| group.tree));
        let (arena, roots) = arena.retain(&roots)?;
        for (group, tree) in groups.iter_mut().zip(roots) {
            group.tree = tree;
        }
        let mut glyph_lookups: HashMap<Glyph, Vec<usize>> = HashMap::new();
        for (index, group) in groups.iter().enumerate() {
            for (glyph, _) in arena.individuals(group.tree)? {
                if !glyph_lookups.contains_key(&glyph) {
                    glyph_lookups
                        .try_reserve(1)
                        .map_err(|_| FontError::Capacity)?;
                }
                let lookups = glyph_lookups.entry(glyph).or_default();
                lookups.try_reserve(1)?;
                lookups.push(index);
            }
        }
        Ok(Self {
            cmap,
            arena,
            groups,
            glyph_lookups,
            cache: (options.cache_size > 0).then(|| Cache::new(options.cache_size)),
        })
    }

    pub fn find_ligatures(&mut self, text: &[u16]) -> Result<LigatureResult, FontError> {
        if let Some(Cached::Full(result)) = self.cache.as_mut().and_then(|cache| cache.get(text)) {
            return Ok(result);
        }
        let input = self.glyphs(text)?;
        if self.groups.is_empty() {
            let shared = Rc::new(RefCell::new(input));
            return Ok(Rc::new(RefCell::new(LigatureData {
                input_glyphs: Rc::clone(&shared),
                output_glyphs: shared,
                context_ranges: Rc::new(RefCell::new(Vec::new())),
            })));
        }
        let mut output = Vec::new();
        output.try_reserve_exact(input.len())?;
        output.extend_from_slice(&input);
        let shaped = self.shape(output)?;
        let result = Rc::new(RefCell::new(LigatureData {
            input_glyphs: Rc::new(RefCell::new(input)),
            output_glyphs: Rc::new(RefCell::new(shaped.sequence)),
            context_ranges: Rc::new(RefCell::new(shaped.ranges)),
        }));
        if let Some(cache) = &mut self.cache {
            cache.set(text, Cached::Full(Rc::clone(&result)))?;
        }
        Ok(result)
    }
    pub fn find_ligature_ranges(&mut self, text: &[u16]) -> Result<RangeArray, FontError> {
        if self.groups.is_empty() {
            return Ok(Rc::new(RefCell::new(Vec::new())));
        }
        if let Some(cached) = self.cache.as_mut().and_then(|cache| cache.get(text)) {
            return cached.ranges();
        }
        let shaped = self.shape(self.glyphs(text)?)?;
        let result = Rc::new(RefCell::new(shaped.ranges));
        if let Some(cache) = &mut self.cache {
            cache.set(text, Cached::Ranges(Rc::clone(&result)))?;
        }
        Ok(result)
    }
    fn glyphs(&self, text: &[u16]) -> Result<Vec<GlyphValue>, FontError> {
        let mut glyphs = Vec::new();
        glyphs.try_reserve_exact(text.len())?;
        let mut index = 0;
        while index < text.len() {
            let first = text[index];
            index += 1;
            let codepoint = if (0xD800..=0xDBFF).contains(&first)
                && text
                    .get(index)
                    .is_some_and(|unit| (0xDC00..=0xDFFF).contains(unit))
            {
                let second = text[index];
                index += 1;
                0x10000 + (u32::from(first - 0xD800) << 10) + u32::from(second - 0xDC00)
            } else {
                u32::from(first)
            };
            glyphs.push(GlyphValue::Number(self.cmap.glyph(codepoint)));
        }
        Ok(glyphs)
    }
}
