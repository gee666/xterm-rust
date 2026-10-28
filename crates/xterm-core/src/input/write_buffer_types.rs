// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::WeakWriteBuffer;
use crate::{event::EmitError, parser::handler::HandlerResult};
use std::{any::Any, fmt, future::Future, pin::Pin, rc::Rc};

/// An adapter-owned byte view, such as a mutable/detachable JS Uint8Array.
/// The queue observes length when enqueuing and again after parsing, as source
/// does. The host action downcasts this value and reads bytes only when parsed.
/// It is always truthy, including an empty or detached view.
pub trait WritePayload: fmt::Debug + 'static {
    fn len(&self) -> Result<usize, WriteError>;
    fn is_empty(&self) -> Result<bool, WriteError> {
        self.len().map(|length| length == 0)
    }
    fn as_any(&self) -> &dyn Any;
    /// Convert the same shared allocation into a live parser source. Custom
    /// payloads interpreted by another action may legitimately lack this adapter.
    fn into_parse_source(
        self: Rc<Self>,
    ) -> Option<Rc<dyn crate::input::parse_source::ParseInputSource>> {
        None
    }
}
/// Text retains UTF-16 units. Byte chunks remain distinct, including empty ones.
#[derive(Clone, Debug)]
pub enum WriteData {
    Text(Vec<u16>),
    Bytes(Vec<u8>),
    External(Rc<dyn WritePayload>),
}
impl PartialEq for WriteData {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a == b,
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
            (Self::External(a), Self::External(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for WriteData {}
impl WriteData {
    pub fn text(value: &str) -> Self {
        Self::Text(value.encode_utf16().collect())
    }
    pub fn len(&self) -> Result<usize, WriteError> {
        match self {
            Self::Text(v) => Ok(v.len()),
            Self::Bytes(v) => Ok(v.len()),
            Self::External(v) => v.len(),
        }
    }
    pub fn is_empty(&self) -> Result<bool, WriteError> {
        self.len().map(|length| length == 0)
    }
    pub(super) fn truthy(&self) -> bool {
        !matches!(self, Self::Text(v) if v.is_empty())
    }
}
#[derive(Debug, Clone)]
pub enum WriteError {
    Discarded,
    Action(String),
    Callback(String),
    Schedule(String),
    Event(EmitError),
    /// An actual parser/host exception retained without string conversion.
    External(Rc<dyn std::error::Error>),
}
impl WriteError {
    pub fn external(error: impl std::error::Error + 'static) -> Self {
        Self::External(Rc::new(error))
    }
    pub fn external_ref<E: std::error::Error + 'static>(&self) -> Option<&E> {
        match self {
            Self::External(error) => error.downcast_ref(),
            _ => None,
        }
    }
}
impl PartialEq for WriteError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Discarded, Self::Discarded) => true,
            (Self::Action(a), Self::Action(b))
            | (Self::Callback(a), Self::Callback(b))
            | (Self::Schedule(a), Self::Schedule(b)) => a == b,
            (Self::Event(a), Self::Event(b)) => a == b,
            (Self::External(a), Self::External(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for WriteError {}
impl fmt::Display for WriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Discarded => {
                formatter.write_str("write data discarded, use flow control to avoid losing data")
            }
            Self::Action(message) | Self::Callback(message) | Self::Schedule(message) => {
                formatter.write_str(message)
            }
            Self::Event(error) => error.fmt(formatter),
            Self::External(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for WriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Event(error) => Some(error),
            Self::External(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

pub enum WriteAction {
    Complete,
    /// Rejection is reported asynchronously, then parsing resumes with false.
    Pending(Pin<Box<dyn Future<Output = Result<bool, WriteError>>>>),
}
impl WriteAction {
    pub fn from_parser(result: Option<HandlerResult>) -> Result<Self, WriteError> {
        match result {
            Some(HandlerResult::Pending(future)) => {
                Ok(Self::Pending(Box::pin(async move { Ok(future.await) })))
            }
            Some(HandlerResult::PendingFallible(future)) => {
                Ok(Self::Pending(Box::pin(async move {
                    future.await.map_err(WriteError::external)
                })))
            }
            Some(HandlerResult::Failed(error)) => Err(WriteError::external(error)),
            Some(HandlerResult::Ready(_)) | None => Ok(Self::Complete),
        }
    }
}

/// Zero-delay timers and local future execution belong to the host adapter.
/// `request` stores its weak handle and arguments until a later callback calls
/// `upgrade()?.process(last_time, promise_result)`. Cancellation prevents that
/// callback. `spawn` polls only after returning, and reports process failures.
/// Supplied timer callbacks and future polling must never execute inline.
/// Other host code, such as a browser getter, may reenter: the core holds no
/// scheduler or queue-slot borrow while calling the host. Adapters likewise
/// release interior borrows before foreign calls. `report_error` schedules or
/// records the original asynchronous exception without swallowing it.
pub trait WriteScheduler: Sized + 'static {
    /// A cloneable host token, not an RAII cancellation owner. Dropping a
    /// token must not cancel its callback; cancellation is explicit below.
    type Handle: Clone;
    fn request(
        &self,
        queue: WeakWriteBuffer<Self>,
        last_time: f64,
        promise_result: bool,
    ) -> Result<Self::Handle, WriteError>;
    fn cancel(&self, handle: Self::Handle) -> Result<(), WriteError>;
    fn now(&self) -> f64;
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()>>>);
    fn report_error(&self, error: WriteError);
}
