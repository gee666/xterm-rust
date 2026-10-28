// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
// Port of browser/scrollable/scrollbarVisibilityController.ts.

use super::ScrollbarVisibility;
use crate::links::time_debounce::TimerScheduler;
use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

type ClassSetter<S> = Rc<dyn Fn(&str) -> Result<(), <S as TimerScheduler>::Error>>;

#[derive(Debug, PartialEq, Eq)]
pub enum VisibilityError<E> {
    Host(E),
    DisposedTimer,
}
impl<E: fmt::Display> fmt::Display for VisibilityError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(error) => error.fmt(f),
            Self::DisposedTimer => f.write_str("Calling setIfNotSet on a disposed TimeoutTimer"),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for VisibilityError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Host(error) => Some(error),
            Self::DisposedTimer => None,
        }
    }
}

struct State<S: TimerScheduler> {
    scheduler: RefCell<S>,
    visibility: RefCell<ScrollbarVisibility>,
    visible_class: String,
    invisible_class: String,
    target: RefCell<Option<ClassSetter<S>>>,
    raw_should_be_visible: Cell<bool>,
    should_be_visible: Cell<bool>,
    needed: Cell<bool>,
    visible: Cell<bool>,
    pending: RefCell<Option<S::Handle>>,
    timer_disposed: Cell<bool>,
}
impl<S: TimerScheduler> Drop for State<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.pending.get_mut().take() {
            let scheduler = self.scheduler.get_mut();
            if let Err(error) = scheduler.cancel_timeout(handle) {
                scheduler.cleanup_error(error);
            }
        }
    }
}

/// Visibility policy separate from DOM ownership. Scheduler callbacks execute on
/// a later turn, as setTimeout does; class setters may reenter synchronously.
/// Bind the host timeout to weak_dispatcher rather than capturing this owner.
pub struct VisibilityController<S: TimerScheduler> {
    state: Rc<State<S>>,
}
impl<S: TimerScheduler> Clone for VisibilityController<S> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}

impl<S: TimerScheduler> VisibilityController<S> {
    pub fn new(
        scheduler: S,
        visibility: ScrollbarVisibility,
        visible_class: String,
        invisible_class: String,
    ) -> Self {
        Self {
            state: Rc::new(State {
                scheduler: RefCell::new(scheduler),
                visibility: RefCell::new(visibility),
                visible_class,
                invisible_class,
                target: RefCell::new(None),
                raw_should_be_visible: Cell::new(false),
                should_be_visible: Cell::new(false),
                needed: Cell::new(false),
                visible: Cell::new(false),
                pending: RefCell::new(None),
                timer_disposed: Cell::new(false),
            }),
        }
    }

    pub fn set_visibility(
        &self,
        visibility: ScrollbarVisibility,
    ) -> Result<(), VisibilityError<S::Error>> {
        let changed = !self.state.visibility.borrow().strict_eq(&visibility);
        if changed {
            let previous = self.state.visibility.replace(visibility);
            drop(previous);
            self.update_should_be_visible()?;
        }
        Ok(())
    }

    pub fn set_should_be_visible(&self, raw: bool) -> Result<(), VisibilityError<S::Error>> {
        self.state.raw_should_be_visible.set(raw);
        self.update_should_be_visible()
    }

    pub fn set_is_needed(&self, needed: bool) -> Result<(), VisibilityError<S::Error>> {
        if self.state.needed.replace(needed) != needed {
            self.ensure_visibility()?;
        }
        Ok(())
    }

    pub fn set_dom_node(
        &self,
        setter: impl Fn(&str) -> Result<(), S::Error> + 'static,
    ) -> Result<(), VisibilityError<S::Error>> {
        let setter: ClassSetter<S> = Rc::new(setter);
        let previous = self.state.target.replace(Some(Rc::clone(&setter)));
        drop(previous);
        setter(&self.state.invisible_class).map_err(VisibilityError::Host)?;
        self.set_should_be_visible(false)
    }

    pub fn ensure_visibility(&self) -> Result<(), VisibilityError<S::Error>> {
        if !self.state.needed.get() {
            return self.hide(false);
        }
        if self.state.should_be_visible.get() {
            self.reveal()
        } else {
            self.hide(true)
        }
    }

    pub fn is_visible(&self) -> bool {
        self.state.visible.get()
    }
    pub fn is_needed(&self) -> bool {
        self.state.needed.get()
    }
    pub fn has_pending_reveal(&self) -> bool {
        self.state.pending.borrow().is_some()
    }
    pub fn timer_is_disposed(&self) -> bool {
        self.state.timer_disposed.get()
    }

    /// TimeoutTimer clears its token before invoking the class update callback.
    pub fn on_timeout(&self) -> Result<(), VisibilityError<S::Error>> {
        self.state.pending.borrow_mut().take();
        self.set_class(&self.state.visible_class)
    }

    pub fn weak_dispatcher(&self) -> impl Fn() -> Result<(), VisibilityError<S::Error>> + 'static
    where
        S: 'static,
    {
        let state = Rc::downgrade(&self.state);
        move || {
            if let Some(state) = state.upgrade() {
                Self { state }.on_timeout()?;
            }
            Ok(())
        }
    }

    pub fn dispose(&self) -> Result<(), VisibilityError<S::Error>> {
        self.cancel_pending()?;
        self.state.timer_disposed.set(true);
        Ok(())
    }

    fn update_should_be_visible(&self) -> Result<(), VisibilityError<S::Error>> {
        let visibility = self.state.visibility.borrow().clone();
        let should = if visibility.is_hidden() {
            false
        } else if visibility.is_visible() {
            true
        } else {
            self.state.raw_should_be_visible.get()
        };
        if self.state.should_be_visible.replace(should) != should {
            self.ensure_visibility()?;
        }
        Ok(())
    }

    fn reveal(&self) -> Result<(), VisibilityError<S::Error>> {
        if self.state.visible.replace(true) {
            return Ok(());
        }
        if self.state.timer_disposed.get() {
            return Err(VisibilityError::DisposedTimer);
        }
        if self.state.pending.borrow().is_none() {
            let handle = self
                .state
                .scheduler
                .borrow_mut()
                .request_timeout(0.0)
                .map_err(VisibilityError::Host)?;
            *self.state.pending.borrow_mut() = Some(handle);
        }
        Ok(())
    }

    fn hide(&self, fade: bool) -> Result<(), VisibilityError<S::Error>> {
        self.cancel_pending()?;
        if !self.state.visible.replace(false) {
            return Ok(());
        }
        if fade {
            self.set_class(&format!("{} xterm-fade", self.state.invisible_class))
        } else {
            self.set_class(&self.state.invisible_class)
        }
    }

    fn cancel_pending(&self) -> Result<(), VisibilityError<S::Error>> {
        let handle = self.state.pending.borrow().clone();
        if let Some(handle) = handle {
            self.state
                .scheduler
                .borrow_mut()
                .cancel_timeout(handle)
                .map_err(VisibilityError::Host)?;
            self.state.pending.borrow_mut().take();
        }
        Ok(())
    }

    fn set_class(&self, name: &str) -> Result<(), VisibilityError<S::Error>> {
        let target = self.state.target.borrow().clone();
        if let Some(target) = target {
            target(name).map_err(VisibilityError::Host)?;
        }
        Ok(())
    }
}
