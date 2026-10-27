// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::SearchOptions;

#[derive(Default, Debug)]
pub struct SearchState {
    pub cached_search_term: Option<Vec<u16>>,
    pub last_search_options: Option<SearchOptions>,
}

impl SearchState {
    pub fn is_valid_search_term(&self, term: &[u16]) -> bool {
        !term.is_empty()
    }
    pub fn did_options_change(&self, new: Option<&SearchOptions>) -> bool {
        let Some(old) = &self.last_search_options else {
            return true;
        };
        let Some(new) = new else { return false };
        old.case_sensitive != new.case_sensitive
            || old.regex != new.regex
            || old.whole_word != new.whole_word
    }
    pub fn should_update_highlighting(
        &self,
        term: &[u16],
        options: Option<&SearchOptions>,
    ) -> bool {
        options.is_some_and(|o| o.decorations)
            && (self.cached_search_term.as_deref() != Some(term)
                || self.did_options_change(options))
    }
    pub fn clear_cached_term(&mut self) {
        self.cached_search_term = None;
    }
    pub fn reset(&mut self) {
        self.cached_search_term = None;
        self.last_search_options = None;
    }
}
