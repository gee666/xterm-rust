// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Parser test futures have no external I/O and resolve on their first poll.

use std::future::Future;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_core::parser::handler::{DispatchError, HandlerResult};

struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}

pub fn resolve(result: HandlerResult) -> bool {
    resolve_checked(result).expect("fixture callback failed")
}

pub fn resolve_checked(result: HandlerResult) -> Result<bool, DispatchError> {
    let waker = Waker::from(Arc::new(NoWake));
    let mut future = Box::pin(result.resolve());
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture future unexpectedly requires external scheduling"),
    }
}
