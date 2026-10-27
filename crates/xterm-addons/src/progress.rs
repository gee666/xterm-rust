// Copyright (c) 2024 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-progress/ProgressAddon.ts.

//! OSC 9;4 progress reporting. Public numeric fields deliberately use f64:
//! upstream's API preserves fractional values and NaN, unlike protocol states.

use crate::status_terminal::StatusTerminal;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::event::{EmitError, Emitter, Event};
use xterm_core::lifecycle::{Disposable, DisposableHandle};

const REMOVE: f64 = 0.0;
const SET: f64 = 1.0;
const ERROR: f64 = 2.0;
const INDETERMINATE: f64 = 3.0;
const PAUSE: f64 = 4.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ProgressState {
    pub state: f64,
    pub value: f64,
}

#[derive(Default)]
struct Inner {
    progress: Cell<ProgressState>,
    change: RefCell<Option<Rc<Emitter<ProgressState>>>>,
}

impl Inner {
    fn set(&self, mut progress: ProgressState) -> Result<bool, EmitError> {
        // A NaN state is accepted upstream, since both comparisons are false.
        if progress.state < REMOVE || progress.state > PAUSE {
            return Ok(false);
        }
        progress.value = if progress.value.is_nan() {
            f64::NAN
        } else if progress.value <= 0.0 {
            // Math.max(-0, +0) chooses positive zero in JavaScript.
            0.0
        } else {
            progress.value.min(100.0)
        };
        self.progress.set(progress);
        let change = self.change.borrow().clone();
        if let Some(change) = change {
            change.fire(&progress)?;
        }
        Ok(true)
    }

    fn osc(&self, data: &[u16]) -> Result<bool, EmitError> {
        if !data.starts_with(&[u16::from(b'4'), u16::from(b';')]) {
            return Ok(false);
        }
        let parts: Vec<_> = data.split(|c| *c == u16::from(b';')).collect();
        if parts.len() > 3 {
            return Ok(true);
        }
        let state = to_int(parts[1]);
        let value = to_int(parts.get(2).copied().unwrap_or_default());
        let previous = self.progress.get().value;
        let value = if state == REMOVE {
            0.0
        } else if state == INDETERMINATE {
            previous
        } else if state == SET || state == ERROR || state == PAUSE {
            if value < 0.0 {
                return Ok(true);
            }
            if state != SET && value == 0.0 {
                previous
            } else {
                value
            }
        } else {
            return Ok(true);
        };
        self.set(ProgressState { state, value })?;
        Ok(true)
    }
}

fn to_int(s: &[u16]) -> f64 {
    let mut value = 0.0;
    for &unit in s {
        if !(u16::from(b'0')..=u16::from(b'9')).contains(&unit) {
            return -1.0;
        }
        // Accumulate in binary64 instead of parsing a Rust integer. This keeps
        // JS rounding and overflow to infinity for arbitrarily long values.
        value = value * 10.0 + f64::from(unit - u16::from(b'0'));
    }
    value
}

#[derive(Default)]
pub struct ProgressAddon {
    inner: Rc<Inner>,
    registration: RefCell<Option<DisposableHandle>>,
}

impl ProgressAddon {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn progress(&self) -> ProgressState {
        self.inner.progress.get()
    }
    /// Returns false for an out-of-range state, which upstream reports as a warning.
    pub fn set_progress(&self, progress: ProgressState) -> Result<bool, EmitError> {
        self.inner.set(progress)
    }
    /// Like upstream, the event exists only after activation.
    pub fn on_change(&self) -> Option<Event<ProgressState>> {
        self.inner
            .change
            .borrow()
            .as_ref()
            .map(|emitter| emitter.event())
    }

    pub fn activate(&self, terminal: Rc<dyn StatusTerminal>) {
        let weak = Rc::downgrade(&self.inner);
        let registration = terminal.register_osc_handler(
            9,
            Rc::new(move |data| weak.upgrade().map_or(Ok(false), |inner| inner.osc(data))),
        );
        *self.registration.borrow_mut() = Some(registration);
        *self.inner.change.borrow_mut() = Some(Rc::new(Emitter::new()));
    }
}

impl Disposable for ProgressAddon {
    fn dispose(&self) {
        let registration = self.registration.borrow().clone();
        if let Some(registration) = registration {
            registration.dispose();
        }
        let emitter = self.inner.change.borrow().clone();
        if let Some(emitter) = emitter {
            emitter.dispose();
        }
    }
}

impl Drop for ProgressAddon {
    fn drop(&mut self) {
        self.dispose();
    }
}
