// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Buffer dimensions, full/partial-region scrolling and display tracking.
use super::{
    buffer_set::{BufferRef, CleanupFactory, CleanupFactoryHandle},
    screen::LineRef,
    AttributeData, BufferError, BufferSet,
};
use crate::{
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
    options::OptionsService,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

pub const MINIMUM_COLS: usize = 2;
pub const MINIMUM_ROWS: usize = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferResizeEvent {
    pub cols: usize,
    pub rows: usize,
    pub cols_changed: bool,
    pub rows_changed: bool,
}

pub struct BufferService {
    pub buffers: Rc<BufferSet>,
    pub is_user_scrolling: Cell<bool>,
    cached_blank_line: RefCell<Option<LineRef>>,
    resize_event: Emitter<BufferResizeEvent>,
    scroll_event: Rc<Emitter<usize>>,
    activation_listener: DisposableHandle,
    disposed: Cell<bool>,
}

impl BufferService {
    pub fn new(options: Rc<OptionsService>) -> Result<Self, BufferError> {
        let cols = dimension(&options, "cols", MINIMUM_COLS)?;
        let rows = dimension(&options, "rows", MINIMUM_ROWS)?;
        let buffers = Rc::new(BufferSet::new(cols, rows, options)?);
        let scroll_event = Rc::new(Emitter::new());
        let emitter = Rc::clone(&scroll_event);
        let activation_listener = buffers.on_buffer_activate().try_subscribe(move |event| {
            let display = event
                .active_buffer
                .try_borrow()
                .map_err(EmitError::callback)?
                .ydisp;
            emitter.fire(&display)
        });
        Ok(Self {
            buffers,
            is_user_scrolling: Cell::new(false),
            cached_blank_line: RefCell::new(None),
            resize_event: Emitter::new(),
            scroll_event,
            activation_listener,
            disposed: Cell::new(false),
        })
    }
    pub fn cols(&self) -> usize {
        self.buffers.dimensions().cols.get()
    }
    pub fn rows(&self) -> usize {
        self.buffers.dimensions().rows.get()
    }
    pub fn install_cleanup_factory(
        &self,
        factory: CleanupFactory,
    ) -> Result<CleanupFactoryHandle, BufferError> {
        self.buffers.install_cleanup_factory(factory)
    }
    pub fn buffer(&self) -> BufferRef {
        self.buffers.active()
    }
    pub fn on_resize(&self) -> Event<BufferResizeEvent> {
        self.resize_event.event()
    }
    pub fn on_scroll(&self) -> Event<usize> {
        self.scroll_event.event()
    }

    /// Dimensions are committed before buffer reflow and the resize event, even
    /// when resizing fails. This matches the source's update-before-call order.
    pub fn resize(&self, cols: usize, rows: usize) -> Result<(), BufferError> {
        if cols == 0 || rows == 0 {
            return Err(BufferError::InvalidDimensions);
        }
        let event = BufferResizeEvent {
            cols,
            rows,
            cols_changed: self.cols() != cols,
            rows_changed: self.rows() != rows,
        };
        let dimensions = self.buffers.dimensions();
        dimensions.cols.set(cols);
        dimensions.rows.set(rows);
        self.buffers.resize(cols, rows)?;
        self.resize_event.fire(&event)?;
        Ok(())
    }
    pub fn reset(&self) -> Result<(), BufferError> {
        self.buffers.reset()?;
        self.is_user_scrolling.set(false);
        Ok(())
    }

    pub fn scroll(&self, erase_attr: &AttributeData, is_wrapped: bool) -> Result<(), BufferError> {
        let active = self.buffer();
        let (new_line, lines, top, bottom, full_screen, trim) = {
            let buffer = active.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            let cached = self.cached_blank_line.borrow().clone();
            let new_line = if let Some(line) = cached {
                let reusable = {
                    let line = line.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
                    line.len() == self.cols()
                        && line.get_fg(0) == Some(erase_attr.fg)
                        && line.get_bg(0) == Some(erase_attr.bg)
                };
                if reusable {
                    line.try_borrow_mut()
                        .map_err(|_| BufferError::BorrowedLine)?
                        .is_wrapped = is_wrapped;
                    line
                } else {
                    buffer.get_blank_line(erase_attr, is_wrapped)
                }
            } else {
                buffer.get_blank_line(erase_attr, is_wrapped)
            };
            *self.cached_blank_line.borrow_mut() = Some(Rc::clone(&new_line));
            let top = buffer
                .ybase
                .checked_add(buffer.scroll_top)
                .ok_or(BufferError::SizeOverflow)?;
            let bottom = buffer
                .ybase
                .checked_add(buffer.scroll_bottom)
                .ok_or(BufferError::SizeOverflow)?;
            (
                new_line,
                buffer.lines.share(),
                top,
                bottom,
                buffer.scroll_top == 0,
                buffer.lines.is_full(),
            )
        };
        // List mutation emits synchronous trim/insert callbacks. Keep the Buffer
        // entirely unborrowed so observers can inspect or mutate its partial state.
        if full_screen {
            if bottom.checked_add(1) == Some(lines.len()) {
                if trim {
                    let recycled = lines.recycle()?.ok_or(BufferError::MissingLine(bottom))?;
                    let source = new_line
                        .try_borrow()
                        .map_err(|_| BufferError::BorrowedLine)?;
                    recycled
                        .try_borrow_mut()
                        .map_err(|_| BufferError::BorrowedLine)?
                        .copy_from(&source, true);
                } else {
                    lines.push(clone_blank(&new_line)?)?;
                }
            } else {
                lines.splice(
                    bottom.checked_add(1).ok_or(BufferError::SizeOverflow)?,
                    0,
                    &[clone_blank(&new_line)?],
                )?;
            }
        } else {
            let height = bottom
                .checked_sub(top)
                .and_then(|n| n.checked_add(1))
                .ok_or(BufferError::InvalidDimensions)?;
            let start = isize::try_from(top.checked_add(1).ok_or(BufferError::SizeOverflow)?)
                .map_err(|_| BufferError::SizeOverflow)?;
            lines.shift_elements(start, height - 1, -1)?;
            lines.set(bottom, Some(clone_blank(&new_line)?))?;
        }
        let display = {
            let mut buffer = active
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            if full_screen {
                if !trim {
                    buffer.ybase = buffer
                        .ybase
                        .checked_add(1)
                        .ok_or(BufferError::SizeOverflow)?;
                    if !self.is_user_scrolling.get() {
                        buffer.ydisp = buffer
                            .ydisp
                            .checked_add(1)
                            .ok_or(BufferError::SizeOverflow)?;
                    }
                } else if self.is_user_scrolling.get() {
                    buffer.ydisp = buffer.ydisp.saturating_sub(1);
                }
            }
            if !self.is_user_scrolling.get() {
                buffer.ydisp = buffer.ybase;
            }
            buffer.ydisp
        };
        self.scroll_event.fire(&display)?;
        Ok(())
    }

    /// Displacement is an integer line count. Browser API number conversion must
    /// happen before this coordinate-based core call, without lossy casts.
    pub fn scroll_lines(
        &self,
        displacement: i64,
        suppress_scroll_event: bool,
    ) -> Result<(), BufferError> {
        let active = self.buffer();
        let display = {
            let mut buffer = active
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            if displacement < 0 {
                if buffer.ydisp == 0 {
                    return Ok(());
                }
                self.is_user_scrolling.set(true);
            } else if i128::from(displacement) + buffer.ydisp as i128 >= buffer.ybase as i128 {
                self.is_user_scrolling.set(false);
            }
            let previous = buffer.ydisp;
            let next =
                (buffer.ydisp as i128 + i128::from(displacement)).clamp(0, buffer.ybase as i128);
            buffer.ydisp = usize::try_from(next).map_err(|_| BufferError::SizeOverflow)?;
            if previous == buffer.ydisp {
                return Ok(());
            }
            buffer.ydisp
        };
        if !suppress_scroll_event {
            self.scroll_event.fire(&display)?;
        }
        Ok(())
    }
    pub fn dispose(&self) -> Result<(), BufferError> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        // Source registers these resources in this order.
        self.resize_event.dispose();
        self.scroll_event.dispose();
        self.buffers.dispose()?;
        self.activation_listener.dispose();
        Ok(())
    }
}
impl Drop for BufferService {
    fn drop(&mut self) {
        self.activation_listener.dispose();
    }
}
fn clone_blank(line: &LineRef) -> Result<LineRef, BufferError> {
    Ok(Rc::new(RefCell::new(
        line.try_borrow()
            .map_err(|_| BufferError::BorrowedLine)?
            .clone_line(true),
    )))
}
fn dimension(
    options: &OptionsService,
    name: &'static str,
    minimum: usize,
) -> Result<usize, BufferError> {
    let raw = options.get(name)?;
    let value = if raw.is_truthy() { raw.number()? } else { 0.0 };
    if value.is_nan() {
        return Err(BufferError::InvalidOption(name));
    }
    let value = value.max(minimum as f64);
    if !value.is_finite() || value.fract() != 0.0 || value > u32::MAX as f64 {
        return Err(BufferError::InvalidOption(name));
    }
    // The bounds above establish a nonnegative integer representable on both
    // supported 32-bit WASM and native usize targets.
    Ok(value as usize)
}
