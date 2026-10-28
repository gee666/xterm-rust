// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from headless/public/Terminal.ts.

use crate::options::{OptionError, OptionValue, OptionsService};
use crate::public_api::validation::{verify_mutable_option, verify_proposed_api, ApiError};
use std::{fmt, rc::Rc};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicOptionError {
    Api(ApiError),
    Option(OptionError),
}
impl fmt::Display for PublicOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::Option(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for PublicOptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Api(error) => Some(error),
            Self::Option(error) => Some(error),
        }
    }
}
impl From<ApiError> for PublicOptionError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}
impl From<OptionError> for PublicOptionError {
    fn from(error: OptionError) -> Self {
        Self::Option(error)
    }
}

/// Live public option view, sharing the engine's OptionsService rather than a
/// detached snapshot. Clones preserve the same service and change listeners.
/// Unlike arbitrary JavaScript property creation, unknown native keys return
/// OptionError::UnknownKey and cannot become detached expando properties.
#[derive(Clone)]
pub struct PublicOptions {
    service: Rc<OptionsService>,
}
impl PublicOptions {
    pub fn new(service: Rc<OptionsService>) -> Self {
        Self { service }
    }
    pub fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        self.service.get(key)
    }
    pub fn service(&self) -> Rc<OptionsService> {
        Rc::clone(&self.service)
    }
    pub fn keys(&self) -> Vec<&'static str> {
        self.service.keys()
    }

    /// Readonly checks happen even for an assignment of the current value.
    pub fn set(&self, key: &str, value: OptionValue) -> Result<(), PublicOptionError> {
        verify_mutable_option(key)?;
        self.service.set(key, value)?;
        Ok(())
    }

    /// Source assignment is sequential, not transactional. Earlier changes and
    /// notifications remain visible if a later property rejects the assignment.
    pub fn assign<K: AsRef<str>>(
        &self,
        options: impl IntoIterator<Item = (K, OptionValue)>,
    ) -> Result<(), PublicOptionError> {
        for (key, value) in options {
            self.set(key.as_ref(), value)?;
        }
        Ok(())
    }

    pub fn check_proposed_api(&self) -> Result<(), PublicOptionError> {
        verify_proposed_api(self.service.get("allowProposedApi")?.is_truthy())?;
        Ok(())
    }
}
