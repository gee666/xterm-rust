//! Retained attribute objects follow in-place SGR changes and freeze on reset.
use super::{TerminalError, TerminalState};
use crate::buffer::AttributeData;
use std::{cell::RefCell, rc::Rc};

/// A source attribute-object identity, not a copy refreshed by the browser.
/// Reset freezes the replaced object's last value before installing a new one.
#[derive(Clone)]
pub struct AttributeView {
    state: Rc<RefCell<TerminalState>>,
    identity: Rc<RefCell<Option<AttributeData>>>,
}
impl AttributeView {
    pub fn capture(state: Rc<RefCell<TerminalState>>) -> Result<Self, TerminalError> {
        let identity = Rc::clone(
            &state
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .attributes_identity,
        );
        Ok(Self { state, identity })
    }
    pub fn same_object(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.identity, &other.identity)
    }
    pub fn read(&self) -> Result<AttributeData, TerminalError> {
        if let Some(value) = self
            .identity
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .as_ref()
        {
            return Ok(value.clone());
        }
        Ok(self
            .state
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .attributes
            .clone())
    }
}
