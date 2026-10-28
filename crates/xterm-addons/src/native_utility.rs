//! Live Rust engine adapters for attach and OSC52 clipboard.
//!
//! Native writes are synchronous and propagate Pending. Browser hosts with the
//! JavaScript WriteBuffer contract must enqueue socket writes through that queue.
use crate::{
    attach::*,
    clipboard::{ClipboardError, ClipboardTerminal, Completion, OscCallback},
};
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    core_service::CoreService,
    event::EmitError,
    lifecycle::DisposableHandle,
    parser::{
        handler::{DispatchError, HandlerResult},
        osc::{OscHandler, OscRegistrar},
    },
    terminal::CoreTerminal,
};

enum EngineOwner {
    Core(Rc<RefCell<CoreTerminal>>),
    Public(Rc<xterm_core::headless::Terminal>),
}

pub struct NativeUtilityTerminal {
    engine: EngineOwner,
    core: Rc<CoreService>,
    osc: OscRegistrar,
    clipboard_error: Rc<RefCell<Option<ClipboardError>>>,
    attach_error: Rc<RefCell<Option<AttachError>>>,
}
impl NativeUtilityTerminal {
    pub fn new(engine: Rc<RefCell<CoreTerminal>>) -> Self {
        let core = Rc::clone(&engine.borrow().state().core);
        let osc = engine.borrow_mut().parser_mut().osc.registrar();
        Self {
            engine: EngineOwner::Core(engine),
            core,
            osc,
            clipboard_error: Rc::default(),
            attach_error: Rc::default(),
        }
    }
    pub fn from_headless(
        terminal: Rc<xterm_core::headless::Terminal>,
    ) -> Result<Self, xterm_core::headless::HeadlessError> {
        let core = terminal.with_state(|state| Rc::clone(&state.core))?;
        let (osc, _) = terminal.addon_registrars()?;
        Ok(Self {
            engine: EngineOwner::Public(terminal),
            core,
            osc,
            clipboard_error: Rc::default(),
            attach_error: Rc::default(),
        })
    }
    pub fn take_clipboard_error(&self) -> Option<ClipboardError> {
        self.clipboard_error.borrow_mut().take()
    }
    pub fn take_attach_error(&self) -> Option<AttachError> {
        self.attach_error.borrow_mut().take()
    }
}
impl ClipboardTerminal for NativeUtilityTerminal {
    fn register_osc_handler(&self, identifier: u16, handler: OscCallback) -> DisposableHandle {
        let errors = self.clipboard_error.clone();
        self.osc
            .register_handler(
                f64::from(identifier),
                Rc::new(RefCell::new(OscHandler::new(move |data| {
                    match handler(&data) {
                        Completion::Ready(Ok(handled)) => HandlerResult::Ready(handled),
                        Completion::Ready(Err(error)) => {
                            *errors.borrow_mut() = Some(error.clone());
                            HandlerResult::Failed(DispatchError::Callback(EmitError::callback(
                                error,
                            )))
                        }
                        Completion::Pending(future) => {
                            let errors = errors.clone();
                            HandlerResult::PendingFallible(Box::pin(async move {
                                future.await.map_err(|error| {
                                    *errors.borrow_mut() = Some(error.clone());
                                    DispatchError::Callback(EmitError::callback(error))
                                })
                            }))
                        }
                    }
                }))),
            )
            .expect("owned engine keeps OSC registry alive")
    }
    fn input(&self, data: &[u16], user: bool) -> Result<(), ClipboardError> {
        self.core
            .trigger_data_event(data, user)
            .map_err(ClipboardError::host)
    }
}
impl AttachTerminal for NativeUtilityTerminal {
    fn write(&self, data: SocketMessage) -> Result<(), AttachError> {
        let source = data.into_parse_source()?;
        match &self.engine {
            EngineOwner::Core(engine) => {
                let mut engine = engine.try_borrow_mut().map_err(|_| {
                    AttachError::Adapter("terminal is already parsing a write".into())
                })?;
                engine.write_source(source).map_err(AttachError::host)
            }
            EngineOwner::Public(terminal) => {
                terminal.write_source(source).map_err(AttachError::host)
            }
        }
    }
    fn on_data(&self, callback: DataCallback) -> DisposableHandle {
        let errors = self.attach_error.clone();
        self.core.on_data().try_subscribe(move |data| {
            callback(data).map_err(|error| {
                *errors.borrow_mut() = Some(error.clone());
                EmitError::Callback(Rc::new(error))
            })
        })
    }
    fn on_binary(&self, callback: DataCallback) -> DisposableHandle {
        let errors = self.attach_error.clone();
        self.core.on_binary().try_subscribe(move |data| {
            callback(data).map_err(|error| {
                *errors.borrow_mut() = Some(error.clone());
                EmitError::Callback(Rc::new(error))
            })
        })
    }
}
