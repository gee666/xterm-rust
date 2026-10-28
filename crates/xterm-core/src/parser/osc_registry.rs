// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. OSC numeric identifiers over shared registration ownership.
use super::{key, HandlerHandle, OscHandlerTrait};
use crate::lifecycle::DisposableHandle;
use crate::parser::handler::registry::{HandlerRegistry, Registrar};

pub(super) type Registry = HandlerRegistry<u64, dyn OscHandlerTrait>;

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
