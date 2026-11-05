// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Operators used by scroll state. Raw browser values must not be converted to
//! numbers until the corresponding source operator performs that conversion.
use std::convert::Infallible;

/// Each operation has JavaScript evaluation order and may invoke user coercion.
/// Implementations must preserve object identity when cloning a value.
pub trait ScrollValue: Clone {
    type Error;
    fn number(value: f64) -> Self;
    fn int32(&self) -> Result<Self, Self::Error>;
    fn add(&self, other: &Self) -> Result<Self, Self::Error>;
    fn subtract(&self, other: &Self) -> Result<Self, Self::Error>;
    fn less(&self, other: &Self) -> Result<bool, Self::Error>;
    fn greater(&self, other: &Self) -> Result<bool, Self::Error>;
    fn strict_eq(&self, other: &Self) -> bool;
}

impl ScrollValue for f64 {
    type Error = Infallible;
    fn number(value: f64) -> Self {
        value
    }
    fn int32(&self) -> Result<Self, Infallible> {
        const MODULUS: f64 = 4_294_967_296.0;
        const SIGN_BIT: f64 = 2_147_483_648.0;
        if !self.is_finite() || *self == 0.0 {
            return Ok(0.0);
        }
        let unsigned = self.trunc().rem_euclid(MODULUS);
        Ok(if unsigned == 0.0 {
            0.0
        } else if unsigned >= SIGN_BIT {
            unsigned - MODULUS
        } else {
            unsigned
        })
    }
    fn add(&self, other: &Self) -> Result<Self, Infallible> {
        Ok(self + other)
    }
    fn subtract(&self, other: &Self) -> Result<Self, Infallible> {
        Ok(self - other)
    }
    fn less(&self, other: &Self) -> Result<bool, Infallible> {
        Ok(self < other)
    }
    fn greater(&self, other: &Self) -> Result<bool, Infallible> {
        Ok(self > other)
    }
    fn strict_eq(&self, other: &Self) -> bool {
        self == other
    }
}

pub(super) fn infallible<T>(result: Result<T, Infallible>) -> T {
    match result {
        Ok(value) => value,
        Err(never) => match never {},
    }
}
