use super::PublicOptionError;
use crate::{
    event::EmitError,
    public_api::{addon_manager::AddonError, validation::ApiError},
    terminal::TerminalError,
    unicode_service::UnicodeServiceError,
};
use std::fmt;

#[derive(Debug)]
pub enum HeadlessError {
    Api(ApiError),
    Options(PublicOptionError),
    Engine(TerminalError),
    Dispatch(EmitError),
    Unicode(UnicodeServiceError),
    Addon(Box<AddonError<HeadlessError>>),
    Borrowed,
    HostPayloadRequiresConversion,
}
impl fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Options(error) => error.fmt(f),
            Self::Engine(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
            Self::Unicode(error) => error.fmt(f),
            Self::Addon(error) => error.fmt(f),
            Self::Borrowed => f.write_str("terminal is already borrowed"),
            Self::HostPayloadRequiresConversion => {
                f.write_str("host payload must be converted by its adapter before native parsing")
            }
        }
    }
}
impl std::error::Error for HeadlessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Api(error) => Some(error),
            Self::Options(error) => Some(error),
            Self::Engine(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Unicode(error) => Some(error),
            Self::Addon(error) => Some(error.as_ref()),
            Self::Borrowed | Self::HostPayloadRequiresConversion => None,
        }
    }
}
impl From<ApiError> for HeadlessError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}
impl From<PublicOptionError> for HeadlessError {
    fn from(error: PublicOptionError) -> Self {
        Self::Options(error)
    }
}
impl From<TerminalError> for HeadlessError {
    fn from(error: TerminalError) -> Self {
        Self::Engine(error)
    }
}
impl From<EmitError> for HeadlessError {
    fn from(error: EmitError) -> Self {
        Self::Dispatch(error)
    }
}
impl From<UnicodeServiceError> for HeadlessError {
    fn from(error: UnicodeServiceError) -> Self {
        Self::Unicode(error)
    }
}
impl From<AddonError<HeadlessError>> for HeadlessError {
    fn from(error: AddonError<HeadlessError>) -> Self {
        Self::Addon(Box::new(error))
    }
}
