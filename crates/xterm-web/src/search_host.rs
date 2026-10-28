// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Shared selection/terminal adapter used by browser search and native tests.
use crate::selection::{
    model::{Dimensions, Position},
    service::SelectionService,
};
use std::{num::NonZeroU32, rc::Rc};
use xterm_addons::search::SearchAddonError;
use xterm_core::{
    decoration::DecorationService,
    headless::Terminal,
    search::{SearchPosition, SearchResult, SearchSelection},
};

pub struct SearchTerminal {
    pub native: Rc<Terminal>,
    pub selection: Rc<SelectionService>,
    pub decorations: Rc<DecorationService>,
}
impl SearchTerminal {
    pub fn selection_position(&self) -> Result<Option<SearchSelection>, SearchAddonError> {
        let dimensions = self.dimensions()?;
        if !self
            .selection
            .has_selection(dimensions)
            .map_err(host_error)?
        {
            return Ok(None);
        }
        let start = self.selection.selection_start();
        let end = self
            .selection
            .selection_end(dimensions)
            .map_err(host_error)?;
        match (start, end) {
            (Some(start), Some(end)) => Ok(Some(SearchSelection {
                start: position(start)?,
                end: position(end)?,
            })),
            _ => Ok(None),
        }
    }
    pub fn clear_selection(&self) -> Result<(), SearchAddonError> {
        let buffers = self
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))?;
        self.selection
            .clear_selection(&buffers, self.dimensions()?)
            .map_err(host_error)
    }
    pub fn select(&self, result: &SearchResult) -> Result<(), SearchAddonError> {
        let buffers = self
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))?;
        self.selection
            .set_selection(
                &buffers,
                self.dimensions()?,
                Position::new(result.col as i64, result.row as i64),
                result.size as i64,
            )
            .map_err(host_error)
    }
    pub fn dimensions(&self) -> Result<Dimensions, SearchAddonError> {
        let active = self.native.with_state(|state| state.active_buffer())?;
        let scrollback = active
            .try_borrow()
            .map_err(|_| SearchAddonError::Borrowed)?
            .ybase;
        let columns = u32::try_from(self.native.cols()?)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(SearchAddonError::InvalidCoordinates)?;
        let rows = u32::try_from(self.native.rows()?)
            .ok()
            .and_then(NonZeroU32::new)
            .ok_or(SearchAddonError::InvalidCoordinates)?;
        let scrollback =
            u32::try_from(scrollback).map_err(|_| SearchAddonError::InvalidCoordinates)?;
        Ok(Dimensions {
            columns,
            rows,
            scrollback,
        })
    }
}
fn position(value: Position) -> Result<SearchPosition, SearchAddonError> {
    Ok(SearchPosition {
        row: isize::try_from(value.row).map_err(|_| SearchAddonError::InvalidCoordinates)?,
        col: isize::try_from(value.column).map_err(|_| SearchAddonError::InvalidCoordinates)?,
    })
}
fn host_error(error: impl std::error::Error + 'static) -> SearchAddonError {
    SearchAddonError::Host(Rc::new(error))
}
