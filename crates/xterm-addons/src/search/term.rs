// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::SearchAddonError;
use std::{any::Any, borrow::Cow, rc::Rc};

/// Retains term identity separately from its matching text. Browser length
/// access can throw or reenter, so callers must not hold addon/buffer borrows
/// while validating a term. Invalid terms are cached without coercing to text.
pub trait SearchTermSource {
    fn as_any(&self) -> &dyn Any;
    fn is_valid(&self) -> Result<bool, SearchAddonError>;
    fn is_truthy(&self) -> bool;
    /// Undefined is the source cache's absence sentinel, unlike other invalid terms.
    fn is_undefined(&self) -> bool {
        false
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool;
    /// Matching text for a validated term. Invalid terms must not reach matching.
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError>;
}
pub type SearchTermHandle = Rc<dyn SearchTermSource>;

/// Native text and the cache's absence sentinel have different identities even
/// though neither absence nor an empty string is a valid matching term.
pub(super) enum NativeTerm {
    Text(Vec<u16>),
    Undefined,
}
impl SearchTermSource for NativeTerm {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        Ok(self.is_truthy())
    }
    fn is_truthy(&self) -> bool {
        matches!(self, Self::Text(text) if !text.is_empty())
    }
    fn is_undefined(&self) -> bool {
        matches!(self, Self::Undefined)
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool {
        match self {
            Self::Undefined => other.is_undefined(),
            Self::Text(text) => other
                .as_any()
                .downcast_ref::<Self>()
                .is_some_and(|other| matches!(other, Self::Text(other) if text == other)),
        }
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        Ok(Cow::Borrowed(match self {
            Self::Text(text) => text,
            Self::Undefined => &[],
        }))
    }
}
