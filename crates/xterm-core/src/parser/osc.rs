// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/parser/OscParser.ts.

//! OSC streaming dispatch. Identifiers use `f64` to retain JavaScript rounding
//! and overflow. Payload slices are UTF-32; fallback strings are UTF-16 units.
//! Pending end results must be awaited, then passed back to `end`. Reset may
//! cancel the remaining dispatch without awaiting a previously returned future.

use crate::lifecycle::DisposableHandle;
use crate::text_decoder::utf32_to_string;
use std::cell::RefCell;
use std::rc::Rc;

#[path = "osc_handler.rs"]
mod handler;
pub use handler::OscHandler;
#[path = "osc_registry.rs"]
mod registry;
pub use registry::{OscDisposalHandle, OscRegistrar};

pub const PAYLOAD_LIMIT: usize = 10_000_000;

pub use super::handler::{DispatchError as OscError, HandlerResult};

pub trait OscHandlerTrait {
    fn start(&mut self) -> Result<(), OscError>;
    fn put(&mut self, data: &[u32]) -> Result<(), OscError>;
    fn end(&mut self, success: bool) -> HandlerResult;
}

pub type HandlerHandle = Rc<RefCell<dyn OscHandlerTrait>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OscAction {
    Start,
    Put(Vec<u16>),
    End(bool),
}

#[derive(Default, PartialEq, Eq)]
enum State {
    #[default]
    Start,
    Id,
    Payload,
    Abort,
}

struct Pause {
    next: usize,
    cleanup: bool,
}

pub struct OscParser {
    state: State,
    id: f64,
    control: Rc<registry::Control>,
    pause: Option<Pause>,
    cleanup: Vec<HandlerResult>,
}

impl Default for OscParser {
    fn default() -> Self {
        Self {
            state: State::Start,
            id: -1.0,
            control: Rc::default(),
            pause: None,
            cleanup: Vec::new(),
        }
    }
}

fn key(id: f64) -> u64 {
    if id == 0.0 {
        0
    } else if id.is_nan() {
        f64::NAN.to_bits()
    } else {
        id.to_bits()
    }
}

impl OscParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Explicit disposal removes the first registration with the same identity.
    /// Dropping the returned handle does not unregister, matching JavaScript.
    pub fn register_handler(&mut self, id: f64, handler: HandlerHandle) -> DisposableHandle {
        self.control.registry.register(key(id), handler)
    }

    pub fn clear_handler(&mut self, id: f64) {
        self.control.registry.clear(&key(id));
    }

    pub fn registrar(&self) -> OscRegistrar {
        OscRegistrar::new(&self.control.registry)
    }

    pub fn set_handler_fallback(&mut self, mut callback: impl FnMut(f64, OscAction) + 'static) {
        self.set_handler_fallback_checked(move |id, action| {
            callback(id, action);
            Ok(())
        });
    }

    /// Callback failures correspond to source exceptions. Dispatch stops without
    /// clearing the identifier, payload state or queued cleanup.
    pub fn set_handler_fallback_checked(
        &mut self,
        callback: impl FnMut(f64, OscAction) -> Result<(), OscError> + 'static,
    ) {
        self.control.fallback.set(Box::new(callback));
    }

    pub fn disposal_handle(&self) -> OscDisposalHandle {
        OscDisposalHandle::new(&self.control)
    }

    pub fn dispose(&mut self) {
        self.control.dispose();
    }

    fn report(&mut self, action: OscAction) -> Result<(), OscError> {
        // The slot borrow ends before user code. Disposal may clear this slot
        // during the call; the current callback is never restored afterward.
        if let Some(fallback) = self.control.fallback.get() {
            fallback.borrow_mut()(self.id, action)?;
        }
        Ok(())
    }

    // Clone one identity at a time, never borrowing the collection in callbacks.
    fn at(&self, index: usize) -> Result<HandlerHandle, OscError> {
        self.control.active.at(index)
    }

    pub fn reset(&mut self) -> Result<(), OscError> {
        if self.state == State::Payload {
            let end = self
                .pause
                .as_ref()
                .map_or_else(|| self.control.active.len(), |pause| pause.next);
            for index in (0..end).rev() {
                let handler = self.at(index)?;
                // Rust futures are lazy. Retain asynchronous cleanup for the
                // adapter's executor instead of silently cancelling it.
                let result = handler.borrow_mut().end(false);
                match result {
                    HandlerResult::Pending(_) | HandlerResult::PendingFallible(_) => {
                        self.cleanup.push(result)
                    }
                    HandlerResult::Failed(error) => return Err(error),
                    HandlerResult::Ready(_) => {}
                }
            }
        }
        self.finish();
        Ok(())
    }

    /// Transfer reset cleanup futures to the adapter's executor. Drive every
    /// returned future even though its boolean result no longer affects dispatch.
    /// Unlike JS promises, Rust futures do not execute merely by being created.
    pub fn take_cleanup(&mut self) -> Vec<HandlerResult> {
        std::mem::take(&mut self.cleanup)
    }

    fn finish(&mut self) {
        self.pause = None;
        self.control.active.clear();
        self.id = -1.0;
        self.state = State::Start;
    }

    fn begin_payload(&mut self) -> Result<(), OscError> {
        self.control
            .active
            .set(self.control.registry.get(&key(self.id)));
        let count = self.control.active.len();
        if count == 0 {
            self.report(OscAction::Start)?;
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().start()?;
            }
        }
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), OscError> {
        self.reset()?;
        self.state = State::Id;
        Ok(())
    }

    /// Shared APC dispatch entry. APC identifiers are supplied by its caller.
    pub(crate) fn start_identified(&mut self, id: f64) -> Result<(), OscError> {
        self.reset()?;
        self.id = id;
        self.state = State::Payload;
        self.begin_payload()
    }

    /// APC forwards even empty chunks, including fallback calls while idle.
    pub(crate) fn put_identified(&mut self, data: &[u32]) -> Result<(), OscError> {
        if self.state == State::Start {
            self.id = 0.0;
        }
        self.dispatch_payload(data)
    }

    pub(crate) fn end_identified(
        &mut self,
        success: bool,
        previous_result: bool,
    ) -> Result<Option<HandlerResult>, OscError> {
        if self.state == State::Start {
            self.id = 0.0;
            self.state = State::Payload;
        }
        self.end(success, previous_result)
    }

    /// Slice the caller's data to represent the upstream `start..end` range.
    pub fn put(&mut self, data: &[u32]) -> Result<(), OscError> {
        if self.state == State::Abort {
            return Ok(());
        }
        let mut index = 0;
        if self.state == State::Id {
            while let Some(&code) = data.get(index) {
                index += 1;
                if code == u32::from(b';') {
                    self.state = State::Payload;
                    self.begin_payload()?;
                    break;
                }
                if !(u32::from(b'0')..=u32::from(b'9')).contains(&code) {
                    self.state = State::Abort;
                    return Ok(());
                }
                if self.id == -1.0 {
                    self.id = 0.0;
                }
                // Keep the upstream operation order, including floating rounding.
                self.id = self.id * 10.0 + f64::from(code) - 48.0;
            }
        }
        if self.state == State::Payload && index < data.len() {
            self.dispatch_payload(&data[index..])?;
        }
        Ok(())
    }

    fn dispatch_payload(&mut self, data: &[u32]) -> Result<(), OscError> {
        let count = self.control.active.len();
        if count == 0 {
            self.report(OscAction::Put(utf32_to_string(data)))?;
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().put(data)?;
            }
        }
        Ok(())
    }

    /// Return a pending handler future, or `None` once dispatch is complete.
    /// On resume use the same success flag and the resolved previous result.
    pub fn end(
        &mut self,
        success: bool,
        previous_result: bool,
    ) -> Result<Option<HandlerResult>, OscError> {
        if self.state == State::Start {
            return Ok(None);
        }
        if self.state == State::Abort {
            self.finish();
            return Ok(None);
        }
        if self.state == State::Id {
            self.begin_payload()?;
        }
        let count = self.control.active.len();
        if count == 0 {
            self.report(OscAction::End(success))?;
            self.finish();
            return Ok(None);
        }
        let (mut next, mut cleanup) = if let Some(pause) = self.pause.take() {
            (pause.next, pause.cleanup || previous_result)
        } else {
            (count, false)
        };
        while next > 0 {
            next -= 1;
            let handler = self.at(next)?;
            let result = handler.borrow_mut().end(success && !cleanup);
            match result {
                HandlerResult::Ready(handled) => cleanup |= handled,
                HandlerResult::Failed(error) => return Err(error),
                pending @ (HandlerResult::Pending(_) | HandlerResult::PendingFallible(_)) => {
                    self.pause = Some(Pause { next, cleanup });
                    return Ok(Some(pending));
                }
            }
        }
        self.finish();
        Ok(None)
    }
}
