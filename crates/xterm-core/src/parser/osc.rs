// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/parser/OscParser.ts.

//! OSC streaming dispatch. Identifiers use `f64` to retain JavaScript rounding
//! and overflow. Payload slices are UTF-32; fallback strings are UTF-16 units.
//! Pending end results must be awaited, then passed back to `end`. Reset may
//! cancel the remaining dispatch without awaiting a previously returned future.

use crate::lifecycle::{to_disposable, DisposableHandle};
use crate::text_decoder::utf32_to_string;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[path = "osc_handler.rs"]
mod handler;
pub use handler::OscHandler;

pub const PAYLOAD_LIMIT: usize = 10_000_000;

pub use super::handler::{DispatchError as OscError, HandlerResult};

pub trait OscHandlerTrait {
    fn start(&mut self);
    fn put(&mut self, data: &[u32]);
    fn end(&mut self, success: bool) -> HandlerResult;
}

pub type HandlerHandle = Rc<RefCell<dyn OscHandlerTrait>>;
type HandlerList = Rc<RefCell<Vec<HandlerHandle>>>;

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
    handlers: HashMap<u64, HandlerList>,
    active: HandlerList,
    fallback: Option<Box<dyn FnMut(f64, OscAction)>>,
    pause: Option<Pause>,
    cleanup: Vec<HandlerResult>,
}

impl Default for OscParser {
    fn default() -> Self {
        Self {
            state: State::Start,
            id: -1.0,
            handlers: HashMap::new(),
            active: Rc::default(),
            fallback: None,
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
        let list = self.handlers.entry(key(id)).or_default();
        list.borrow_mut().push(Rc::clone(&handler));
        let list = Rc::clone(list);
        to_disposable(move || {
            let mut list = list.borrow_mut();
            if let Some(index) = list.iter().position(|entry| Rc::ptr_eq(entry, &handler)) {
                list.remove(index);
            }
        })
    }

    pub fn clear_handler(&mut self, id: f64) {
        self.handlers.remove(&key(id));
    }

    pub fn set_handler_fallback(&mut self, callback: impl FnMut(f64, OscAction) + 'static) {
        self.fallback = Some(Box::new(callback));
    }

    pub fn dispose(&mut self) {
        self.handlers.clear();
        self.fallback = None;
        self.active = Rc::default();
    }

    fn report(&mut self, action: OscAction) {
        if let Some(fallback) = &mut self.fallback {
            fallback(self.id, action);
        }
    }

    // Clone one identity at a time, never borrowing the collection in callbacks.
    fn at(&self, index: usize) -> Result<HandlerHandle, OscError> {
        self.active
            .borrow()
            .get(index)
            .cloned()
            .ok_or(OscError::ActiveHandlerRemoved)
    }

    pub fn reset(&mut self) -> Result<(), OscError> {
        if self.state == State::Payload {
            let end = self
                .pause
                .as_ref()
                .map_or_else(|| self.active.borrow().len(), |pause| pause.next);
            for index in (0..end).rev() {
                let handler = self.at(index)?;
                // Rust futures are lazy. Retain asynchronous cleanup for the
                // adapter's executor instead of silently cancelling it.
                let result = handler.borrow_mut().end(false);
                if matches!(result, HandlerResult::Pending(_)) {
                    self.cleanup.push(result);
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
        self.active = Rc::default();
        self.id = -1.0;
        self.state = State::Start;
    }

    fn begin_payload(&mut self) -> Result<(), OscError> {
        self.active = self
            .handlers
            .get(&key(self.id))
            .cloned()
            .unwrap_or_default();
        let count = self.active.borrow().len();
        if count == 0 {
            self.report(OscAction::Start);
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().start();
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
        let count = self.active.borrow().len();
        if count == 0 {
            self.report(OscAction::Put(utf32_to_string(data)));
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().put(data);
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
        let count = self.active.borrow().len();
        if count == 0 {
            self.report(OscAction::End(success));
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
                HandlerResult::Pending(future) => {
                    self.pause = Some(Pause { next, cleanup });
                    return Ok(Some(HandlerResult::Pending(future)));
                }
            }
        }
        self.finish();
        Ok(None)
    }
}
