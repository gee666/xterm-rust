// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

impl SearchAddon {
    pub(super) fn finish_search(
        &self,
        term: super::super::SearchTermHandle,
        options: Option<&SearchOptionsHandle>,
    ) -> Result<(), SearchAddonError> {
        let decorations = options
            .map(|options| options.decorations())
            .transpose()?
            .flatten();
        self.inner
            .tracker
            .fire_results_changed(decorations.is_some())?;
        let cached = (!term.is_undefined()).then_some(term);
        *self.inner.cached_term.borrow_mut() = cached;
        self.inner.after.fire(&())?;
        Ok(())
    }
}
