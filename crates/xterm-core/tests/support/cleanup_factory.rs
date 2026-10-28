// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    buffer::{buffer_set::CleanupFactory, BufferService},
    options::{OptionValue, OptionsService},
};
pub fn service() -> Rc<BufferService> {
    Rc::new(
        BufferService::new(Rc::new(OptionsService::new(
            false,
            [
                ("cols", OptionValue::Number(4.0)),
                ("rows", OptionValue::Number(3.0)),
                ("scrollback", OptionValue::Number(1.0)),
            ],
        )))
        .unwrap(),
    )
}
struct Adapter {
    id: usize,
    drops: Rc<RefCell<Vec<usize>>>,
    callback: Option<Box<dyn FnOnce()>>,
}
impl Drop for Adapter {
    fn drop(&mut self) {
        self.drops.borrow_mut().push(self.id);
        if let Some(callback) = self.callback.take() {
            callback();
        }
    }
}
pub fn resource(
    id: usize,
    drops: Rc<RefCell<Vec<usize>>>,
    callback: Option<Box<dyn FnOnce()>>,
) -> Box<dyn Any> {
    Box::new(Adapter {
        id,
        drops,
        callback,
    })
}
pub fn factory(calls: Rc<Cell<usize>>, drops: Rc<RefCell<Vec<usize>>>) -> CleanupFactory {
    Rc::new(move |_| {
        let id = calls.get();
        calls.set(id + 1);
        Ok(resource(id, Rc::clone(&drops), None))
    })
}
