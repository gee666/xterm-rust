// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Callback-safe DCS registration and disposal ownership.

use super::{DcsAction, DcsHandlerTrait, HandlerHandle};
use crate::lifecycle::DisposableHandle;
use crate::parser::handler::{
    registry::{ActiveSlot, CallbackSlot, HandlerRegistry, Registrar},
    DispatchError,
};
use std::rc::{Rc, Weak};

type Fallback = dyn FnMut(i32, DcsAction) -> Result<(), DispatchError>;

#[derive(Default)]
pub(super) struct Control {
    registry: HandlerRegistry<i32, dyn DcsHandlerTrait>,
    active: ActiveSlot<dyn DcsHandlerTrait>,
    fallback: CallbackSlot<Fallback>,
}
impl Control {
    pub(super) fn register(&self, ident: i32, handler: HandlerHandle) -> DisposableHandle {
        self.registry.register(ident, handler)
    }
    pub(super) fn registrar(&self) -> Registrar<i32, dyn DcsHandlerTrait> {
        self.registry.registrar()
    }
    pub(super) fn clear_handler(&self, ident: i32) {
        self.registry.clear(&ident);
    }
    pub(super) fn select(&self, ident: i32) {
        self.active.set(self.registry.get(&ident));
    }
    pub(super) fn clear_active(&self) {
        self.active.clear();
    }
    pub(super) fn active_len(&self) -> usize {
        self.active.len()
    }
    pub(super) fn at(&self, index: usize) -> Result<HandlerHandle, DispatchError> {
        self.active.at(index)
    }
    pub(super) fn set_fallback(&self, fallback: Box<Fallback>) {
        self.fallback.set(fallback);
    }
    pub(super) fn report(&self, ident: i32, action: DcsAction) -> Result<(), DispatchError> {
        if let Some(fallback) = self.fallback.get() {
            fallback.borrow_mut()(ident, action)?;
        }
        Ok(())
    }
    pub(super) fn dispose(&self) {
        self.registry.clear_all();
        self.fallback.clear();
        self.active.clear();
    }
}

/// A weak callback-safe equivalent of DcsParser.dispose. It clears the same
/// live registry, fallback and active-list property read by the current frame.
/// Identifier and paused metadata are not reset, as in the source method.
#[derive(Clone)]
pub struct DcsDisposalHandle(Weak<Control>);
impl DcsDisposalHandle {
    pub(super) fn new(control: &Rc<Control>) -> Self {
        Self(Rc::downgrade(control))
    }

    /// None means the owning parser was dropped. Every call on a live parser
    /// clears its current registrations; a parent terminal owns its once guard.
    pub fn dispose(&self) -> Option<()> {
        self.0.upgrade()?.dispose();
        Some(())
    }
}
