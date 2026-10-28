// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{SearchAddonError, SearchColor, SearchDecorationOptionsHandle, SearchHost};
use std::{cell::RefCell, collections::HashSet, rc::Rc};
use xterm_core::{
    buffer::{marker::Marker, marker_line::MarkerLineKey},
    decoration::{
        Decoration, DecorationError, DecorationLayer, DecorationOptions, OverviewPosition,
        OverviewRulerOptions,
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
    highlighted_lines: RefCell<HashSet<MarkerLineKey>>,
}
impl DecorationManager {
    pub fn new(host: Rc<dyn SearchHost>) -> Self {
        Self {
            host,
            highlights: RefCell::new(Vec::new()),
            highlighted_lines: RefCell::new(HashSet::new()),
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
        options: &SearchDecorationOptionsHandle,
    ) -> Result<(), SearchAddonError> {
        self.clear_highlights()?;
        for result in results {
            for item in self.create(result, options, false)? {
                self.highlighted_lines
                    .borrow_mut()
                    .insert(item.decoration.marker().line_value().map_key());
                self.highlights.borrow_mut().push(item);
            }
        }
        Ok(())
    }
    pub fn create_active(
        &self,
        result: SearchResult,
        options: &SearchDecorationOptionsHandle,
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
        options: &SearchDecorationOptionsHandle,
        active: bool,
    ) -> Result<Vec<ManagedDecoration>, SearchAddonError> {
        let terminal = self.host.terminal();
        let cols =
            isize::try_from(terminal.cols()?).map_err(|_| SearchAddonError::InvalidCoordinates)?;
        if cols == 0 {
            return Err(SearchAddonError::InvalidCoordinates);
        }
        let mut col = result.col;
        let mut remaining = result.size;
        let buffer = terminal.with_state(|state| state.active_buffer())?;
        let mut marker_offset = {
            let buffer = buffer
                .try_borrow()
                .map_err(|_| SearchAddonError::Borrowed)?;
            let row =
                i64::try_from(result.row).map_err(|_| SearchAddonError::InvalidCoordinates)?;
            let base =
                i64::try_from(buffer.ybase).map_err(|_| SearchAddonError::InvalidCoordinates)?;
            let cursor =
                i64::try_from(buffer.y).map_err(|_| SearchAddonError::InvalidCoordinates)?;
            row.checked_sub(base)
                .and_then(|row| row.checked_sub(cursor))
                .ok_or(SearchAddonError::InvalidCoordinates)?
        };
        let mut ranges = Vec::new();
        while remaining > 0 {
            let width = cols
                .checked_sub(col)
                .ok_or(SearchAddonError::InvalidCoordinates)?
                .min(remaining);
            // At col==cols the source first emits a zero-width range, then
            // resets to col zero. Do not turn that valid source quirk into Err.
            ranges.push((marker_offset, col, width));
            col = 0;
            remaining = remaining
                .checked_sub(width)
                .ok_or(SearchAddonError::InvalidCoordinates)?;
            if remaining > 0 {
                marker_offset = marker_offset
                    .checked_add(1)
                    .ok_or(SearchAddonError::InvalidCoordinates)?;
            }
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
            decoration_options.background_color = options.color(if active {
                SearchColor::ActiveBackground
            } else {
                SearchColor::MatchBackground
            })?;
            let needs_overview = !self
                .highlighted_lines
                .borrow()
                .contains(&marker.line_value().map_key());
            if needs_overview {
                decoration_options.overview_ruler_options = Some(OverviewRulerOptions {
                    color: options
                        .color(if active {
                            SearchColor::ActiveOverview
                        } else {
                            SearchColor::MatchOverview
                        })?
                        .unwrap_or_default(),
                    position: Some(OverviewPosition::Center),
                });
            }
            // Source calls the public Terminal.registerDecoration wrapper after
            // evaluating option getters, so preserve its validation and order.
            xterm_core::public_api::validation::verify_positive_integers(&[
                col as f64,
                width as f64,
                1.0,
            ])
            .map_err(|error| SearchAddonError::Host(Rc::new(error)))?;
            let decoration = self
                .host
                .decorations()
                .register_decoration(Rc::new(RefCell::new(decoration_options)))
                .map_err(|error| match error {
                    DecorationError::Dispatch(error) => SearchAddonError::Dispatch(error),
                    other => SearchAddonError::Host(Rc::new(other)),
                })?;
            if let Some(decoration) = decoration {
                let resources = Rc::new(Resources {
                    marker,
                    rendered: RefCell::new(None),
                    disposed: RefCell::new(None),
                });
                let weak_host = Rc::downgrade(&self.host);
                let colors = Rc::clone(options);
                *resources.rendered.borrow_mut() =
                    Some(decoration.on_render().try_subscribe(move |element| {
                        // Upstream passes false to _applyStyles even for an active
                        // result. Preserve that behavior, not its unused active class.
                        if let Some(host) = weak_host.upgrade() {
                            let border = colors.color(if active {
                                SearchColor::ActiveBorder
                            } else {
                                SearchColor::MatchBorder
                            })?;
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
