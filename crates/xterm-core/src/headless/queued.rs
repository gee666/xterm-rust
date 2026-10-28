use super::{native_cleanup, HeadlessError, NativeWritePump, NativeWriteScheduler, Terminal};
use crate::{
    buffer::buffer_set::{CleanupFactory, CleanupFactoryHandle},
    input::write_buffer::{
        WriteAction, WriteBuffer, WriteCallback, WriteData, WriteError, WriteScheduler,
    },
    lifecycle::DisposableHandle,
};
use std::{cell::Cell, rc::Rc};

/// Native source-style write queue with an explicit event pump. The terminal's
/// synchronous convenience methods remain available but must not interleave
/// with unresolved queued input. Browser hosts use their own scheduler instead.
pub struct QueuedTerminal {
    terminal: Rc<Terminal>,
    queue: WriteBuffer<NativeWriteScheduler>,
    pump: NativeWritePump,
    parsed: DisposableHandle,
    input: DisposableHandle,
    terminal_dispose: DisposableHandle,
    cleanup: CleanupFactoryHandle,
}
impl QueuedTerminal {
    pub fn new(terminal: Rc<Terminal>) -> Result<Self, HeadlessError> {
        Self::with_pump(terminal, NativeWritePump::new())
    }
    pub fn with_pump(terminal: Rc<Terminal>, pump: NativeWritePump) -> Result<Self, HeadlessError> {
        let service = terminal.with_state(|state| Rc::clone(&state.buffer_service))?;
        let logger = terminal.with_state(|state| Rc::downgrade(&state.logger))?;
        let cleanup_host = pump.scheduler();
        let factory: CleanupFactory =
            Rc::new(move |buffer| native_cleanup::attach(buffer, &cleanup_host, &logger));
        let cleanup = service
            .install_cleanup_factory(factory)
            .map_err(crate::terminal::TerminalError::from)?;
        let target = Rc::downgrade(&terminal);
        let pending = Cell::new(false);
        let cleanup_scheduler = pump.scheduler();
        let queue = WriteBuffer::new_owned(pump.scheduler(), move |_, data, result| {
            let terminal = target
                .upgrade()
                .ok_or_else(|| WriteError::Action("native terminal was dropped".into()))?;
            // The source queue passes true even for a first call. Only a prior
            // suspended parser result makes that argument a real continuation.
            let resume = if pending.get() {
                let result = result.ok_or_else(|| {
                    WriteError::external(crate::terminal::TerminalError::Parse(
                        crate::parser::escape::ParseError::ImproperContinuation,
                    ))
                })?;
                pending.set(false);
                Some(result)
            } else {
                None
            };
            let action = if let Some(result) = resume {
                terminal.parse_write_data(&data, Some(result))
            } else {
                data.into_parse_source()
                    .and_then(|source| terminal.parse_write_source(source, None))
            };
            for cleanup in terminal.take_cleanup().map_err(WriteError::external)? {
                let scheduler = cleanup_scheduler.clone();
                let errors = cleanup_scheduler.clone();
                scheduler.spawn(Box::pin(async move {
                    if let Err(error) = cleanup.resolve().await {
                        errors.report_error(WriteError::external(error));
                    }
                }));
            }
            let action = action?;
            pending.set(matches!(action, WriteAction::Pending(_)));
            Ok(action)
        });
        let target = Rc::downgrade(&terminal);
        let parsed = queue.on_write_parsed().try_subscribe(move |_| {
            if let Some(terminal) = target.upgrade() {
                terminal.notify_write_parsed()?;
            }
            Ok(())
        });
        let core = terminal.with_state(|state| Rc::clone(&state.core))?;
        let weak = queue.downgrade();
        let input = core.on_user_input().subscribe(move |_| {
            if let Some(queue) = weak.upgrade() {
                queue.handle_user_input();
            }
        });
        let weak = queue.downgrade();
        let terminal_dispose = terminal.on_dispose_request().try_subscribe(move |_| {
            if let Some(queue) = weak.upgrade() {
                queue
                    .try_dispose()
                    .map_err(crate::event::EmitError::callback)?;
            }
            Ok(())
        });
        Ok(Self {
            terminal,
            queue,
            pump,
            parsed,
            input,
            terminal_dispose,
            cleanup,
        })
    }
    pub fn terminal(&self) -> &Rc<Terminal> {
        &self.terminal
    }
    pub fn queue(&self) -> WriteBuffer<NativeWriteScheduler> {
        self.queue.clone()
    }
    pub fn pump(&self) -> &NativeWritePump {
        &self.pump
    }
    pub fn write(
        &self,
        data: WriteData,
        callback: Option<WriteCallback<NativeWriteScheduler>>,
    ) -> Result<(), WriteError> {
        self.queue.write(data, callback)
    }
    pub fn writeln(
        &self,
        data: WriteData,
        callback: Option<WriteCallback<NativeWriteScheduler>>,
    ) -> Result<(), WriteError> {
        self.queue.write(data, None)?;
        self.queue.write(WriteData::text("\r\n"), callback)
    }
    pub fn resize(&self, cols: usize, rows: usize) -> Result<(), WriteError> {
        if cols == self.terminal.cols().map_err(WriteError::external)?
            && rows == self.terminal.rows().map_err(WriteError::external)?
        {
            return Ok(());
        }
        self.queue.flush_sync()?;
        self.terminal
            .resize(cols, rows)
            .map_err(WriteError::external)
    }
    pub fn dispose(&self) -> Result<(), HeadlessError> {
        // The terminal closes its write buffer after source service cleanup.
        // Cancelling here first would change marker-disposal callback behavior.
        self.terminal.dispose()
    }
}
impl Drop for QueuedTerminal {
    fn drop(&mut self) {
        self.cleanup.dispose();
        self.queue.dispose();
        self.parsed.dispose();
        self.input.dispose();
        self.terminal_dispose.dispose();
    }
}
