// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_core::search::{
    with_buffer, SearchEngine, SearchOptions as MatchOptions, SearchResult, SearchSelection,
};

enum Operation {
    At(isize, isize),
    Next,
    Previous,
}

impl SearchAddon {
    pub fn find_next(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        self.find_next_with_scroll(term, options, || Ok(false))
    }
    /// Read the host's internal `noScroll` option after matching and before
    /// changing the selected result. An empty term never reads this option.
    pub fn find_next_with_scroll(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
        no_scroll: impl FnOnce() -> Result<bool, SearchAddonError>,
    ) -> Result<bool, SearchAddonError> {
        self.find_term_with_scroll(
            Rc::new(super::super::term::NativeTerm::Text(term.to_vec())),
            options,
            false,
            no_scroll,
        )
    }
    pub fn find_previous(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        self.find_previous_with_scroll(term, options, || Ok(false))
    }
    /// Reverse-search counterpart of [`Self::find_next_with_scroll`].
    pub fn find_previous_with_scroll(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
        no_scroll: impl FnOnce() -> Result<bool, SearchAddonError>,
    ) -> Result<bool, SearchAddonError> {
        self.find_term_with_scroll(
            Rc::new(super::super::term::NativeTerm::Text(term.to_vec())),
            options,
            true,
            no_scroll,
        )
    }
    /// Search a retained host term. Validation follows the source highlight and
    /// selection phases, rather than coercing the argument at the API boundary.
    pub fn find_term_with_scroll(
        &self,
        term: super::super::SearchTermHandle,
        options: Option<SearchOptionsHandle>,
        reverse: bool,
        no_scroll: impl FnOnce() -> Result<bool, SearchAddonError>,
    ) -> Result<bool, SearchAddonError> {
        let host = self.host()?;
        self.inner.before.fire(&())?;
        *self.inner.last_options.borrow_mut() = options.clone();
        let decorations = options
            .as_ref()
            .map(|options| options.decorations())
            .transpose()?
            .flatten();
        if decorations.is_some() {
            let cached = self.inner.cached_term.borrow().clone();
            let update = cached
                .as_ref()
                .is_none_or(|cached| !term.equals(cached.as_ref()))
                || {
                    // A decoration getter can replace the stored request via
                    // a nested search before the source compares any flags.
                    let last = self.inner.last_options.borrow().clone();
                    last.is_none_or(|options| !options.is_truthy())
                }
                || options
                    .as_ref()
                    .filter(|options| options.is_truthy())
                    .map(|options| {
                        options.flags_changed(&|| self.inner.last_options.borrow().clone())
                    })
                    .transpose()?
                    .unwrap_or(false);
            if update {
                if !term.is_valid()? {
                    self.clear_decorations(false)?;
                } else if let Some(options) = &options {
                    self.highlight_all(&term.text()?, options)?;
                }
            }
        }
        if !term.is_valid()? {
            host.clear_selection()?;
            self.clear_decorations(false)?;
            self.finish_search(term, options.as_ref())?;
            return Ok(false);
        }
        let text = term.text()?;
        let selection = host.selection()?;
        host.clear_selection()?;
        // Matching option access can throw or reenter. The source has already
        // captured and cleared selection and initialized its line cache here.
        self.init_cache()?;
        let flags = options
            .as_ref()
            .map(|options| options.matching_options())
            .transpose()?
            .unwrap_or_default();
        let cached = self.inner.cached_term.borrow().clone();
        let same = cached
            .as_ref()
            .is_some_and(|cached| term.equals(cached.as_ref()));
        let result = self.run_engine(
            &text,
            &flags,
            if reverse {
                Operation::Previous
            } else {
                Operation::Next
            },
            selection,
            same.then_some(text.as_ref()),
        )?;
        // Source evaluates selection decorations after the engine returns,
        // even for a miss, and before reading internal noScroll or selecting.
        // No terminal/cache borrow may remain live across this getter.
        let decorations = options
            .as_ref()
            .map(|options| options.decorations())
            .transpose()?
            .flatten();
        let found = self.select_result(result, decorations.as_ref(), no_scroll()?)?;
        drop(text);
        self.finish_search(term, options.as_ref())?;
        Ok(found)
    }
    fn highlight_all(
        &self,
        term: &[u16],
        options: &SearchOptionsHandle,
    ) -> Result<(), SearchAddonError> {
        if term.is_empty() {
            return self.clear_decorations(false);
        }
        self.clear_decorations(true)?;
        let flags = options.matching_options()?;
        let mut results = Vec::new();
        let mut previous: Option<SearchResult> = None;
        self.init_cache()?;
        let mut result = self.run_engine(term, &flags, Operation::At(0, 0), None, None)?;
        while let Some(found) = result {
            if previous
                .as_ref()
                .is_some_and(|previous| previous.row == found.row && previous.col == found.col)
                || self.inner.highlight_limit.reached(results.len())?
            {
                break;
            }
            let cols = isize::try_from(self.host()?.terminal().cols()?)
                .map_err(|_| SearchAddonError::InvalidCoordinates)?;
            let mut next_col = found
                .col
                .checked_add(found.size)
                .ok_or(SearchAddonError::InvalidCoordinates)?;
            let mut next_row = found.row;
            if next_col >= cols {
                next_row = next_row
                    .checked_add(next_col / cols)
                    .ok_or(SearchAddonError::InvalidCoordinates)?;
                next_col %= cols;
            }
            previous = Some(found.clone());
            results.push(found);
            self.init_cache()?;
            result =
                self.run_engine(term, &flags, Operation::At(next_row, next_col), None, None)?;
        }
        let end = self.inner.highlight_limit.end_index(results.len())?;
        self.inner
            .tracker
            .update_results(results.clone(), end as f64);
        // Source stores results before eligibility, then captures the manager
        // before reading the call argument. Either getter may reenter, replace
        // that manager, return a falsy value, or throw.
        if options.decorations()?.is_some() {
            let manager = self
                .inner
                .manager
                .borrow()
                .clone()
                .ok_or(SearchAddonError::NotLoaded)?;
            manager.create_highlights(&results, &options.highlight_decorations()?)?;
        }
        Ok(())
    }
    fn select_result(
        &self,
        result: Option<SearchResult>,
        decorations: Option<&super::super::SearchDecorationOptionsHandle>,
        no_scroll: bool,
    ) -> Result<bool, SearchAddonError> {
        self.inner.tracker.clear_selected()?;
        let host = self.host()?;
        let Some(result) = result else {
            host.clear_selection()?;
            return Ok(false);
        };
        host.select(&result)?;
        if let Some(decorations) = decorations {
            let manager = self
                .inner
                .manager
                .borrow()
                .clone()
                .ok_or(SearchAddonError::NotLoaded)?;
            if let Some(active) = manager.create_active(result.clone(), decorations)? {
                self.inner.tracker.set_selected(Some(active));
            }
        }
        if !no_scroll {
            let terminal = host.terminal();
            let active = terminal.with_state(|state| state.active_buffer())?;
            let viewport = active
                .try_borrow()
                .map_err(|_| SearchAddonError::Borrowed)?
                .ydisp as isize;
            let rows = isize::try_from(terminal.rows()?)
                .map_err(|_| SearchAddonError::InvalidCoordinates)?;
            if result.row >= viewport + rows || result.row < viewport {
                terminal.scroll_lines((result.row - viewport - rows / 2) as i64)?;
            }
        }
        Ok(true)
    }
    fn run_engine(
        &self,
        term: &[u16],
        options: &MatchOptions,
        operation: Operation,
        selection: Option<SearchSelection>,
        cached: Option<&[u16]>,
    ) -> Result<Option<SearchResult>, SearchAddonError> {
        let host = self.host()?;
        let terminal = host.terminal();
        let active = terminal.with_state(|state| state.active_buffer())?;
        let buffer = active
            .try_borrow()
            .map_err(|_| SearchAddonError::Borrowed)?;
        let mut cache = self
            .inner
            .cache
            .try_borrow_mut()
            .map_err(|_| SearchAddonError::Borrowed)?;
        Ok(with_buffer(&buffer, terminal.cols()?, |view| {
            let mut engine = SearchEngine::new(view, &mut cache, host.now_ms());
            match operation {
                Operation::At(row, col) => engine.find(term, row, col, options),
                Operation::Next => engine.find_next(term, options, cached, selection),
                Operation::Previous => engine.find_previous(term, options, cached, selection),
            }
        })?)
    }
}
