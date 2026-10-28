// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

//! Shared synchronous/asynchronous result used by streaming parser handlers.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

#[path = "handler_registry.rs"]
pub mod registry;

/// Recoverable equivalent of upstream's TypeError when a callback removes a
/// handler needed by the ongoing reverse traversal. Partial state is retained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DispatchError {
    ActiveHandlerRemoved,
    /// A synchronous addon/event callback failed. Dispatch must stop before the
    /// next terminal input is interpreted, preserving already committed state.
    CallbackFailed,
    /// Preserve the original typed listener failure for native and WASM hosts.
    Callback(crate::event::EmitError),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ActiveHandlerRemoved => {
                formatter.write_str("active handler removed during dispatch")
            }
            Self::CallbackFailed => formatter.write_str("terminal handler callback failed"),
            Self::Callback(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for DispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Callback(error) => Some(error),
            Self::ActiveHandlerRemoved | Self::CallbackFailed => None,
        }
    }
}

/// An immediate result or an owned, single-threaded future.
///
/// Parser dispatchers return pending futures to their caller. Await the future
/// before resuming dispatch with its boolean result. This keeps reset and
/// cancellation possible without holding a mutable parser borrow across await.
pub enum HandlerResult {
    Ready(bool),
    Pending(Pin<Box<dyn Future<Output = bool>>>),
    /// A rejected asynchronous callback stops input processing. Do not resume
    /// the parser on error; its paused state remains available for reset.
    PendingFallible(Pin<Box<dyn Future<Output = Result<bool, DispatchError>>>>),
    /// Unlike a false return, an exception must not bubble to older handlers.
    Failed(DispatchError),
}

impl HandlerResult {
    /// Await either future representation without treating an exception as false.
    /// Resume a paused parser only for `Ok`, passing the returned boolean.
    pub async fn resolve(self) -> Result<bool, DispatchError> {
        match self {
            Self::Ready(value) => Ok(value),
            Self::Pending(future) => Ok(future.await),
            Self::PendingFallible(future) => future.await,
            Self::Failed(error) => Err(error),
        }
    }
}

impl From<Result<bool, crate::event::EmitError>> for HandlerResult {
    fn from(value: Result<bool, crate::event::EmitError>) -> Self {
        match value {
            Ok(handled) => Self::Ready(handled),
            Err(error) => Self::Failed(DispatchError::Callback(error)),
        }
    }
}
