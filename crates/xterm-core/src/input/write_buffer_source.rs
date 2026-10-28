// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{WriteData, WriteError};
use crate::{
    input::parse_source::{ParseChunk, ParseInputSource, ParseSourceError},
    log_service::LogValue,
};
use std::{ops::Range, rc::Rc};
impl WriteData {
    /// Native vectors are shared without copying; a host payload converts the
    /// same allocation through its explicit live-source capability.
    pub fn into_parse_source(self: Rc<Self>) -> Result<Rc<dyn ParseInputSource>, WriteError> {
        if let Self::External(payload) = self.as_ref() {
            return Rc::clone(payload).into_parse_source().ok_or_else(|| {
                WriteError::Action("external write payload has no retained parse source".into())
            });
        }
        Ok(self)
    }
    fn native_source(&self) -> Result<&dyn ParseInputSource, ParseSourceError> {
        match self {
            Self::Text(data) => Ok(data),
            Self::Bytes(data) => Ok(data),
            Self::External(_) => Err(ParseSourceError::new(WriteError::Action(
                "convert external input with into_parse_source before parsing".into(),
            ))),
        }
    }
}
/// Direct upcasting is for native variants. Use into_parse_source for External
/// so the host's carrier, typed errors and original view identity are retained.
impl ParseInputSource for WriteData {
    fn len(&self) -> Result<usize, ParseSourceError> {
        self.native_source()?.len()
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        self.native_source()?.read_all()
    }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        self.native_source()?.read_range(range)
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        self.native_source()?.debug_units()
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        self.native_source()?.trace_value()
    }
}
