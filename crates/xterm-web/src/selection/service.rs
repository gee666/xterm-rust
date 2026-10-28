// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Selection state and synchronous events. The DOM adapter owns scheduling/listeners.
#[path = "service_actions.rs"]
mod actions;
#[path = "service_pointer.rs"]
mod pointer;

use super::{
    model::{CoordinateOverflow, Dimensions, Position, SelectionModel},
    text, word,
};
use std::{
    cell::{Cell, RefCell},
    error::Error,
    fmt,
};
use xterm_core::lifecycle::Disposable;
use xterm_core::{
    buffer::{Buffer, BufferError},
    event::{EmitError, Emitter, Event},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionMode {
    #[default]
    Normal,
    Word,
    Line,
    Column,
}

#[derive(Debug)]
pub enum SelectionError {
    Coordinates(CoordinateOverflow),
    Buffer(BufferError),
    Dispatch(EmitError),
    Arithmetic,
    Allocation(std::collections::TryReserveError),
    Borrowed,
}
impl fmt::Display for SelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Coordinates(error) => error.fmt(f),
            Self::Buffer(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
            Self::Arithmetic => f.write_str("selection coordinate arithmetic overflow"),
            Self::Allocation(error) => error.fmt(f),
            Self::Borrowed => f.write_str("selection buffer is already borrowed"),
        }
    }
}
impl Error for SelectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Coordinates(error) => Some(error),
            Self::Buffer(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Allocation(error) => Some(error),
            Self::Arithmetic | Self::Borrowed => None,
        }
    }
}
impl From<CoordinateOverflow> for SelectionError {
    fn from(value: CoordinateOverflow) -> Self {
        Self::Coordinates(value)
    }
}
impl From<BufferError> for SelectionError {
    fn from(value: BufferError) -> Self {
        Self::Buffer(value)
    }
}
impl From<EmitError> for SelectionError {
    fn from(value: EmitError) -> Self {
        Self::Dispatch(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionPlatform {
    pub windows: bool,
    pub linux: bool,
    pub mac: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedrawRequest {
    pub start: Option<Position>,
    pub end: Option<Position>,
    pub column_select_mode: bool,
}
#[derive(Clone, Copy, Debug, Default)]
struct PreviousSelection {
    start: Option<Position>,
    end: Option<Position>,
    has_selection: bool,
}

/// No RefCell borrow or buffer borrow is held across a user event callback.
/// The host connects refresh requests to animation frames and drag-stop to DOM
/// listener/timer disposal. `flush_refresh` reads live dimensions at frame time.
pub struct SelectionService {
    model: RefCell<SelectionModel>,
    mode: Cell<SelectionMode>,
    enabled: Cell<bool>,
    platform: SelectionPlatform,
    previous: Cell<PreviousSelection>,
    refresh_pending: Cell<bool>,
    refresh_request: Emitter<()>,
    redraw: Emitter<RedrawRequest>,
    selection_change: Emitter<()>,
    linux_selection: Emitter<Vec<u16>>,
    drag_stop: Emitter<()>,
}
impl SelectionService {
    pub fn new(platform: SelectionPlatform) -> Self {
        Self {
            model: RefCell::new(SelectionModel::default()),
            mode: Cell::new(SelectionMode::Normal),
            enabled: Cell::new(true),
            platform,
            previous: Cell::new(PreviousSelection::default()),
            refresh_pending: Cell::new(false),
            refresh_request: Emitter::new(),
            redraw: Emitter::new(),
            selection_change: Emitter::new(),
            linux_selection: Emitter::new(),
            drag_stop: Emitter::new(),
        }
    }
    pub fn model(&self) -> SelectionModel {
        self.model.borrow().clone()
    }
    /// Host updates only coordinate state; no callback runs inside this closure.
    pub fn update_model(&self, update: impl FnOnce(&mut SelectionModel)) {
        update(&mut self.model.borrow_mut());
    }
    pub fn mode(&self) -> SelectionMode {
        self.mode.get()
    }
    pub fn set_mode(&self, mode: SelectionMode) {
        self.mode.set(mode);
    }
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }
    pub fn enable(&self) {
        self.enabled.set(true);
    }
    pub fn on_request_refresh(&self) -> Event<()> {
        self.refresh_request.event()
    }
    pub fn on_request_redraw(&self) -> Event<RedrawRequest> {
        self.redraw.event()
    }
    pub fn on_selection_change(&self) -> Event<()> {
        self.selection_change.event()
    }
    pub fn on_linux_mouse_selection(&self) -> Event<Vec<u16>> {
        self.linux_selection.event()
    }
    pub fn on_stop_drag(&self) -> Event<()> {
        self.drag_stop.event()
    }
    pub fn selection_start(&self) -> Option<Position> {
        self.model.borrow().final_selection_start()
    }
    pub fn selection_end(
        &self,
        dimensions: Dimensions,
    ) -> Result<Option<Position>, SelectionError> {
        Ok(self.model.borrow().final_selection_end(dimensions)?)
    }
    pub fn has_selection(&self, dimensions: Dimensions) -> Result<bool, SelectionError> {
        let model = self.model();
        Ok(
            matches!((model.final_selection_start(), model.final_selection_end(dimensions)?), (Some(start), Some(end)) if start != end),
        )
    }
    pub fn selection_text(
        &self,
        buffer: &Buffer,
        dimensions: Dimensions,
    ) -> Result<Vec<u16>, SelectionError> {
        text::selection_text(
            buffer,
            &self.model(),
            dimensions,
            self.mode(),
            self.platform.windows,
        )
    }
    pub fn is_cell_in_selection(
        &self,
        point: Position,
        dimensions: Dimensions,
    ) -> Result<bool, SelectionError> {
        let model = self.model();
        Ok(
            match (
                model.final_selection_start(),
                model.final_selection_end(dimensions)?,
            ) {
                (Some(start), Some(end)) => are_coords_in_selection(point, start, end),
                _ => false,
            },
        )
    }
    pub fn refresh(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
        linux_mouse_selection: bool,
    ) -> Result<(), SelectionError> {
        if !self.refresh_pending.replace(true) {
            if let Err(error) = self.refresh_request.fire(&()) {
                self.refresh_pending.set(false);
                return Err(error.into());
            }
        }
        if self.platform.linux && linux_mouse_selection {
            let text = {
                let active = buffer.buffer();
                let active = active.try_borrow().map_err(|_| SelectionError::Borrowed)?;
                self.selection_text(&active, dimensions)?
            };
            if !text.is_empty() {
                self.linux_selection.fire(&text)?;
            }
        }
        Ok(())
    }
    pub fn flush_refresh(&self, dimensions: Dimensions) -> Result<(), SelectionError> {
        self.refresh_pending.set(false);
        let model = self.model();
        self.redraw.fire(&RedrawRequest {
            start: model.final_selection_start(),
            end: model.final_selection_end(dimensions)?,
            column_select_mode: self.mode() == SelectionMode::Column,
        })?;
        Ok(())
    }
    pub fn fire_if_changed(&self, dimensions: Dimensions) -> Result<(), SelectionError> {
        let model = self.model();
        let start = model.final_selection_start();
        let end = model.final_selection_end(dimensions)?;
        let has_selection = matches!((start, end), (Some(start), Some(end)) if start != end);
        let old = self.previous.get();
        if (!has_selection && old.has_selection)
            || (has_selection && (old.start != start || old.end != end))
        {
            self.previous.set(PreviousSelection {
                start,
                end,
                has_selection,
            });
            self.selection_change.fire(&())?;
        }
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), SelectionError> {
        self.drag_stop.fire(&())?;
        self.refresh_request.dispose();
        self.redraw.dispose();
        self.selection_change.dispose();
        self.linux_selection.dispose();
        self.drag_stop.dispose();
        Ok(())
    }
}
fn normalized_word_start(row: i64, start: i64, columns: u32) -> Result<Position, SelectionError> {
    let columns = i64::from(columns);
    let offset = start.div_euclid(columns);
    Ok(Position::new(
        start.rem_euclid(columns),
        row.checked_add(offset).ok_or(SelectionError::Arithmetic)?,
    ))
}
pub fn buffer_dimensions(
    buffers: &xterm_core::buffer::BufferService,
) -> Result<Dimensions, SelectionError> {
    let columns = u32::try_from(buffers.cols()).map_err(|_| SelectionError::Arithmetic)?;
    let rows = u32::try_from(buffers.rows()).map_err(|_| SelectionError::Arithmetic)?;
    let active = buffers.buffer();
    let scrollback = u32::try_from(
        active
            .try_borrow()
            .map_err(|_| SelectionError::Borrowed)?
            .ybase,
    )
    .map_err(|_| SelectionError::Arithmetic)?;
    Ok(Dimensions {
        columns: std::num::NonZeroU32::new(columns).ok_or(SelectionError::Arithmetic)?,
        rows: std::num::NonZeroU32::new(rows).ok_or(SelectionError::Arithmetic)?,
        scrollback,
    })
}

pub fn are_coords_in_selection(point: Position, start: Position, end: Position) -> bool {
    (point.row > start.row && point.row < end.row)
        || (start.row == end.row
            && point.row == start.row
            && point.column >= start.column
            && point.column < end.column)
        || (start.row < end.row && point.row == end.row && point.column < end.column)
        || (start.row < end.row && point.row == start.row && point.column >= start.column)
}
