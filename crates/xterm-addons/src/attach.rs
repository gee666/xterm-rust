// Copyright (c) 2014, 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-attach/src/AttachAddon.ts.

//! WebSocket attachment. Adapters own the socket and terminal event sources;
//! no socket borrow is held while an event callback executes.

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
use xterm_core::{
    input::{
        parse_source::{OwnedParseInput, ParseInputSource},
        write_buffer::WritePayload,
    },
    lifecycle::{Disposable, DisposableHandle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketState {
    Connecting,
    Open,
    Closing,
    Closed,
    Unknown,
}

#[derive(Clone, Debug)]
pub enum SocketMessage {
    Text(Vec<u16>),
    Binary(Vec<u8>),
    /// A retained mutable host view, read by the parser only when consumed.
    View(Rc<dyn WritePayload>),
}
impl PartialEq for SocketMessage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::Binary(a), Self::Binary(b)) => a == b,
            (Self::View(a), Self::View(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for SocketMessage {}
impl SocketMessage {
    pub fn into_parse_source(self) -> Result<Rc<dyn ParseInputSource>, AttachError> {
        match self {
            Self::Text(data) => Ok(Rc::new(OwnedParseInput::Text(data))),
            Self::Binary(data) => Ok(Rc::new(OwnedParseInput::Bytes(data))),
            Self::View(data) => data
                .into_parse_source()
                .ok_or_else(|| AttachError::Adapter("socket view has no parser source".into())),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SocketEvent {
    Message,
    Close,
    Error,
}

#[derive(Clone, Debug)]
pub enum AttachError {
    Connecting,
    Closed,
    UnexpectedState,
    Adapter(String),
    Host(Rc<dyn std::error::Error>),
}
impl AttachError {
    pub fn host(error: impl std::error::Error + 'static) -> Self {
        Self::Host(Rc::new(error))
    }
}
impl PartialEq for AttachError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Connecting, Self::Connecting)
            | (Self::Closed, Self::Closed)
            | (Self::UnexpectedState, Self::UnexpectedState) => true,
            (Self::Adapter(a), Self::Adapter(b)) => a == b,
            (Self::Host(a), Self::Host(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for AttachError {}

impl fmt::Display for AttachError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connecting => f.write_str("Attach addon was loaded before socket was open"),
            Self::Closed => f.write_str("Attach addon socket is closed"),
            Self::UnexpectedState => f.write_str("Unexpected socket state"),
            Self::Adapter(message) => f.write_str(message),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for AttachError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Host(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

pub type SocketCallback = Rc<dyn Fn(Option<SocketMessage>) -> Result<(), AttachError>>;
pub type DataCallback = Rc<dyn Fn(&[u16]) -> Result<(), AttachError>>;

/// Browser implementations must use ArrayBuffer, not Blob, for binary messages.
pub trait AttachSocket {
    fn use_array_buffer(&self) -> Result<(), AttachError>;
    fn state(&self) -> Result<SocketState, AttachError>;
    fn send(&self, data: SocketMessage) -> Result<(), AttachError>;
    fn listen(
        &self,
        event: SocketEvent,
        callback: SocketCallback,
    ) -> Result<DisposableHandle, AttachError>;
    fn warn_closing(&self) -> Result<(), AttachError>;
}

pub trait AttachTerminal {
    fn write(&self, data: SocketMessage) -> Result<(), AttachError>;
    fn on_data(&self, callback: DataCallback) -> DisposableHandle;
    fn on_binary(&self, callback: DataCallback) -> DisposableHandle;
}

pub struct AttachAddon {
    socket: Rc<dyn AttachSocket>,
    bidirectional: bool,
    disposables: Rc<RefCell<Vec<DisposableHandle>>>,
}

impl AttachAddon {
    pub fn new(
        socket: Rc<dyn AttachSocket>,
        bidirectional: Option<bool>,
    ) -> Result<Self, AttachError> {
        Self::with_options(socket, || Ok(bidirectional))
    }

    /// Socket configuration precedes dynamic option reads in JavaScript.
    pub fn with_options(
        socket: Rc<dyn AttachSocket>,
        options: impl FnOnce() -> Result<Option<bool>, AttachError>,
    ) -> Result<Self, AttachError> {
        socket.use_array_buffer()?;
        let bidirectional = options()?.unwrap_or(true);
        Ok(Self {
            socket,
            bidirectional,
            disposables: Rc::default(),
        })
    }

    pub fn activate(&self, terminal: Rc<dyn AttachTerminal>) -> Result<(), AttachError> {
        let target = Rc::downgrade(&terminal);
        let message = self.socket.listen(
            SocketEvent::Message,
            Rc::new(move |data| match data {
                Some(data) => target
                    .upgrade()
                    .ok_or_else(|| AttachError::Adapter("attached terminal was dropped".into()))?
                    .write(data),
                None => Err(AttachError::Adapter("message event has no payload".into())),
            }),
        )?;
        self.disposables.borrow_mut().push(message);
        if self.bidirectional {
            let socket = Rc::downgrade(&self.socket);
            let data = terminal.on_data(Rc::new(move |data| {
                let socket = socket
                    .upgrade()
                    .ok_or_else(|| AttachError::Adapter("attached socket was dropped".into()))?;
                send(&*socket, || SocketMessage::Text(data.to_vec()))
            }));
            self.disposables.borrow_mut().push(data);
            let socket = Rc::downgrade(&self.socket);
            let binary = terminal.on_binary(Rc::new(move |data| {
                let socket = socket
                    .upgrade()
                    .ok_or_else(|| AttachError::Adapter("attached socket was dropped".into()))?;
                // JS charCodeAt reads UTF-16 units, then masks each to one byte.
                send(&*socket, || {
                    SocketMessage::Binary(data.iter().map(|unit| (unit & 255) as u8).collect())
                })
            }));
            self.disposables.borrow_mut().push(binary);
        }
        for event in [SocketEvent::Close, SocketEvent::Error] {
            // Weak ownership prevents socket -> callback -> socket cycles.
            let disposables = Rc::downgrade(&self.disposables);
            let handle = self.socket.listen(
                event,
                Rc::new(move |_| {
                    if let Some(disposables) = disposables.upgrade() {
                        dispose_all(&disposables);
                    }
                    Ok(())
                }),
            )?;
            self.disposables.borrow_mut().push(handle);
        }
        Ok(())
    }
}

impl Disposable for AttachAddon {
    fn dispose(&self) {
        dispose_all(&self.disposables);
    }
}

impl Drop for AttachAddon {
    fn drop(&mut self) {
        self.dispose();
    }
}

fn dispose_all(disposables: &RefCell<Vec<DisposableHandle>>) {
    // Upstream retains the handles and invokes them on repeated dispose calls.
    let snapshot = disposables.borrow().clone();
    for disposable in snapshot {
        disposable.dispose();
    }
}

fn send(
    socket: &dyn AttachSocket,
    data: impl FnOnce() -> SocketMessage,
) -> Result<(), AttachError> {
    match socket.state()? {
        SocketState::Open => socket.send(data()),
        SocketState::Connecting => Err(AttachError::Connecting),
        SocketState::Closing => {
            socket.warn_closing()?;
            Ok(())
        }
        SocketState::Closed => Err(AttachError::Closed),
        SocketState::Unknown => Err(AttachError::UnexpectedState),
    }
}
