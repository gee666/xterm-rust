// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent};
use crate::{
    buffer::BufferService,
    buffer::{AttributeData, Buffer, BufferLine},
    charset::CharsetService,
    core_service::CoreService,
    options::OptionsService,
    unicode_service::UnicodeService,
};
use std::{cell::RefCell, rc::Rc};

/// Shared state read by native and browser adapters. Buffer rows retain identity.
pub struct TerminalState {
    pub buffer_service: Rc<BufferService>,
    pub links: Rc<crate::osc_link_service::OscLinkService>,
    pub options: Rc<OptionsService>,
    pub core: Rc<CoreService>,
    pub mouse: Rc<crate::mouse_state::MouseStateService>,
    pub logger: Rc<crate::log_service::LogService>,
    pub unicode: Rc<UnicodeService>,
    pub charset: CharsetService,
    pub attributes: AttributeData,
    pub(crate) attributes_identity: Rc<RefCell<Option<AttributeData>>>,
    pub title: Vec<u16>,
    pub icon_name: Vec<u16>,
    pub(crate) title_stack: Vec<Vec<u16>>,
    pub(crate) icon_stack: Vec<Vec<u16>>,
    pub(crate) events: Rc<RefCell<Vec<TerminalEvent>>>,
    pub(crate) event_handler: Rc<RefCell<Option<super::EventHandler>>>,
    pub(crate) error: Option<TerminalError>,
    pub(crate) dirty: Option<(usize, usize)>,
    pub(crate) preceding_join_state: Rc<std::cell::Cell<u32>>,
    pub(crate) windows_wrapping: Rc<std::cell::Cell<bool>>,
    pub(crate) pending_replies: std::collections::VecDeque<Vec<u16>>,
}
impl std::ops::Deref for TerminalState {
    type Target = BufferService;
    fn deref(&self) -> &BufferService {
        &self.buffer_service
    }
}
impl TerminalState {
    pub(crate) fn replace_attributes(&mut self, attributes: AttributeData) {
        let previous = std::mem::replace(&mut self.attributes, attributes);
        *self.attributes_identity.borrow_mut() = Some(previous);
        self.attributes_identity = Rc::new(RefCell::new(None));
    }
    pub fn take_error(&mut self) -> Option<TerminalError> {
        self.error.take()
    }
    pub fn active_buffer(&self) -> Rc<RefCell<Buffer>> {
        self.buffers.active()
    }
    pub(crate) fn line(&self, y: usize) -> Result<Rc<RefCell<BufferLine>>, TerminalError> {
        let buffer = self.active_buffer();
        let buffer = buffer.borrow();
        if y >= self.rows() {
            return Err(TerminalError::MissingLine);
        }
        buffer
            .lines
            .get(buffer.ybase + y)
            .ok_or(TerminalError::MissingLine)
    }
    pub(crate) fn dirty_row(&mut self, y: usize) {
        self.dirty = Some(match self.dirty {
            None => (y, y),
            Some((start, end)) => (start.min(y), end.max(y)),
        });
    }
    pub(crate) fn dirty_all(&mut self) {
        self.dirty = Some((0, self.rows() - 1));
    }
    pub(crate) fn erase_attributes(&self) -> AttributeData {
        // BCE retains background color, not styles, protection or hyperlinks.
        AttributeData {
            bg: self.attributes.bg
                & (crate::buffer::constants::attributes::CM_MASK
                    | crate::buffer::constants::attributes::RGB_MASK),
            ..AttributeData::default()
        }
    }
    pub(crate) fn option_bool(&self, key: &str) -> Result<bool, TerminalError> {
        Ok(self.options.get(key)?.is_truthy())
    }
    pub(crate) fn reply(&mut self, text: &str) -> Result<(), TerminalError> {
        self.pending_replies
            .push_back(text.encode_utf16().collect());
        Ok(())
    }
}
