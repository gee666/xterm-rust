// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::MultiHighlight;
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
    search::SearchResult,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchResultChange {
    pub result_index: isize,
    pub result_count: usize,
}

/// Interior mutability is confined to state changes, never held during a user
/// callback or decoration disposal, which can synchronously reenter the addon.
pub struct SearchResultTracker {
    results: RefCell<Vec<SearchResult>>,
    selected: RefCell<Option<Rc<MultiHighlight>>>,
    changed: Emitter<SearchResultChange>,
}
impl Default for SearchResultTracker {
    fn default() -> Self {
        Self {
            results: RefCell::new(Vec::new()),
            selected: RefCell::new(None),
            changed: Emitter::new(),
        }
    }
}
impl SearchResultTracker {
    pub fn on_did_change_results(&self) -> Event<SearchResultChange> {
        self.changed.event()
    }
    pub fn results(&self) -> Vec<SearchResult> {
        self.results.borrow().clone()
    }
    pub fn selected(&self) -> Option<Rc<MultiHighlight>> {
        self.selected.borrow().clone()
    }
    /// Assignment alone does not dispose the previous object, matching upstream.
    pub fn set_selected(&self, selected: Option<Rc<MultiHighlight>>) {
        *self.selected.borrow_mut() = selected;
    }
    pub fn update_results(&self, mut results: Vec<SearchResult>, maximum: f64) {
        let length = results.len() as f64;
        let end = if maximum.is_nan() {
            0.0
        } else if maximum < 0.0 {
            (length + maximum.trunc()).max(0.0)
        } else {
            maximum.trunc().min(length)
        };
        results.truncate(end as usize);
        *self.results.borrow_mut() = results;
    }
    pub fn clear_results(&self) {
        self.results.borrow_mut().clear();
    }
    pub fn clear_selected(&self) -> Result<(), EmitError> {
        if let Some(selected) = self.selected() {
            selected.dispose()?;
            self.selected.borrow_mut().take();
        }
        Ok(())
    }
    pub fn find_result_index(&self, result: &SearchResult) -> isize {
        self.results
            .borrow()
            .iter()
            .position(|candidate| {
                candidate.row == result.row
                    && candidate.col == result.col
                    && candidate.size == result.size
            })
            .map_or(-1, |index| index as isize)
    }
    pub fn fire_results_changed(&self, has_decorations: bool) -> Result<(), EmitError> {
        if !has_decorations {
            return Ok(());
        }
        let index = self
            .selected()
            .map_or(-1, |s| self.find_result_index(&s.search_match));
        let count = self.results.borrow().len();
        self.changed.fire(&SearchResultChange {
            result_index: index,
            result_count: count,
        })
    }
    pub fn reset(&self) -> Result<(), EmitError> {
        self.clear_selected()?;
        self.clear_results();
        Ok(())
    }
}
impl Disposable for SearchResultTracker {
    fn dispose(&self) {
        self.changed.dispose();
    }
}
