// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{SearchAddonError, SearchDecorationOptions, SearchHost};
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
use xterm_core::{
    buffer::marker::Marker,
    decoration::{
        Decoration, DecorationLayer, DecorationOptions, OverviewPosition, OverviewRulerOptions,
    },
    event::EmitError,
    lifecycle::DisposableHandle,
    search::SearchResult,
};

struct Resources {
    marker: Marker,
    rendered: RefCell<Option<DisposableHandle>>,
    disposed: RefCell<Option<DisposableHandle>>,
}
impl Resources {
    fn cleanup(&self) -> Result<(), EmitError> {
        self.marker.dispose()?;
        let rendered = self.rendered.borrow().clone();
        if let Some(rendered) = rendered {
            rendered.dispose();
        }
        let disposed = self.disposed.borrow().clone();
        if let Some(disposed) = disposed {
            disposed.dispose();
        }
        Ok(())
    }
}
impl Drop for Resources {
    fn drop(&mut self) {
        // Subscription removal does not invoke user callbacks. Explicit fallible
        // disposal owns marker callbacks; Drop only prevents retained listeners.
        if let Some(handle) = self.rendered.get_mut().take() {
            handle.dispose();
        }
        if let Some(handle) = self.disposed.get_mut().take() {
            handle.dispose();
        }
    }
}
#[derive(Clone)]
struct ManagedDecoration {
    decoration: Decoration,
    _resources: Rc<Resources>,
}

pub struct MultiHighlight {
    pub search_match: SearchResult,
    decorations: Vec<ManagedDecoration>,
}
impl MultiHighlight {
    pub fn decorations(&self) -> Vec<Decoration> {
        self.decorations
            .iter()
            .map(|item| item.decoration.clone())
            .collect()
    }
    pub fn dispose(&self) -> Result<(), EmitError> {
        for item in &self.decorations {
            item.decoration.dispose()?;
        }
        Ok(())
    }
}

pub struct DecorationManager {
    host: Rc<dyn SearchHost>,
    highlights: RefCell<Vec<ManagedDecoration>>,
    highlighted_lines: RefCell<BTreeSet<i64>>,
}
impl DecorationManager {
    pub fn new(host: Rc<dyn SearchHost>) -> Self {
        Self {
            host,
            highlights: RefCell::new(Vec::new()),
            highlighted_lines: RefCell::new(BTreeSet::new()),
        }
    }
    pub fn clear_highlights(&self) -> Result<(), EmitError> {
        let highlights = self.highlights.borrow().clone();
        for item in highlights {
            item.decoration.dispose()?;
        }
        self.highlights.borrow_mut().clear();
        self.highlighted_lines.borrow_mut().clear();
        Ok(())
    }
    pub fn create_highlights(
        &self,
        results: &[SearchResult],
        options: &SearchDecorationOptions,
    ) -> Result<(), SearchAddonError> {
        self.clear_highlights()?;
        for result in results {
            for item in self.create(result, options, false)? {
                self.highlighted_lines
                    .borrow_mut()
                    .insert(item.decoration.marker().line());
                self.highlights.borrow_mut().push(item);
            }
        }
        Ok(())
    }
    pub fn create_active(
        &self,
        result: SearchResult,
        options: &SearchDecorationOptions,
    ) -> Result<Option<Rc<MultiHighlight>>, SearchAddonError> {
        let decorations = self.create(&result, options, true)?;
        Ok((!decorations.is_empty()).then(|| {
            Rc::new(MultiHighlight {
                search_match: result,
                decorations,
            })
        }))
    }
    fn create(
        &self,
        result: &SearchResult,
        options: &SearchDecorationOptions,
        active: bool,
    ) -> Result<Vec<ManagedDecoration>, SearchAddonError> {
        let terminal = self.host.terminal();
        let cols =
            isize::try_from(terminal.cols()?).map_err(|_| SearchAddonError::InvalidCoordinates)?;
        let mut col = result.col;
        let mut remaining = result.size;
        let buffer = terminal.with_state(|state| state.active_buffer())?;
        let mut marker_offset = {
            let buffer = buffer
                .try_borrow()
                .map_err(|_| SearchAddonError::Borrowed)?;
            result.row as i64 - buffer.ybase as i64 - buffer.y as i64
        };
        let mut ranges = Vec::new();
        while remaining > 0 {
            let width = (cols - col).min(remaining);
            if width <= 0 {
                return Err(SearchAddonError::InvalidCoordinates);
            }
            ranges.push((marker_offset, col, width));
            col = 0;
            remaining -= width;
            marker_offset += 1;
        }
        let mut decorations = Vec::new();
        for (offset, col, width) in ranges {
            let marker = terminal.register_marker(offset)?;
            let mut decoration_options = DecorationOptions::new(marker.clone());
            decoration_options.x = Some(col as f64);
            decoration_options.width = Some(width as f64);
            decoration_options.layer = Some(if active {
                DecorationLayer::Top
            } else {
                DecorationLayer::Bottom
            });
            decoration_options.background_color = if active {
                options.active_match_background.clone()
            } else {
                options.match_background.clone()
            };
            if !self.highlighted_lines.borrow().contains(&marker.line()) {
                decoration_options.overview_ruler_options = Some(OverviewRulerOptions {
                    color: if active {
                        options.active_match_overview_ruler.clone()
                    } else {
                        options.match_overview_ruler.clone()
                    },
                    position: Some(OverviewPosition::Center),
                });
            }
            let decoration = self
                .host
                .decorations()
                .register_decoration(Rc::new(RefCell::new(decoration_options)))
                .map_err(|error| SearchAddonError::Host(Rc::new(error)))?;
            if let Some(decoration) = decoration {
                let resources = Rc::new(Resources {
                    marker,
                    rendered: RefCell::new(None),
                    disposed: RefCell::new(None),
                });
                let weak_host = Rc::downgrade(&self.host);
                let border = if active {
                    options.active_match_border.clone()
                } else {
                    options.match_border.clone()
                };
                *resources.rendered.borrow_mut() =
                    Some(decoration.on_render().try_subscribe(move |element| {
                        // Upstream passes false to _applyStyles even for an active
                        // result. Preserve that behavior, not its unused active class.
                        if let Some(host) = weak_host.upgrade() {
                            host.style_decoration(element, border.as_deref())?;
                        }
                        Ok(())
                    }));
                let weak_resources = Rc::downgrade(&resources);
                *resources.disposed.borrow_mut() =
                    Some(decoration.on_dispose().try_subscribe(move |_| {
                        if let Some(resources) = weak_resources.upgrade() {
                            resources.cleanup()?;
                        }
                        Ok(())
                    }));
                decorations.push(ManagedDecoration {
                    decoration,
                    _resources: resources,
                });
            }
        }
        Ok(decorations)
    }
}
