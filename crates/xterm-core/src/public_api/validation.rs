// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from headless/public/Terminal.ts
// and browser/public/Terminal.ts.

//! Shared validation at the JavaScript Number boundary, before index conversion.
//! Integer validation does not impose an i32, usize or safe-integer restriction:
//! those would reject values accepted by the source public API.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    IntegerRequired,
    PositiveIntegerRequired,
    ConstructorOnlyOption(String),
    ProposedApiDisabled,
    NotOpened,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntegerRequired => f.write_str("This API only accepts integers"),
            Self::PositiveIntegerRequired => f.write_str("This API only accepts positive integers"),
            Self::ConstructorOnlyOption(key) => {
                write!(f, "Option \"{key}\" can only be set in the constructor")
            }
            Self::ProposedApiDisabled => {
                f.write_str("You must set the allowProposedApi option to true to use proposed API")
            }
            Self::NotOpened => f.write_str("Terminal must be opened first"),
        }
    }
}
impl std::error::Error for ApiError {}

pub fn verify_integers(values: &[f64]) -> Result<(), ApiError> {
    if values
        .iter()
        .any(|value| !value.is_finite() || value.fract() != 0.0)
    {
        Err(ApiError::IntegerRequired)
    } else {
        Ok(())
    }
}

/// Unlike integer validation, the browser's truthiness guard admits NaN.
/// Zero and negative zero are also accepted despite the method's name.
pub fn verify_positive_integers(values: &[f64]) -> Result<(), ApiError> {
    if values.iter().any(|value| {
        *value != 0.0
            && !value.is_nan()
            && (!value.is_finite() || value.fract() != 0.0 || *value < 0.0)
    }) {
        Err(ApiError::PositiveIntegerRequired)
    } else {
        Ok(())
    }
}

pub fn verify_mutable_option(key: &str) -> Result<(), ApiError> {
    if matches!(key, "cols" | "rows") {
        Err(ApiError::ConstructorOnlyOption(key.into()))
    } else {
        Ok(())
    }
}

pub fn verify_proposed_api(allow_proposed_api: bool) -> Result<(), ApiError> {
    if allow_proposed_api {
        Ok(())
    } else {
        Err(ApiError::ProposedApiDisabled)
    }
}
