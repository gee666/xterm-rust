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
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError>;
}
pub type SearchTermHandle = Rc<dyn SearchTermSource>;

pub(super) struct Utf16Term(pub Vec<u16>);
impl SearchTermSource for Utf16Term {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        Ok(!self.0.is_empty())
    }
    fn is_truthy(&self) -> bool {
        !self.0.is_empty()
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.0 == other.0)
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        Ok(Cow::Borrowed(&self.0))
    }
}
