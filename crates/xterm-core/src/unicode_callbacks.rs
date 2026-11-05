// Copyright (c) 2019, 2021 The xterm.js authors. All rights reserved. MIT license.
//! Fallible host providers retain callback failures without unwinding through FFI.

use crate::{
    event::EmitError,
    unicode::{UnicodeCharProperties, UnicodeCharWidth, UnicodeVersionProvider},
};
use std::rc::Rc;

/// Host callbacks may fail or reenter the Unicode service. No service borrow is
/// held while invoking these methods. Native providers use the infallible trait.
pub trait UnicodeProviderCallbacks {
    fn version(&self) -> Result<String, EmitError>;
    fn wcwidth(&self, codepoint: u32) -> Result<UnicodeCharWidth, EmitError>;
    /// Atlas numeric glyphs can carry the signed combined-cell marker. Native
    /// providers treat negative values as controls; JS hosts override this to
    /// deliver the actual Number to their callback.
    fn wcwidth_i32(&self, codepoint: i32) -> Result<UnicodeCharWidth, EmitError> {
        match u32::try_from(codepoint) {
            Ok(codepoint) => self.wcwidth(codepoint),
            Err(_) => Ok(UnicodeCharWidth::Zero),
        }
    }
    fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> Result<UnicodeCharProperties, EmitError>;
}

pub(crate) enum Provider {
    Native(Rc<dyn UnicodeVersionProvider>),
    Host(Rc<dyn UnicodeProviderCallbacks>),
}

impl Provider {
    pub fn wcwidth(&self, codepoint: u32) -> Result<UnicodeCharWidth, EmitError> {
        match self {
            Self::Native(provider) => Ok(provider.wcwidth(codepoint)),
            Self::Host(provider) => provider.wcwidth(codepoint),
        }
    }

    pub fn wcwidth_i32(&self, codepoint: i32) -> Result<UnicodeCharWidth, EmitError> {
        match self {
            Self::Native(provider) => Ok(provider.wcwidth_i32(codepoint)),
            Self::Host(provider) => provider.wcwidth_i32(codepoint),
        }
    }

    pub fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> Result<UnicodeCharProperties, EmitError> {
        match self {
            Self::Native(provider) => Ok(provider.char_properties(codepoint, preceding)),
            Self::Host(provider) => provider.char_properties(codepoint, preceding),
        }
    }
}
