// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    rc::{Rc, Weak},
};
use xterm_core::{
    buffer::{Buffer, BufferCleanup, BufferSet},
    options::OptionsService,
    task_queue::{CallbackKind, Scheduler},
};

struct Host {
    buffer: Weak<RefCell<Buffer>>,
    inspected: Rc<Cell<bool>>,
}
impl Scheduler for Host {
    type Handle = u64;
    type Error = Infallible;
    fn request(&self, _kind: CallbackKind) -> Result<u64, Infallible> {
        Ok(1)
    }
    fn cancel(&self, _handle: u64) {
        let buffer = self.buffer.upgrade().unwrap();
        let buffer = buffer
            .try_borrow()
            .expect("host cancellation must not encounter an outer Buffer borrow");
        assert!(!buffer.has_pending_cleanup());
        self.inspected.set(true);
    }
    fn now(&self) -> f64 {
        0.0
    }
    fn deadline_exceeded(&self, _milliseconds: f64) -> Result<(), Infallible> {
        panic!("fixed time cannot exceed deadline");
    }
}
#[test]
fn set_reset_releases_buffer_before_host_cleanup_cancellation() {
    let set = BufferSet::new(10, 5, Rc::new(OptionsService::default())).unwrap();
    let old = set.normal();
    let inspected = Rc::new(Cell::new(false));
    let _adapter = BufferCleanup::new(
        &old,
        Host {
            buffer: Rc::downgrade(&old),
            inspected: Rc::clone(&inspected),
        },
        true,
    )
    .unwrap();
    old.borrow_mut().resize(3, 5).unwrap();
    assert!(old.borrow().has_pending_cleanup());
    set.reset().unwrap();
    assert!(inspected.get());
}
