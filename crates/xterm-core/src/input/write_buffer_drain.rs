// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::*;

impl<S: WriteScheduler> WriteBuffer<S> {
    /// Deprecated upstream operation. Limits deliberately discard the rejected
    /// chunk; synchronous drains deliberately do not wait for pending futures.
    pub fn write_sync(
        &self,
        data: WriteData,
        max_subsequent_calls: Option<f64>,
    ) -> Result<(), WriteError> {
        if self.is_disposed() {
            return Ok(());
        }
        if max_subsequent_calls.is_some_and(|limit| self.state.sync_calls.get() > limit) {
            self.state.sync_calls.set(0.0);
            return Ok(());
        }
        self.enqueue(data, None)?;
        self.state.sync_calls.set(self.state.sync_calls.get() + 1.0);
        if self.state.sync_writing.replace(true) {
            return Ok(());
        }
        self.drain_sync()?;
        self.state.pending_units.set(0.0);
        self.state.offset.set(SYNC_OFFSET);
        self.state.sync_writing.set(false);
        self.state.sync_calls.set(0.0);
        Ok(())
    }
    pub fn flush_sync(&self) -> Result<(), WriteError> {
        if self.is_disposed() || self.state.sync_writing.replace(true) {
            return Ok(());
        }
        let processed = self.drain_sync()?;
        self.clear_data(SYNC_OFFSET);
        self.state.sync_writing.set(false);
        if processed {
            self.fire_parsed()?;
        }
        Ok(())
    }
    fn drain_sync(&self) -> Result<bool, WriteError> {
        let mut processed = false;
        loop {
            // Keep the callback queue separate: a falsy text chunk exits the
            // JS while condition before its corresponding callback is shifted.
            let next = self.state.data.borrow_mut().pop_front();
            let Some(data) = next.filter(|data| data.truthy()) else {
                break;
            };
            processed = true;
            let result = (self.state.action)(self, Rc::clone(&data), None)?;
            if let WriteAction::Pending(future) = result {
                // The source calls the async action but does not await it in
                // sync mode. Run its future without resuming the parser here.
                let weak = self.downgrade();
                self.state.scheduler.spawn(Box::pin(async move {
                    if let Err(error) = future.await {
                        if let Some(queue) = weak.upgrade() {
                            queue.state.scheduler.report_error(error);
                        }
                    }
                }));
            }
            let callback = self.state.callbacks.borrow_mut().pop_front().flatten();
            if let Some(callback) = callback {
                callback(self)?;
            }
        }
        Ok(processed)
    }
    pub(super) fn inner_write(
        &self,
        last_time: f64,
        promise_result: bool,
    ) -> Result<(), WriteError> {
        if self.is_disposed() {
            return Ok(());
        }
        let start = if last_time == 0.0 || last_time.is_nan() {
            self.now()
        } else {
            last_time
        };
        loop {
            let next = self
                .state
                .data
                .borrow()
                .get(self.state.offset.get())
                .cloned();
            let Some(data) = next else {
                break;
            };
            if let WriteAction::Pending(future) =
                (self.state.action)(self, Rc::clone(&data), Some(promise_result))?
            {
                self.await_handler(future, start);
                return Ok(());
            }
            let callback = self
                .state
                .callbacks
                .borrow()
                .get(self.state.offset.get())
                .cloned()
                .flatten();
            if let Some(callback) = callback {
                callback(self)?;
            }
            self.state
                .offset
                .set(self.state.offset.get().saturating_add(1));
            let pending = self.pending_units();
            let length = data.len()?;
            self.state.pending_units.set(pending - length as f64);
            if self.now() - start >= WRITE_TIMEOUT_MS {
                break;
            }
        }
        let offset = self.state.offset.get();
        if self.state.data.borrow().len() > offset {
            if offset > LENGTH_THRESHOLD {
                self.state.data.borrow_mut().drain(..offset);
                self.state.callbacks.borrow_mut().drain(..offset);
                self.state.offset.set(0);
            }
            self.schedule(0.0, true)?;
        } else {
            self.clear_data(0);
        }
        self.fire_parsed()
    }
    fn await_handler(
        &self,
        future: Pin<Box<dyn Future<Output = Result<bool, WriteError>>>>,
        start: f64,
    ) {
        let weak = self.downgrade();
        self.state.scheduler.spawn(Box::pin(async move {
            let result = future.await;
            let Some(queue) = weak.upgrade() else {
                return;
            };
            match result {
                Ok(value) => queue.resume_handler(start, value),
                Err(error) => {
                    queue.state.scheduler.report_error(error);
                    if queue.is_disposed() {
                        return;
                    }
                    // Source catch queues the uncaught error before its
                    // resolved-false promise schedules parser continuation.
                    let weak = queue.downgrade();
                    queue.state.scheduler.spawn(Box::pin(async move {
                        if let Some(queue) = weak.upgrade() {
                            queue.resume_handler(start, false);
                        }
                    }));
                }
            }
        }));
    }
    fn resume_handler(&self, start: f64, result: bool) {
        if self.is_disposed() {
            return;
        }
        let resumed = if self.now() - start >= WRITE_TIMEOUT_MS {
            self.schedule(0.0, result)
        } else {
            self.inner_write(start, result)
        };
        if let Err(error) = resumed {
            self.state.scheduler.report_error(error);
        }
    }
}
