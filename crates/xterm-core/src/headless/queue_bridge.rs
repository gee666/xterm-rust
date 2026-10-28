use super::{HeadlessError, Terminal};
use crate::{
    input::{
        parse_source::ParseInputSource,
        write_buffer::{WriteAction, WriteData, WriteError},
    },
    terminal::{TerminalError, TerminalEvent},
};
use std::rc::Rc;

impl Terminal {
    /// Retains a live source across a parser pause. The caller still owns future
    /// driving through take_pending/resume; this does not consume that future.
    pub fn write_source(&self, source: Rc<dyn ParseInputSource>) -> Result<(), HeadlessError> {
        if self.is_disposed()? {
            return Ok(());
        }
        self.operate(|engine| engine.write_source(source))
    }
    pub fn take_cleanup(
        &self,
    ) -> Result<Vec<crate::parser::handler::HandlerResult>, HeadlessError> {
        Ok(self
            .engine
            .try_borrow_mut()
            .map_err(|_| HeadlessError::Borrowed)?
            .take_cleanup())
    }
    /// Borrowed compatibility entrypoint. Native vectors snapshot once; external
    /// byte views keep their shared allocation through the optional source trait.
    /// A continuation never inspects or converts the supplied input again.
    pub fn parse_write_data(
        &self,
        data: &WriteData,
        resume: Option<bool>,
    ) -> Result<WriteAction, WriteError> {
        if let Some(result) = resume {
            return self.resume_queued(result);
        }
        self.parse_write_source(Rc::new(data.clone()).into_parse_source()?, None)
    }
    /// Owned queue entrypoint. A live source is read only as the core requests
    /// bounded chunks, including after an async handler mutates a remaining tail.
    pub fn parse_write_source(
        &self,
        source: Rc<dyn ParseInputSource>,
        resume: Option<bool>,
    ) -> Result<WriteAction, WriteError> {
        if let Some(result) = resume {
            return self.resume_queued(result);
        }
        let result = self
            .events
            .with_queued_parse(|| self.operate(|engine| engine.write_source(source)));
        self.queue_result(result)
    }
    fn resume_queued(&self, result: bool) -> Result<WriteAction, WriteError> {
        let result = self.events.with_queued_parse(|| self.resume(result));
        self.queue_result(result)
    }
    fn queue_result(&self, result: Result<(), HeadlessError>) -> Result<WriteAction, WriteError> {
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
