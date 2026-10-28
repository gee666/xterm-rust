// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Live public buffer views. Line/cell operations use the core BufferLine and
//! CellData directly. Native callers release RefCell borrows before mutating the
//! terminal; retained line handles have the same stale-reference caveat as JS.

use crate::{
    buffer::{
        buffer_set::BufferRef, Buffer, BufferError, BufferLine, BufferSet, CellData, LineRef,
    },
    event::{EmitError, Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
};
use std::{cell::RefCell, fmt, rc::Rc};

/// A signed public cell lookup, used by native and JavaScript adapters.
/// Unlike circular buffer line lookup, BufferLine.getCell checks its bounds.
pub fn get_cell_signed(line: &BufferLine, index: i64) -> Option<CellData> {
    usize::try_from(index)
        .ok()
        .and_then(|index| line.get_cell(index))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferType {
    Normal,
    Alternate,
}

pub struct BufferApiView {
    buffer: RefCell<BufferRef>,
    buffer_type: BufferType,
}
impl BufferApiView {
    pub fn new(buffer: BufferRef, buffer_type: BufferType) -> Self {
        Self {
            buffer: RefCell::new(buffer),
            buffer_type,
        }
    }
    pub fn init(&self, buffer: BufferRef) -> &Self {
        // Releasing the old buffer can run payload Drop code that reenters this
        // view. Publish the new backing and end its borrow before that cleanup.
        let previous = self.buffer.replace(buffer);
        drop(previous);
        self
    }
    pub fn buffer_type(&self) -> BufferType {
        self.buffer_type
    }
    fn read<R>(&self, action: impl FnOnce(&Buffer) -> R) -> Result<R, BufferError> {
        let buffer = self.buffer.borrow();
        let buffer = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
        Ok(action(&buffer))
    }
    pub fn cursor_y(&self) -> Result<usize, BufferError> {
        self.read(|buffer| buffer.y)
    }
    pub fn cursor_x(&self) -> Result<usize, BufferError> {
        self.read(|buffer| buffer.x)
    }
    pub fn viewport_y(&self) -> Result<usize, BufferError> {
        self.read(|buffer| buffer.ydisp)
    }
    pub fn base_y(&self) -> Result<usize, BufferError> {
        self.read(|buffer| buffer.ybase)
    }
    pub fn len(&self) -> Result<usize, BufferError> {
        self.read(|buffer| buffer.lines.len())
    }
    pub fn is_empty(&self) -> Result<bool, BufferError> {
        self.read(|buffer| buffer.lines.is_empty())
    }
    /// Native indices are unsigned integral values. Browser bindings must validate
    /// JavaScript number indexing separately, rather than truncating into usize.
    pub fn get_line(&self, index: usize) -> Result<Option<LineRef>, BufferError> {
        // The source directly delegates to CircularList.get, including wrapped
        // indices and stale slots outside logical length. Do not add bounds checks.
        self.read(|buffer| buffer.lines.get(index))
    }
    /// Signed native indexing preserves the source CircularList's signed
    /// remainder and negative-property behavior without a usize conversion.
    pub fn get_line_signed(&self, index: i64) -> Result<Option<LineRef>, BufferError> {
        self.read(|buffer| buffer.lines.get_signed(index))
    }
    /// JavaScript number indexing delegates to the source ring arithmetic.
    pub fn get_line_number(&self, index: f64) -> Result<Option<LineRef>, BufferError> {
        self.read(|buffer| buffer.lines.get_number(index))
    }
    /// This is a zero-content, width-zero CellData, not Buffer's width-one blank.
    pub fn get_null_cell(&self) -> CellData {
        CellData::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidActiveBuffer;
impl fmt::Display for InvalidActiveBuffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Active buffer is neither normal nor alternate")
    }
}
impl std::error::Error for InvalidActiveBuffer {}

struct NamespaceState {
    core: Rc<BufferSet>,
    normal: Rc<BufferApiView>,
    alternate: Rc<BufferApiView>,
    changed: Emitter<Rc<BufferApiView>>,
}
impl NamespaceState {
    fn normal(&self) -> Rc<BufferApiView> {
        self.normal.init(self.core.normal());
        Rc::clone(&self.normal)
    }
    fn alternate(&self) -> Rc<BufferApiView> {
        self.alternate.init(self.core.alt());
        Rc::clone(&self.alternate)
    }
    fn active(&self) -> Result<Rc<BufferApiView>, InvalidActiveBuffer> {
        let active = self.core.active();
        if Rc::ptr_eq(&active, &self.core.normal()) {
            return Ok(self.normal());
        }
        if Rc::ptr_eq(&active, &self.core.alt()) {
            return Ok(self.alternate());
        }
        Err(InvalidActiveBuffer)
    }
}

pub struct BufferNamespaceApi {
    state: Rc<NamespaceState>,
    subscription: DisposableHandle,
}
impl BufferNamespaceApi {
    pub fn new(core: Rc<BufferSet>) -> Self {
        let state = Rc::new(NamespaceState {
            normal: Rc::new(BufferApiView::new(core.normal(), BufferType::Normal)),
            alternate: Rc::new(BufferApiView::new(core.alt(), BufferType::Alternate)),
            core,
            changed: Emitter::new(),
        });
        let weak = Rc::downgrade(&state);
        let subscription = state.core.on_buffer_activate().try_subscribe(move |_| {
            if let Some(state) = weak.upgrade() {
                let active = state.active().map_err(EmitError::callback)?;
                state.changed.fire(&active)?;
            }
            Ok(())
        });
        Self {
            state,
            subscription,
        }
    }
    pub fn active(&self) -> Result<Rc<BufferApiView>, InvalidActiveBuffer> {
        self.state.active()
    }
    pub fn normal(&self) -> Rc<BufferApiView> {
        self.state.normal()
    }
    pub fn alternate(&self) -> Rc<BufferApiView> {
        self.state.alternate()
    }
    pub fn on_buffer_change(&self) -> Event<Rc<BufferApiView>> {
        self.state.changed.event()
    }
}
impl Disposable for BufferNamespaceApi {
    fn dispose(&self) {
        self.subscription.dispose();
        self.state.changed.dispose();
    }
}
impl Drop for BufferNamespaceApi {
    fn drop(&mut self) {
        self.dispose();
    }
}
