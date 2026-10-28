// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Replaceable parser properties released before callouts.
use super::{HandlerHandle, HandlerList};
use crate::parser::handler::DispatchError;
use std::{cell::RefCell, rc::Rc};

/// The source parser reads its active-array property again at each lookup.
/// Keep that property distinct from the selected, shared registration array.
pub struct ActiveSlot<H: ?Sized> {
    list: RefCell<HandlerList<H>>,
}
impl<H: ?Sized> Default for ActiveSlot<H> {
    fn default() -> Self {
        Self {
            list: RefCell::default(),
        }
    }
}
impl<H: ?Sized> ActiveSlot<H> {
    pub fn set(&self, list: HandlerList<H>) {
        drop(self.list.replace(list));
    }
    pub fn clear(&self) {
        self.set(Rc::default());
    }
    pub fn current(&self) -> HandlerList<H> {
        self.list.borrow().clone()
    }
    pub fn len(&self) -> usize {
        let list = self.current();
        let length = list.borrow().len();
        length
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn at(&self, index: usize) -> Result<HandlerHandle<H>, DispatchError> {
        let list = self.current();
        let handler = list.borrow().get(index).cloned();
        handler.ok_or(DispatchError::ActiveHandlerRemoved)
    }
}

pub type CallbackHandle<F> = Rc<RefCell<Box<F>>>;
/// Clone the executing callback before invoking it. Replacing or clearing this
/// property then cannot borrow the container across a reentrant disposal.
pub struct CallbackSlot<F: ?Sized> {
    callback: RefCell<Option<CallbackHandle<F>>>,
}
impl<F: ?Sized> Default for CallbackSlot<F> {
    fn default() -> Self {
        Self {
            callback: RefCell::new(None),
        }
    }
}
impl<F: ?Sized> CallbackSlot<F> {
    pub fn set(&self, callback: Box<F>) {
        drop(self.callback.replace(Some(Rc::new(RefCell::new(callback)))));
    }
    pub fn clear(&self) {
        drop(self.callback.replace(None));
    }
    pub fn get(&self) -> Option<CallbackHandle<F>> {
        self.callback.borrow().clone()
    }
}
