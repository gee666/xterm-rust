// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

impl SearchAddon {
    pub(super) fn find_empty(
        &self,
        options: Option<&SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        let decorations = options
            .map(|options| options.decorations())
            .transpose()?
            .flatten();
        if decorations.is_some() {
            let cached = self.inner.state.borrow().cached_search_term.clone();
            let update = cached.as_ref().is_none_or(|term| !term.is_empty())
                || options
                    .map(|options| options.flags_changed())
                    .transpose()?
                    .unwrap_or(false);
            if update {
                self.clear_decorations(false)?;
            }
        }
        self.host()?.clear_selection()?;
        self.clear_decorations(false)?;
        let decorations = options
            .map(|options| options.decorations())
            .transpose()?
            .flatten();
        self.inner
            .tracker
            .fire_results_changed(decorations.is_some())?;
        self.inner.state.borrow_mut().cached_search_term = Some(Vec::new());
        self.inner.after.fire(&())?;
        Ok(false)
    }
}
