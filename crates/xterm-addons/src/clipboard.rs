// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-clipboard/src/ClipboardAddon.ts.

//! OSC 52 with synchronous and asynchronous providers. Browser adapters must
//! propagate permission errors, not convert rejected promises to success.

use std::cell::RefCell;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use xterm_core::lifecycle::{Disposable, DisposableHandle};

pub const OSC_CLIPBOARD: u16 = 52;
const REPLY_PREFIX: &str = "\u{1b}]52;";

#[derive(Clone, Debug)]
pub enum ClipboardError {
    Message(String),
    Host(Rc<dyn std::error::Error>),
}
impl ClipboardError {
    pub fn host(error: impl std::error::Error + 'static) -> Self {
        Self::Host(Rc::new(error))
    }
}
impl PartialEq for ClipboardError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Message(a), Self::Message(b)) => a == b,
            (Self::Host(a), Self::Host(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for ClipboardError {}
impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ClipboardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Host(error) => Some(error.as_ref()),
            Self::Message(_) => None,
        }
    }
}

pub enum Completion<T> {
    Ready(Result<T, ClipboardError>),
    Pending(Pin<Box<dyn Future<Output = Result<T, ClipboardError>>>>),
}

impl<T: 'static> Completion<T> {
    pub fn map<U: 'static>(
        self,
        map: impl FnOnce(T) -> Result<U, ClipboardError> + 'static,
    ) -> Completion<U> {
        match self {
            Self::Ready(result) => Completion::Ready(result.and_then(map)),
            Self::Pending(future) => {
                Completion::Pending(Box::pin(async move { map(future.await?) }))
            }
        }
    }
}

pub trait Base64Codec {
    fn encode_text(&self, text: &[u16]) -> Result<Vec<u16>, ClipboardError>;
    fn decode_text(&self, text: &[u16]) -> Result<Vec<u16>, ClipboardError>;
}

#[derive(Default)]
pub struct Base64;
impl Base64Codec for Base64 {
    fn encode_text(&self, text: &[u16]) -> Result<Vec<u16>, ClipboardError> {
        Ok(crate::base64::encode_text(text).encode_utf16().collect())
    }
    fn decode_text(&self, text: &[u16]) -> Result<Vec<u16>, ClipboardError> {
        Ok(crate::base64::decode_text(text))
    }
}

pub trait ClipboardProvider {
    fn read_text(&self, selection: &[u16]) -> Completion<Vec<u16>>;
    fn write_text(&self, selection: &[u16], text: &[u16]) -> Completion<()>;
}

pub type OscCallback = Rc<dyn Fn(&[u16]) -> Completion<bool>>;
pub trait ClipboardTerminal {
    fn register_osc_handler(&self, identifier: u16, handler: OscCallback) -> DisposableHandle;
    /// Reply is input with was_user_input=false, not a write to the display.
    fn input(&self, data: &[u16], was_user_input: bool) -> Result<(), ClipboardError>;
}

pub trait ClipboardReply {
    fn send_reply(&self, data: &[u16]) -> Result<(), ClipboardError>;
}
impl<T: ClipboardTerminal + ?Sized> ClipboardReply for T {
    fn send_reply(&self, data: &[u16]) -> Result<(), ClipboardError> {
        self.input(data, false)
    }
}
struct CurrentReply(Rc<RefCell<Option<Weak<dyn ClipboardTerminal>>>>);
impl ClipboardReply for CurrentReply {
    fn send_reply(&self, data: &[u16]) -> Result<(), ClipboardError> {
        let terminal = self
            .0
            .borrow()
            .clone()
            .and_then(|target| target.upgrade())
            .ok_or_else(|| ClipboardError::Message("clipboard terminal was dropped".into()))?;
        terminal.input(data, false)
    }
}

pub struct ClipboardAddon {
    target: Rc<RefCell<Option<Weak<dyn ClipboardTerminal>>>>,
    codec: Rc<dyn Base64Codec>,
    provider: Rc<dyn ClipboardProvider>,
    disposable: RefCell<Option<DisposableHandle>>,
}

impl ClipboardAddon {
    pub fn new(provider: Rc<dyn ClipboardProvider>, codec: Option<Rc<dyn Base64Codec>>) -> Self {
        Self {
            target: Rc::default(),
            codec: codec.unwrap_or_else(|| Rc::new(Base64)),
            provider,
            disposable: RefCell::default(),
        }
    }

    pub fn activate(&self, terminal: Rc<dyn ClipboardTerminal>) {
        let codec = Rc::clone(&self.codec);
        let provider = Rc::clone(&self.provider);
        // Pending queries use the most recently activated terminal, as source
        // _readText reads this._terminal only when the provider settles.
        *self.target.borrow_mut() = Some(Rc::downgrade(&terminal));
        let target = Rc::new(CurrentReply(Rc::clone(&self.target)));
        let callback = Rc::new(move |data: &[u16]| {
            handle_osc(
                data,
                Rc::clone(&codec),
                Rc::clone(&provider),
                Rc::clone(&target),
            )
        });
        *self.disposable.borrow_mut() =
            Some(terminal.register_osc_handler(OSC_CLIPBOARD, callback));
    }
}

impl Disposable for ClipboardAddon {
    fn dispose(&self) {
        let handle = self.disposable.borrow().clone();
        if let Some(handle) = handle {
            handle.dispose();
        }
    }
}
impl Drop for ClipboardAddon {
    fn drop(&mut self) {
        self.dispose();
    }
}

pub enum ClipboardRequest<'a> {
    Ignored,
    Read(&'a [u16]),
    Write {
        selection: &'a [u16],
        payload: &'a [u16],
    },
}
pub fn parse_request(data: &[u16]) -> ClipboardRequest<'_> {
    let mut args = data.split(|unit| *unit == u16::from(b';'));
    let selection = args.next().unwrap_or_default();
    let Some(payload) = args.next() else {
        return ClipboardRequest::Ignored;
    };
    if payload == [u16::from(b'?')] {
        ClipboardRequest::Read(selection)
    } else {
        ClipboardRequest::Write { selection, payload }
    }
}
pub fn query_reply(selection: &[u16], encoded: &[u16]) -> Vec<u16> {
    let mut reply: Vec<u16> = REPLY_PREFIX.encode_utf16().collect();
    reply.extend_from_slice(selection);
    reply.push(u16::from(b';'));
    reply.extend_from_slice(encoded);
    reply.push(7);
    reply
}
pub fn handle_osc<T: ClipboardReply + ?Sized + 'static>(
    data: &[u16],
    codec: Rc<dyn Base64Codec>,
    provider: Rc<dyn ClipboardProvider>,
    terminal: Rc<T>,
) -> Completion<bool> {
    match parse_request(data) {
        ClipboardRequest::Ignored => Completion::Ready(Ok(true)),
        ClipboardRequest::Read(selection) => {
            let selection = selection.to_vec();
            provider.read_text(&selection).map(move |data| {
                let encoded = codec.encode_text(&data)?;
                terminal.send_reply(&query_reply(&selection, &encoded))?;
                Ok(true)
            })
        }
        ClipboardRequest::Write { selection, payload } => {
            // Clearing on decode failure is the upstream OSC52 contract.
            let text = codec.decode_text(payload).unwrap_or_default();
            provider.write_text(selection, &text).map(|()| Ok(true))
        }
    }
}
