// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

//! Shared synchronous/asynchronous result used by streaming parser handlers.

use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Recoverable equivalent of upstream's TypeError when a callback removes a
/// handler needed by the ongoing reverse traversal. Partial state is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchError {
    ActiveHandlerRemoved,
}

impl fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("active handler removed during dispatch")
    }
}

impl std::error::Error for DispatchError {}

/// An immediate result or an owned, single-threaded future.
///
/// Parser dispatchers return pending futures to their caller. Await the future
/// before resuming dispatch with its boolean result. This keeps reset and
/// cancellation possible without holding a mutable parser borrow across await.
pub enum HandlerResult {
    Ready(bool),
    Pending(Pin<Box<dyn Future<Output = bool>>>),
}
