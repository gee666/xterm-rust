// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-clipboard/src/ClipboardAddon.ts.

//! OSC 52 with synchronous and asynchronous providers. Browser adapters must
//! propagate permission errors, not convert rejected promises to success.

use std::cell::RefCell;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use xterm_core::lifecycle::{Disposable, DisposableHandle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipboardError(pub String);
impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for ClipboardError {}

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

pub struct ClipboardAddon {
    codec: Rc<dyn Base64Codec>,
    provider: Rc<dyn ClipboardProvider>,
    disposable: RefCell<Option<DisposableHandle>>,
}

impl ClipboardAddon {
    pub fn new(provider: Rc<dyn ClipboardProvider>, codec: Option<Rc<dyn Base64Codec>>) -> Self {
        Self {
            codec: codec.unwrap_or_else(|| Rc::new(Base64)),
            provider,
            disposable: RefCell::default(),
        }
    }

    pub fn activate(&self, terminal: Rc<dyn ClipboardTerminal>) {
        let codec = Rc::clone(&self.codec);
        let provider = Rc::clone(&self.provider);
        // A terminal owns its callback. A weak reference avoids retaining it.
        let target = Rc::downgrade(&terminal);
        let callback = Rc::new(move |data: &[u16]| {
            let Some(target) = target.upgrade() else {
                return Completion::Ready(Err(ClipboardError(
                    "clipboard terminal was dropped".into(),
                )));
            };
            handle_osc(data, Rc::clone(&codec), Rc::clone(&provider), target)
        });
        *self.disposable.borrow_mut() = Some(terminal.register_osc_handler(52, callback));
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

pub fn handle_osc(
    data: &[u16],
    codec: Rc<dyn Base64Codec>,
    provider: Rc<dyn ClipboardProvider>,
    terminal: Rc<dyn ClipboardTerminal>,
) -> Completion<bool> {
    let mut args = data.split(|unit| *unit == u16::from(b';'));
    let selection = args.next().unwrap_or_default();
    let Some(payload) = args.next() else {
        return Completion::Ready(Ok(true));
    };
    if payload == [u16::from(b'?')] {
        let selection = selection.to_vec();
        provider.read_text(&selection).map(move |data| {
            let encoded = codec.encode_text(&data)?;
            let mut reply: Vec<u16> = "\u{1b}]52;".encode_utf16().collect();
            reply.extend(selection);
            reply.push(u16::from(b';'));
            reply.extend(encoded);
            reply.push(7);
            terminal.input(&reply, false)?;
            Ok(true)
        })
    } else {
        // Clearing on decode failure is the upstream OSC 52 contract.
        let text = codec.decode_text(payload).unwrap_or_default();
        provider.write_text(selection, &text).map(|()| Ok(true))
    }
}
