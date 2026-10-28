//! Retained mutable byte fixture shared with the real terminal driver.
use std::{any::Any, borrow::Cow, cell::RefCell, ops::Range, rc::Rc};
use xterm_core::{
    input::{
        parse_source::{ParseChunk, ParseInputSource, ParseSourceError},
        write_buffer::WritePayload,
    },
    log_service::LogValue,
};
#[derive(Debug)]
pub struct MutableView(pub RefCell<Vec<u8>>);
impl WritePayload for MutableView {
    fn len(&self) -> Result<usize, xterm_core::input::write_buffer::WriteError> {
        Ok(self.0.borrow().len())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn into_parse_source(self: Rc<Self>) -> Option<Rc<dyn ParseInputSource>> {
        Some(self)
    }
}
impl ParseInputSource for MutableView {
    fn len(&self) -> Result<usize, ParseSourceError> {
        Ok(self.0.borrow().len())
    }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        let bytes = self.0.borrow();
        let start = range.start.min(bytes.len());
        let end = range.end.min(bytes.len()).max(start);
        Ok(ParseChunk::Bytes(Cow::Owned(bytes[start..end].to_vec())))
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        Ok(ParseChunk::Bytes(Cow::Owned(self.0.borrow().clone())))
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        Ok(self.0.borrow().iter().copied().map(u16::from).collect())
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        Ok(LogValue::Bytes(self.0.borrow().clone()))
    }
}
