// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/parser/Params.ts.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const MAX_VALUE: i64 = i32::MAX as i64;
const MAX_SUBPARAMS: usize = 256;

/// An entry in xterm's alternating parameter/subparameter array representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParamValue {
    Param(i32),
    SubParams(Vec<i32>),
}

/// Invalid parameter input or a storage capacity outside the source contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParamsError {
    TooManySubParams,
    ValueBelowDefault,
    InvalidDigit,
    ZeroCapacityZdm,
}

impl fmt::Display for ParamsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooManySubParams => "maxSubParamsLength must not be greater than 256",
            Self::ValueBelowDefault => "values less than -1 are not allowed",
            Self::InvalidDigit => "a parameter digit must be between 0 and 9",
            Self::ZeroCapacityZdm => "zero default mode requires parameter capacity",
        })
    }
}

impl Error for ParamsError {}

/// Bounded sequence parameter storage, including colon-separated subparameters.
///
/// Slices are borrowed until the next mutable operation. Clone the object or use
/// `to_array` to retain a snapshot. Numeric input is integral; values above the
/// positive int32 limit clamp to that limit. See `docs/params.md` for JS mapping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Params {
    params: Vec<i32>,
    length: usize,
    sub_params: Vec<i32>,
    sub_length: usize,
    sub_indices: Vec<u16>,
    reject_digits: bool,
    reject_sub_digits: bool,
    digit_is_sub: bool,
}

impl Default for Params {
    fn default() -> Self {
        Self::with_capacities(32, 32).expect("default subparameter capacity is below 256")
    }
}

impl Params {
    pub fn with_capacities(max_length: usize, max_sub_length: usize) -> Result<Self, ParamsError> {
        if max_sub_length > MAX_SUBPARAMS {
            return Err(ParamsError::TooManySubParams);
        }
        Ok(Self {
            params: vec![0; max_length],
            length: 0,
            sub_params: vec![0; max_sub_length],
            sub_length: 0,
            sub_indices: vec![0; max_length],
            reject_digits: false,
            reject_sub_digits: false,
            digit_is_sub: false,
        })
    }

    pub fn from_array(values: &[ParamValue]) -> Result<Self, ParamsError> {
        let mut params = Self::default();
        // Upstream skips the first entry when it is a subparameter array.
        let skip = usize::from(matches!(values.first(), Some(ParamValue::SubParams(_))));
        for value in &values[skip..] {
            match value {
                ParamValue::Param(value) => params.add_param(i64::from(*value))?,
                ParamValue::SubParams(values) => {
                    for value in values {
                        params.add_sub_param(i64::from(*value))?;
                    }
                }
            }
        }
        Ok(params)
    }

    pub fn values(&self) -> &[i32] {
        &self.params[..self.length]
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn capacity(&self) -> usize {
        self.params.len()
    }

    pub fn sub_capacity(&self) -> usize {
        self.sub_params.len()
    }

    pub fn sub_len(&self) -> usize {
        self.sub_length
    }

    pub fn to_array(&self) -> Vec<ParamValue> {
        let mut result = Vec::new();
        for (index, value) in self.values().iter().enumerate() {
            result.push(ParamValue::Param(*value));
            if let Some(sub) = self.get_sub_params(index) {
                result.push(ParamValue::SubParams(sub.to_vec()));
            }
        }
        result
    }

    pub fn reset(&mut self) {
        self.length = 0;
        self.sub_length = 0;
        self.reject_digits = false;
        self.reject_sub_digits = false;
        self.digit_is_sub = false;
    }

    /// Reset and store a single zero parameter. Zero-capacity storage cannot do so.
    pub fn reset_zdm(&mut self) -> Result<(), ParamsError> {
        if self.params.is_empty() {
            return Err(ParamsError::ZeroCapacityZdm);
        }
        self.reset();
        self.length = 1;
        self.sub_indices[0] = 0;
        self.params[0] = 0;
        Ok(())
    }

    pub fn add_param(&mut self, value: i64) -> Result<(), ParamsError> {
        self.digit_is_sub = false;
        if self.length >= self.params.len() {
            self.reject_digits = true;
            return Ok(());
        }
        let value = checked_value(value)?;
        // The upstream Uint16Array deliberately packs two eight-bit offsets.
        let offset = self.sub_length as u16;
        self.sub_indices[self.length] = (offset << 8) | offset;
        self.params[self.length] = value;
        self.length += 1;
        Ok(())
    }

    pub fn add_sub_param(&mut self, value: i64) -> Result<(), ParamsError> {
        self.digit_is_sub = true;
        if self.length == 0 {
            return Ok(());
        }
        if self.reject_digits || self.sub_length >= self.sub_params.len() {
            self.reject_sub_digits = true;
            return Ok(());
        }
        let value = checked_value(value)?;
        self.sub_params[self.sub_length] = value;
        self.sub_length += 1;
        self.sub_indices[self.length - 1] = self.sub_indices[self.length - 1].wrapping_add(1);
        Ok(())
    }

    pub fn has_sub_params(&self, index: usize) -> bool {
        self.get_sub_params(index).is_some()
    }

    pub fn get_sub_params(&self, index: usize) -> Option<&[i32]> {
        if index >= self.length {
            return None;
        }
        let packed = self.sub_indices[index];
        let start = usize::from(packed >> 8);
        let end = usize::from(packed & 0xff);
        (end > start).then(|| &self.sub_params[start..end])
    }

    pub fn get_sub_params_all(&self) -> BTreeMap<usize, Vec<i32>> {
        (0..self.length)
            .filter_map(|index| self.get_sub_params(index).map(|sub| (index, sub.to_vec())))
            .collect()
    }

    /// Accumulate one ASCII decimal digit in the current parameter or subparameter.
    pub fn add_digit(&mut self, digit: u8) -> Result<(), ParamsError> {
        if digit > 9 {
            return Err(ParamsError::InvalidDigit);
        }
        let length = if self.digit_is_sub {
            self.sub_length
        } else {
            self.length
        };
        if self.reject_digits || length == 0 || (self.digit_is_sub && self.reject_sub_digits) {
            return Ok(());
        }
        let store = if self.digit_is_sub {
            &mut self.sub_params
        } else {
            &mut self.params
        };
        let current = store[length - 1];
        let value = if current == -1 {
            i64::from(digit)
        } else {
            i64::from(current) * 10 + i64::from(digit)
        };
        store[length - 1] = value.min(MAX_VALUE) as i32;
        Ok(())
    }
}

fn checked_value(value: i64) -> Result<i32, ParamsError> {
    if value < -1 {
        Err(ParamsError::ValueBelowDefault)
    } else {
        Ok(value.min(MAX_VALUE) as i32)
    }
}
