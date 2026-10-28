// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/parser/DcsParser.ts.

//! Streaming DCS dispatch. Handlers run newest first and remain active until
//! unhook or reset. Await a returned future and pass its result to `unhook`
//! with the same success flag. Identifiers retain the source's packed i32 bits.

use super::handler::registry::{HandlerRegistry, Registrar};
use super::handler::{DispatchError, HandlerResult};
use super::params::Params;
use crate::lifecycle::DisposableHandle;
use crate::text_decoder::utf32_to_string;
use std::cell::RefCell;
use std::rc::Rc;

#[path = "dcs_handler.rs"]
mod callback;
pub use callback::DcsHandler;

pub trait DcsHandlerTrait {
    fn hook(&mut self, params: &Params);
    fn put(&mut self, data: &[u32]);
    fn unhook(&mut self, success: bool) -> HandlerResult;
}

pub type HandlerHandle = Rc<RefCell<dyn DcsHandlerTrait>>;
type HandlerList = Rc<RefCell<Vec<HandlerHandle>>>;

/// Reentrant registration without borrowing the active parser. The weak handle
/// returns None after parser drop and cannot create a callback ownership cycle.
#[derive(Clone)]
pub struct DcsRegistrar(Registrar<i32, dyn DcsHandlerTrait>);
impl DcsRegistrar {
    pub fn register_handler(&self, ident: i32, handler: HandlerHandle) -> Option<DisposableHandle> {
        self.0.register(ident, handler)
    }
    pub fn clear_handler(&self, ident: i32) -> Option<()> {
        self.0.clear(&ident)
    }
}

/// Typed replacement for the fallback's action string and untyped argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DcsAction {
    Hook(Params),
    Put(Vec<u16>),
    Unhook(bool),
}

struct Pause {
    next: usize,
    cleanup: bool,
}

#[derive(Default)]
pub struct DcsParser {
    handlers: HandlerRegistry<i32, dyn DcsHandlerTrait>,
    active: HandlerList,
    ident: i32,
    fallback: Option<Box<dyn FnMut(i32, DcsAction)>>,
    pause: Option<Pause>,
    cleanup: Vec<HandlerResult>,
}

impl DcsParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Disposal removes the first matching identity, including duplicates.
    /// Dropping the returned handle does not unregister the handler.
    pub fn register_handler(&mut self, ident: i32, handler: HandlerHandle) -> DisposableHandle {
        self.handlers.register(ident, handler)
    }

    pub fn registrar(&self) -> DcsRegistrar {
        DcsRegistrar(self.handlers.registrar())
    }

    /// Remove future lookup without detaching an already active handler list.
    pub fn clear_handler(&mut self, ident: i32) {
        self.handlers.clear(&ident);
    }

    pub fn set_handler_fallback(&mut self, fallback: impl FnMut(i32, DcsAction) + 'static) {
        self.fallback = Some(Box::new(fallback));
    }

    /// Match explicit upstream disposal, which does not call active handlers.
    pub fn dispose(&mut self) {
        self.handlers.clear_all();
        self.fallback = None;
        self.active = Rc::default();
    }

    fn report(&mut self, action: DcsAction) {
        if let Some(fallback) = &mut self.fallback {
            fallback(self.ident, action);
        }
    }

    fn at(&self, index: usize) -> Result<HandlerHandle, DispatchError> {
        self.active
            .borrow()
            .get(index)
            .cloned()
            .ok_or(DispatchError::ActiveHandlerRemoved)
    }

    /// Abort remaining handlers. Previously returned futures are caller-owned.
    /// Reset retains pending cleanup futures for `take_cleanup`, because Rust
    /// futures are lazy rather than automatically scheduled like JS promises.
    pub fn reset(&mut self) -> Result<(), DispatchError> {
        let end = self
            .pause
            .as_ref()
            .map_or_else(|| self.active.borrow().len(), |pause| pause.next);
        for index in (0..end).rev() {
            let handler = self.at(index)?;
            let result = handler.borrow_mut().unhook(false);
            match result {
                HandlerResult::Ready(_) => {}
                HandlerResult::Pending(_) | HandlerResult::PendingFallible(_) => {
                    self.cleanup.push(result)
                }
                HandlerResult::Failed(error) => return Err(error),
            }
        }
        self.finish();
        Ok(())
    }

    /// Transfer pending reset cleanup to the caller's single-threaded executor.
    /// Drive these futures even if the terminal has since begun another DCS.
    pub fn take_cleanup(&mut self) -> Vec<HandlerResult> {
        std::mem::take(&mut self.cleanup)
    }

    fn finish(&mut self) {
        self.pause = None;
        self.active = Rc::default();
        self.ident = 0;
    }

    pub fn hook(&mut self, ident: i32, params: &Params) -> Result<(), DispatchError> {
        self.reset()?;
        self.ident = ident;
        self.active = self.handlers.get(&ident);
        let count = self.active.borrow().len();
        if count == 0 {
            self.report(DcsAction::Hook(params.clone()));
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().hook(params);
            }
        }
        Ok(())
    }

    /// Pass a slice to represent the upstream `start..end` range.
    pub fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        let count = self.active.borrow().len();
        if count == 0 {
            self.report(DcsAction::Put(utf32_to_string(data)));
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().put(data);
            }
        }
        Ok(())
    }

    /// `None` means completion. Only pending handler results are returned.
    /// Supply `true` for `previous_result` on the initial call, as upstream does.
    pub fn unhook(
        &mut self,
        success: bool,
        previous_result: bool,
    ) -> Result<Option<HandlerResult>, DispatchError> {
        let count = self.active.borrow().len();
        if count == 0 {
            self.report(DcsAction::Unhook(success));
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
            let result = handler.borrow_mut().unhook(success && !cleanup);
            match result {
                HandlerResult::Ready(handled) => cleanup |= handled,
                HandlerResult::Failed(error) => return Err(error),
                HandlerResult::Pending(_) | HandlerResult::PendingFallible(_) => {
                    self.pause = Some(Pause { next, cleanup });
                    return Ok(Some(result));
                }
            }
        }
        self.finish();
        Ok(None)
    }
}
