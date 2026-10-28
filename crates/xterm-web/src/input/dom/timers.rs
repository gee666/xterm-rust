//! Browser task-turn scheduling with owned closures and deterministic cleanup.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;

struct Timer {
    id: i32,
    completed: Rc<Cell<bool>>,
    // The closure must outlive the browser's registration, until it fires or
    // clearTimeout removes that registration.
    _callback: Closure<dyn FnMut() -> Result<(), JsValue>>,
}

pub(super) struct TimerQueue {
    window: Window,
    timers: RefCell<Vec<Timer>>,
}

impl TimerQueue {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            timers: RefCell::new(Vec::new()),
        }
    }

    pub fn schedule(
        &self,
        callback: impl FnOnce() -> Result<(), JsValue> + 'static,
    ) -> Result<(), JsValue> {
        // Do not remove a completed closure from inside its own callback.
        // Pruning on the next registration bounds retained callbacks without
        // Closure::forget or self-owning reference cycles.
        self.timers
            .borrow_mut()
            .retain(|timer| !timer.completed.get());
        let completed = Rc::new(Cell::new(false));
        let completed_callback = completed.clone();
        let mut callback = Some(callback);
        let closure = Closure::wrap(Box::new(move || {
            let result = callback.take().map_or(Ok(()), |callback| callback());
            completed_callback.set(true);
            result
        }) as Box<dyn FnMut() -> Result<(), JsValue>>);
        let id = self
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                0,
            )?;
        self.timers.borrow_mut().push(Timer {
            id,
            completed,
            _callback: closure,
        });
        Ok(())
    }

    pub fn clear(&self) {
        for timer in self.timers.borrow_mut().drain(..) {
            self.window.clear_timeout_with_handle(timer.id);
        }
    }
}

impl Drop for TimerQueue {
    fn drop(&mut self) {
        self.clear();
    }
}
