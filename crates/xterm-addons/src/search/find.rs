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
        self.find_and_select(term, options, false, false)
    }
    pub fn find_previous(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        self.find_previous_internal(term, options, false)
    }
    pub(super) fn find_previous_internal(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
        no_scroll: bool,
    ) -> Result<bool, SearchAddonError> {
        self.find_and_select(term, options, true, no_scroll)
    }
    fn find_and_select(
        &self,
        term: &[u16],
        options: Option<SearchOptionsHandle>,
        reverse: bool,
        no_scroll: bool,
    ) -> Result<bool, SearchAddonError> {
        let host = self.host()?;
        self.inner.before.fire(&())?;
        *self.inner.last_options.borrow_mut() = options.clone();
        // The source stores the new options before asking whether they changed.
        // Preserve that order, including its unchanged-term highlight behavior.
        let flags = options
            .as_ref()
            .map(|o| o.read().map(|options| matching(&options)))
            .transpose()?;
        self.inner.state.borrow_mut().last_search_options = flags.clone();
        let update = self
            .inner
            .state
            .borrow()
            .should_update_highlighting(term, flags.as_ref());
        if update {
            if let Some(options) = &options {
                let request = options.read()?;
                self.highlight_all(term, &request)?;
            }
        }
        let request = options
            .as_ref()
            .map(|o| o.read())
            .transpose()?
            .unwrap_or_default();
        let result = if term.is_empty() {
            host.clear_selection()?;
            self.clear_decorations(false)?;
            None
        } else {
            let selection = host.selection()?;
            host.clear_selection()?;
            let cached = self.inner.state.borrow().cached_search_term.clone();
            self.run_engine(
                term,
                &matching(&request),
                if reverse {
                    Operation::Previous
                } else {
                    Operation::Next
                },
                selection,
                cached.as_deref(),
            )?
        };
        let found = if term.is_empty() {
            false
        } else {
            self.select_result(result, request.decorations.as_ref(), no_scroll)?
        };
        self.inner
            .tracker
            .fire_results_changed(request.decorations.is_some())?;
        self.inner.state.borrow_mut().cached_search_term = Some(term.to_vec());
        self.inner.after.fire(&())?;
        Ok(found)
    }
    fn highlight_all(&self, term: &[u16], options: &SearchOptions) -> Result<(), SearchAddonError> {
        if term.is_empty() {
            return self.clear_decorations(false);
        }
        self.clear_decorations(true)?;
        let flags = matching(options);
        let mut results = Vec::new();
        let mut previous: Option<SearchResult> = None;
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
            result =
                self.run_engine(term, &flags, Operation::At(next_row, next_col), None, None)?;
        }
        let end = self.inner.highlight_limit.end_index(results.len())?;
        self.inner
            .tracker
            .update_results(results.clone(), end as f64);
        let manager = self
            .inner
            .manager
            .borrow()
            .clone()
            .ok_or(SearchAddonError::NotLoaded)?;
        if let Some(decorations) = &options.decorations {
            manager.create_highlights(&results, decorations)?;
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
        self.init_cache()?;
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
fn matching(options: &SearchOptions) -> MatchOptions {
    MatchOptions {
        decorations: options.decorations.is_some(),
        ..options.matching.clone()
    }
}
