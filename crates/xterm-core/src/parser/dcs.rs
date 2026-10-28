// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/parser/DcsParser.ts.

//! Streaming DCS dispatch. Handlers run newest first and remain active until
//! unhook or reset. Await a returned future and pass its result to `unhook`
//! with the same success flag. Identifiers retain the source's packed i32 bits.

use super::handler::registry::Registrar;
use super::handler::{DispatchError, HandlerResult};
use super::params::Params;
use crate::lifecycle::DisposableHandle;
use crate::text_decoder::utf32_to_string;
use std::cell::RefCell;
use std::rc::Rc;

#[path = "dcs_handler.rs"]
mod callback;
pub use callback::DcsHandler;
#[path = "dcs_control.rs"]
mod control;
use control::Control;
pub use control::DcsDisposalHandle;

pub trait DcsHandlerTrait {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError>;
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError>;
    fn unhook(&mut self, success: bool) -> HandlerResult;
}

pub type HandlerHandle = Rc<RefCell<dyn DcsHandlerTrait>>;

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
    control: Rc<Control>,
    ident: i32,
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
        self.control.register(ident, handler)
    }

    pub fn registrar(&self) -> DcsRegistrar {
        DcsRegistrar(self.control.registrar())
    }

    pub fn disposal_handle(&self) -> DcsDisposalHandle {
        DcsDisposalHandle::new(&self.control)
    }

    /// Remove future lookup without detaching an already active handler list.
    pub fn clear_handler(&mut self, ident: i32) {
        self.control.clear_handler(ident);
    }

    pub fn set_handler_fallback(&mut self, mut fallback: impl FnMut(i32, DcsAction) + 'static) {
        self.set_handler_fallback_checked(move |ident, action| {
            fallback(ident, action);
            Ok(())
        });
    }

    /// Fallible host callbacks, including custom diagnostic loggers, stop the
    /// current action before its following state cleanup when they throw.
    pub fn set_handler_fallback_checked(
        &mut self,
        fallback: impl FnMut(i32, DcsAction) -> Result<(), DispatchError> + 'static,
    ) {
        self.control.set_fallback(Box::new(fallback));
    }

    /// Match explicit upstream disposal, which does not call active handlers.
    pub fn dispose(&mut self) {
        self.control.dispose();
    }

    fn report(&mut self, action: DcsAction) -> Result<(), DispatchError> {
        self.control.report(self.ident, action)
    }

    fn at(&self, index: usize) -> Result<HandlerHandle, DispatchError> {
        self.control.at(index)
    }

    /// Abort remaining handlers. Previously returned futures are caller-owned.
    /// Reset retains pending cleanup futures for `take_cleanup`, because Rust
    /// futures are lazy rather than automatically scheduled like JS promises.
    pub fn reset(&mut self) -> Result<(), DispatchError> {
        let end = self
            .pause
            .as_ref()
            .map_or_else(|| self.control.active_len(), |pause| pause.next);
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
        self.control.clear_active();
        self.ident = 0;
    }

    pub fn hook(&mut self, ident: i32, params: &Params) -> Result<(), DispatchError> {
        self.reset()?;
        self.ident = ident;
        self.control.select(ident);
        let count = self.control.active_len();
        if count == 0 {
            self.report(DcsAction::Hook(params.clone()))?;
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().hook(params)?;
            }
        }
        Ok(())
    }

    /// Pass a slice to represent the upstream `start..end` range.
    pub fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        let count = self.control.active_len();
        if count == 0 {
            self.report(DcsAction::Put(utf32_to_string(data)))?;
        } else {
            for index in (0..count).rev() {
                self.at(index)?.borrow_mut().put(data)?;
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
        let count = self.control.active_len();
        if count == 0 {
            self.report(DcsAction::Unhook(success))?;
            self.finish();
            return Ok(None);
        }
        // JS clears its inner paused flag before resumed callbacks. If a lower
        // callback throws, reset aborts the whole active list again; restoring
        // this pause on error would silently change that source behavior.
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
