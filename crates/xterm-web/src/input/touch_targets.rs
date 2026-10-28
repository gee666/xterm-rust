// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Live linked-list iteration matching touch.ts, without reference cycles.
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use web_sys::HtmlElement;

pub(super) struct Target {
    pub element: HtmlElement,
    next: RefCell<Option<Rc<Target>>>,
    previous: RefCell<Weak<Target>>,
}
impl Drop for Target {
    fn drop(&mut self) {
        // Tokens may retain an old next-chain after every registration is
        // removed. Release uniquely owned tails iteratively, not on the stack.
        let mut next = self.next.get_mut().take();
        while let Some(target) = next {
            match Rc::try_unwrap(target) {
                Ok(mut target) => next = target.next.get_mut().take(),
                Err(_) => break,
            }
        }
    }
}
#[derive(Default)]
pub(super) struct Targets {
    first: Option<Rc<Target>>,
    last: Option<Rc<Target>>,
}
impl Targets {
    pub fn push(&mut self, element: HtmlElement) -> Rc<Target> {
        let target = Rc::new(Target {
            element,
            next: RefCell::new(None),
            previous: RefCell::new(Weak::new()),
        });
        if let Some(last) = &self.last {
            *last.next.borrow_mut() = Some(Rc::clone(&target));
            *target.previous.borrow_mut() = Rc::downgrade(last);
        } else {
            self.first = Some(Rc::clone(&target));
        }
        self.last = Some(Rc::clone(&target));
        target
    }
    pub fn remove(&mut self, target: &Rc<Target>) {
        let previous = target.previous.borrow().upgrade();
        let next = target.next.borrow().clone();
        match (previous, next) {
            (Some(previous), Some(next)) => {
                *previous.next.borrow_mut() = Some(Rc::clone(&next));
                *next.previous.borrow_mut() = Rc::downgrade(&previous);
            }
            (None, None) => {
                self.first = None;
                self.last = None;
            }
            (Some(previous), None) => {
                *previous.next.borrow_mut() = None;
                self.last = Some(previous);
            }
            (None, Some(next)) => {
                *next.previous.borrow_mut() = Weak::new();
                self.first = Some(next);
            }
        }
        // Do not clear the removed node's links: source generators paused on
        // that node resume through its old next link after a callback removes it.
    }
    pub fn iter(&self) -> TargetIterator {
        TargetIterator {
            current: self.first.clone(),
            started: false,
        }
    }
}
pub(super) struct TargetIterator {
    current: Option<Rc<Target>>,
    started: bool,
}
impl Iterator for TargetIterator {
    type Item = Rc<Target>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            let next = self.current.as_ref()?.next.borrow().clone();
            self.current = next;
        }
        self.started = true;
        self.current.clone()
    }
}
