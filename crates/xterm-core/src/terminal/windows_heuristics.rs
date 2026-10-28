// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::TerminalError;
use crate::{
    buffer::BufferService,
    event::{EmitError, Emitter},
    lifecycle::{combined_disposable, Disposable, MutableDisposable},
    options::{OptionError, OptionValue},
    parser::{
        escape::{CommandRegistrar, FunctionIdentifier},
        handler::{DispatchError, HandlerResult},
        params::Params,
    },
};
use std::{cell::RefCell, rc::Rc};

/// CoreTerminal installs these two hooks only on a disabled-to-enabled transition.
/// Their position among user listeners/handlers, not just their boolean policy,
/// is observable when users inspect or mutate lines during callback dispatch.
pub(super) struct WindowsWrapping {
    service: Rc<BufferService>,
    line_feed: Rc<Emitter<()>>,
    csi: CommandRegistrar,
    hooks: MutableDisposable,
}
impl WindowsWrapping {
    pub(super) fn new(
        service: Rc<BufferService>,
        line_feed: Rc<Emitter<()>>,
        csi: CommandRegistrar,
    ) -> Self {
        Self {
            service,
            line_feed,
            csi,
            hooks: MutableDisposable::new(),
        }
    }
    pub(super) fn update(&self, pty: &OptionValue) -> Result<(), TerminalError> {
        if !enabled(pty)? {
            self.hooks.clear();
        } else if self.hooks.value().is_none() {
            let service = Rc::clone(&self.service);
            let line_feed = self
                .line_feed
                .event()
                .try_subscribe(move |_| update(&service).map_err(EmitError::callback));
            let service = Rc::clone(&self.service);
            let csi = self
                .csi
                .register(
                    FunctionIdentifier::new("H"),
                    Rc::new(RefCell::new(move |_: &Params| match update(&service) {
                        Ok(()) => HandlerResult::Ready(false),
                        Err(error) => HandlerResult::Failed(DispatchError::Callback(
                            EmitError::callback(error),
                        )),
                    })),
                )
                .map_err(|error| TerminalError::Event(EmitError::callback(error)))?;
            self.hooks
                .set(Some(combined_disposable(vec![line_feed, csi])));
        }
        Ok(())
    }
}
impl Disposable for WindowsWrapping {
    fn dispose(&self) {
        self.hooks.dispose();
    }
}
pub(crate) fn dispose(shared: &Rc<RefCell<super::TerminalState>>) -> Result<(), TerminalError> {
    let wrapping = Rc::clone(
        &shared
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .windows_wrapping,
    );
    wrapping.dispose();
    Ok(())
}
fn update(service: &BufferService) -> Result<(), TerminalError> {
    let active = service.buffer();
    let buffer = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    crate::windows_mode::update_windows_mode_wrapped_state(
        &buffer.lines,
        buffer.ybase,
        buffer.y,
        service.cols(),
    );
    Ok(())
}
fn enabled(pty: &OptionValue) -> Result<bool, OptionError> {
    if !pty.is_truthy() {
        return Ok(false);
    }
    if matches!(pty.get_property("backend")?, OptionValue::Undefined)
        || matches!(pty.get_property("buildNumber")?, OptionValue::Undefined)
    {
        return Ok(false);
    }
    // Keep the source's second reads: host getters can change values in between.
    Ok(pty.get_property("backend")? == OptionValue::from("conpty")
        && pty.get_property("buildNumber")?.number()? < 21376.0)
}
