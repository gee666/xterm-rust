use super::{HeadlessError, NativeWritePump, NativeWriteScheduler, Terminal};
use crate::{
    input::write_buffer::{WriteAction, WriteBuffer, WriteCallback, WriteData, WriteError},
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
}
impl QueuedTerminal {
    pub fn new(terminal: Rc<Terminal>) -> Result<Self, HeadlessError> {
        let pump = NativeWritePump::new();
        let target = Rc::downgrade(&terminal);
        let pending = Cell::new(false);
        let queue = WriteBuffer::new(pump.scheduler(), move |_, data, result| {
            let terminal = target
                .upgrade()
                .ok_or_else(|| WriteError::Action("native terminal was dropped".into()))?;
            // The source queue passes true even for a first call. Only a prior
            // suspended parser result makes that argument a real continuation.
            let resume = if pending.replace(false) {
                Some(result.unwrap_or(true))
            } else {
                None
            };
            let action = terminal.parse_write_data(data, resume)?;
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
        Ok(Self {
            terminal,
            queue,
            pump,
            parsed,
            input,
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
    pub fn dispose(&self) -> Result<(), HeadlessError> {
        self.queue.dispose();
        self.parsed.dispose();
        self.input.dispose();
        self.terminal.dispose()
    }
}
impl Drop for QueuedTerminal {
    fn drop(&mut self) {
        self.queue.dispose();
        self.parsed.dispose();
        self.input.dispose();
    }
}
