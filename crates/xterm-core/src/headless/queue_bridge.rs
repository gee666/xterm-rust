use super::{HeadlessError, Terminal};
use crate::{
    input::write_buffer::{WriteAction, WriteData, WriteError},
    terminal::{TerminalError, TerminalEvent},
};

impl Terminal {
    pub fn take_cleanup(
        &self,
    ) -> Result<Vec<crate::parser::handler::HandlerResult>, HeadlessError> {
        Ok(self
            .engine
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?
            .take_cleanup())
    }
    /// Parse once or resume the same decoded chunk. Queue hosts own completion
    /// callbacks and batched writeParsed events, rather than individual parses.
    pub fn parse_write_data(
        &self,
        data: &WriteData,
        resume: Option<bool>,
    ) -> Result<WriteAction, WriteError> {
        let result = self.events.with_queued_parse(|| match resume {
            Some(result) => self.resume(result),
            None => match data {
                WriteData::Text(text) => self.write_utf16(text),
                WriteData::Bytes(bytes) => self.write_bytes(bytes),
                WriteData::External(_) => Err(HeadlessError::HostPayloadRequiresConversion),
            },
        });
        match result {
            Ok(()) => Ok(WriteAction::Complete),
            Err(HeadlessError::Engine(TerminalError::Pending)) => {
                let pending = self
                    .take_pending()
                    .map_err(WriteError::external)?
                    .ok_or_else(|| {
                        WriteError::Action("pending parser did not provide a continuation".into())
                    })?;
                Ok(WriteAction::Pending(Box::pin(async move {
                    pending.resolve().await.map_err(WriteError::external)
                })))
            }
            Err(error) => Err(WriteError::external(error)),
        }
    }
    /// Host queue completion hook, called from WriteBuffer.on_write_parsed.
    pub fn notify_write_parsed(&self) -> Result<(), crate::event::EmitError> {
        self.events.dispatch(TerminalEvent::WriteParsed)
    }
}
