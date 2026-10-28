// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Buffer-owned cleanup uses the same native timer lane as queued writes.
use super::NativeWriteScheduler;
use crate::{
    buffer::{Buffer, BufferCleanup, BufferError},
    event::EmitError,
    input::write_buffer::{WriteError, WriteScheduler},
    log_service::{LogLevel, LogService},
    task_queue::{CallbackKind, Scheduler},
};
use std::{
    any::Any,
    cell::{OnceCell, RefCell},
    rc::{Rc, Weak},
};

#[cfg(test)]
#[path = "native_cleanup_tests.rs"]
mod tests;

type Adapter = BufferCleanup<CleanupScheduler>;
type Target = Rc<OnceCell<Weak<Adapter>>>;
#[derive(Clone)]
struct CleanupScheduler {
    host: NativeWriteScheduler,
    target: Target,
    logger: Weak<LogService>,
}
impl Scheduler for CleanupScheduler {
    type Handle = u64;
    type Error = WriteError;
    fn request(&self, kind: CallbackKind) -> Result<Self::Handle, Self::Error> {
        if kind != CallbackKind::Priority {
            return Err(WriteError::Schedule(
                "native cleanup host has no idle callback API".into(),
            ));
        }
        let target = Rc::clone(&self.target);
        self.host.request_callback(move || {
            let weak = target.get().ok_or_else(|| {
                WriteError::Schedule(
                    "native host executed cleanup before attachment completed".into(),
                )
            })?;
            // Cancellation may remove the last adapter owner before dispatch.
            let Some(adapter) = weak.upgrade() else {
                return Ok(());
            };
            adapter.process_priority().map_err(WriteError::external)
        })
    }
    fn cancel(&self, handle: Self::Handle) {
        self.host.cancel_timer(handle);
    }
    fn now(&self) -> f64 {
        self.host.now()
    }
    fn deadline_exceeded(&self, milliseconds: f64) -> Result<(), Self::Error> {
        let logger = self
            .logger
            .upgrade()
            .ok_or_else(|| WriteError::Schedule("native cleanup logger was released".into()))?;
        let mut number = ryu_js::Buffer::new();
        let message: Vec<u16> = format!(
            "task queue exceeded allotted deadline by {}ms",
            number.format(milliseconds)
        )
        .encode_utf16()
        .collect();
        logger
            .log(LogLevel::Warn, &message, Vec::new())
            .map_err(WriteError::external)
    }
}

/// Returned ownership belongs to the actual Buffer, matching its source queue.
/// The adapter references that Buffer weakly; the scheduler never retains its
/// parent Terminal or logger strongly. The host pump owns callback execution.
pub(super) fn attach(
    buffer: &Rc<RefCell<Buffer>>,
    host: &NativeWriteScheduler,
    logger: &Weak<LogService>,
) -> Result<Box<dyn Any>, BufferError> {
    let target = Rc::new(OnceCell::new());
    let scheduler = CleanupScheduler {
        host: host.clone(),
        target: Rc::clone(&target),
        logger: logger.clone(),
    };
    let adapter = Rc::new(
        BufferCleanup::new(buffer, scheduler, false)
            .map_err(|error| BufferError::Dispatch(EmitError::callback(error)))?,
    );
    // This private cell is new and no posted host callback can run inline.
    target
        .set(Rc::downgrade(&adapter))
        .unwrap_or_else(|_| unreachable!("new cleanup target is initialized once"));
    Ok(Box::new(adapter))
}
