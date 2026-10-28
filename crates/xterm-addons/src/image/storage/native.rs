// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{ImageBufferType, StorageDiagnostic, StorageError, StorageTerminal};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use xterm_core::{
    buffer::{marker::Marker, BufferService},
    headless::Terminal,
    terminal::{line_feed_state, mark_dirty_state, TerminalState},
};

pub type MarkerProvider = Rc<dyn Fn(i64) -> Result<Option<Marker>, StorageError>>;
pub type StorageLogger = Rc<dyn Fn(StorageDiagnostic, &str) -> Result<(), StorageError>>;

/// Adapter over the actual engine state. Weak terminal/state handles avoid a
/// cycle from terminal event listeners through the addon back into the terminal.
/// Diagnostics and marker registration retain the caller's platform policy.
pub struct TerminalStorageHost {
    state: Weak<RefCell<TerminalState>>,
    service: Rc<BufferService>,
    markers: MarkerProvider,
    logger: StorageLogger,
}
impl TerminalStorageHost {
    pub fn new(
        state: &Rc<RefCell<TerminalState>>,
        markers: MarkerProvider,
        logger: StorageLogger,
    ) -> Result<Self, StorageError> {
        let service = Rc::clone(
            &state
                .try_borrow()
                .map_err(|_| StorageError::BorrowedBuffer)?
                .buffer_service,
        );
        Ok(Self {
            state: Rc::downgrade(state),
            service,
            markers,
            logger,
        })
    }
    pub fn from_terminal(
        terminal: &Rc<Terminal>,
        logger: StorageLogger,
    ) -> Result<Self, StorageError> {
        let state = terminal.state_handle_for_addon();
        let weak = Rc::downgrade(terminal);
        Self::new(
            &state,
            Rc::new(move |offset| {
                let terminal = weak.upgrade().ok_or(StorageError::DisconnectedTerminal)?;
                terminal
                    .register_marker(offset)
                    .map(Some)
                    .map_err(StorageError::host)
            }),
            logger,
        )
    }
    fn state(&self) -> Result<Rc<RefCell<TerminalState>>, StorageError> {
        self.state
            .upgrade()
            .ok_or(StorageError::DisconnectedTerminal)
    }
}
impl StorageTerminal for TerminalStorageHost {
    fn buffer_service(&self) -> Rc<BufferService> {
        Rc::clone(&self.service)
    }
    fn active_buffer_type(&self) -> ImageBufferType {
        if Rc::ptr_eq(&self.service.buffer(), &self.service.buffers.normal()) {
            ImageBufferType::Normal
        } else {
            ImageBufferType::Alternate
        }
    }
    fn mark_dirty(&self, row: usize) -> Result<(), StorageError> {
        mark_dirty_state(&self.state()?, row).map_err(StorageError::host)
    }
    fn line_feed(&self) -> Result<(), StorageError> {
        line_feed_state(&self.state()?).map_err(StorageError::host)
    }
    fn register_marker(&self, offset: i64) -> Result<Option<Marker>, StorageError> {
        (self.markers)(offset)
    }
    fn diagnostic(&self, level: StorageDiagnostic, message: &str) -> Result<(), StorageError> {
        (self.logger)(level, message)
    }
}
