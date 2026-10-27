// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/services/CharSizeService.ts.
use crate::api::Size;
use std::cell::Cell;
use xterm_core::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
};

/// A hidden or disconnected measuring element must not erase a valid size.
#[derive(Default)]
pub struct CharSizeState {
    width: Cell<f64>,
    height: Cell<f64>,
    changes: Emitter<()>,
}
impl CharSizeState {
    pub fn size(&self) -> Size {
        Size {
            width: self.width.get(),
            height: self.height.get(),
        }
    }
    pub fn has_valid_size(&self) -> bool {
        self.width.get() > 0.0 && self.height.get() > 0.0
    }
    pub fn on_change(&self) -> Event<()> {
        self.changes.event()
    }
    pub fn update_measurement(
        &self,
        width: Option<f64>,
        height: Option<f64>,
    ) -> Result<(), EmitError> {
        if let (Some(width), Some(height)) = (width, height) {
            if width > 0.0
                && height > 0.0
                && (self.width.get() != width || self.height.get() != height)
            {
                self.width.set(width);
                self.height.set(height);
                self.changes.fire(&())?;
            }
        }
        Ok(())
    }
}
impl Disposable for CharSizeState {
    fn dispose(&self) {
        self.changes.dispose();
    }
}
impl Drop for CharSizeState {
    fn drop(&mut self) {
        self.dispose();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};
    #[test]
    fn invalid_measurements_retain_last_size_and_changes_commit_before_event() {
        let state = Rc::new(CharSizeState::default());
        let weak = Rc::downgrade(&state);
        let log = Rc::new(RefCell::new(Vec::new()));
        let record = Rc::clone(&log);
        let subscription = state
            .on_change()
            .subscribe(move |_| record.borrow_mut().push(weak.upgrade().unwrap().size()));
        assert!(!state.has_valid_size());
        state.update_measurement(Some(10.0), Some(20.0)).unwrap();
        for (width, height) in [
            (None, Some(5.0)),
            (Some(5.0), None),
            (Some(0.0), Some(2.0)),
            (Some(1.0), Some(-1.0)),
            (Some(f64::NAN), Some(3.0)),
        ] {
            state.update_measurement(width, height).unwrap();
        }
        state.update_measurement(Some(10.0), Some(20.0)).unwrap();
        assert_eq!(
            state.size(),
            Size {
                width: 10.0,
                height: 20.0
            }
        );
        assert_eq!(log.borrow().len(), 1);
        state.update_measurement(Some(11.0), Some(20.0)).unwrap();
        assert_eq!(
            log.borrow().as_slice(),
            [
                Size {
                    width: 10.0,
                    height: 20.0
                },
                Size {
                    width: 11.0,
                    height: 20.0
                }
            ]
        );
        assert!(state.has_valid_size());
        subscription.dispose();
    }
}
