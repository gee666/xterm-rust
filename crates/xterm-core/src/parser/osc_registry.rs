// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. OSC numeric identifiers over shared registration ownership.
use super::{key, HandlerHandle, OscAction, OscError, OscHandlerTrait};
use crate::lifecycle::DisposableHandle;
use crate::parser::handler::registry::{ActiveSlot, CallbackSlot, HandlerRegistry, Registrar};
use std::rc::{Rc, Weak};

pub(super) type Registry = HandlerRegistry<u64, dyn OscHandlerTrait>;
type Fallback = dyn FnMut(f64, OscAction) -> Result<(), OscError>;

/// Only typed protocol wiring lives here. Shared slots implement all aliasing,
/// borrow release and callback-lifetime behavior for both OSC and DCS.
#[derive(Default)]
pub(super) struct Control {
    pub(super) registry: Registry,
    pub(super) active: ActiveSlot<dyn OscHandlerTrait>,
    pub(super) fallback: CallbackSlot<Fallback>,
}
impl Control {
    pub(super) fn dispose(&self) {
        self.registry.clear_all();
        self.fallback.clear();
        self.active.clear();
    }
}

/// Callback-safe source disposal without borrowing/resetting the parser. Each
/// call clears current state properties; a parent terminal owns its once guard.
#[derive(Clone)]
pub struct OscDisposalHandle(Weak<Control>);
impl OscDisposalHandle {
    pub(super) fn new(control: &Rc<Control>) -> Self {
        Self(Rc::downgrade(control))
    }
    pub fn dispose(&self) -> Option<()> {
        self.0.upgrade()?.dispose();
        Some(())
    }
}

/// Callback-safe registration access. Weak ownership prevents a registered
/// callback from keeping its parser or itself alive through this handle.
#[derive(Clone)]
pub struct OscRegistrar {
    registry: Registrar<u64, dyn OscHandlerTrait>,
}
impl OscRegistrar {
    pub(super) fn new(registry: &Registry) -> Self {
        Self {
            registry: registry.registrar(),
        }
    }

    /// None means the parser was dropped, not merely explicitly disposed.
    pub fn register_handler(&self, id: f64, handler: HandlerHandle) -> Option<DisposableHandle> {
        self.registry.register(key(id), handler)
    }

    /// Missing identifiers succeed while the owning parser remains alive.
    pub fn clear_handler(&self, id: f64) -> Option<()> {
        self.registry.clear(&key(id))
    }
}
