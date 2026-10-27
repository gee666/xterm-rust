// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of common/parser/ApcParser.ts.

//! Application Program Commands dispatch by an identifier supplied by the outer
//! escape parser. Their streaming callbacks, payload limit and asynchronous
//! cleanup order are identical to OSC payload dispatch, so both use one engine.
//! Payload strings are UTF-16 units to preserve JavaScript lone surrogates.

use super::handler::{DispatchError, HandlerResult};
use super::osc::OscParser;
use crate::lifecycle::DisposableHandle;

pub use super::osc::{
    HandlerHandle, OscAction as ApcAction, OscHandler as ApcHandler,
    OscHandlerTrait as ApcHandlerTrait, PAYLOAD_LIMIT,
};

/// APC registration, fallback, streaming and resumable handler dispatch.
///
/// A registration stays active when its token is dropped. Explicitly dispose the
/// token to unregister, as with JavaScript's IDisposable. Handler identities and
/// mutable lists retain the upstream behavior when registrations change during
/// an active command. Disposal clears registrations without calling `end`.
#[derive(Default)]
pub struct ApcParser {
    dispatch: OscParser,
}

impl ApcParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Identifiers are the packed integer supplied by the escape parser.
    pub fn register_handler(&mut self, ident: u32, handler: HandlerHandle) -> DisposableHandle {
        self.dispatch.register_handler(f64::from(ident), handler)
    }

    pub fn clear_handler(&mut self, ident: u32) {
        self.dispatch.clear_handler(f64::from(ident));
    }

    pub fn set_handler_fallback(&mut self, mut callback: impl FnMut(u32, ApcAction) + 'static) {
        self.dispatch.set_handler_fallback(move |ident, action| {
            // Only this adapter supplies IDs, and every u32 is exactly represented
            // in f64. Shared idle dispatch always supplies zero.
            callback(ident as u32, action);
        });
    }

    pub fn dispose(&mut self) {
        self.dispatch.dispose();
    }

    /// Abort unfinished handlers in reverse order. A previously returned future
    /// remains owned by its caller; it is not awaited by reset.
    pub fn reset(&mut self) -> Result<(), DispatchError> {
        self.dispatch.reset()
    }

    /// Take asynchronous abort cleanup initiated by reset or a new start.
    /// The caller must poll these futures, matching the work queued by JavaScript
    /// promises. The core does not select an executor or spawn detached tasks.
    pub fn take_cleanup(&mut self) -> Vec<HandlerResult> {
        self.dispatch.take_cleanup()
    }

    /// Starting another command first aborts the previous command.
    pub fn start(&mut self, ident: u32) -> Result<(), DispatchError> {
        self.dispatch.start_identified(f64::from(ident))
    }

    /// A borrowed slice replaces the upstream data/start/end triple.
    pub fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.dispatch.put_identified(data)
    }

    /// Return a pending callback future, or `None` after finishing all handlers.
    /// Await the future and call again with the same success flag and its result.
    /// Use `true` as the initial previous result, like upstream's default.
    pub fn end(
        &mut self,
        success: bool,
        previous_result: bool,
    ) -> Result<Option<HandlerResult>, DispatchError> {
        self.dispatch.end_identified(success, previous_result)
    }
}
