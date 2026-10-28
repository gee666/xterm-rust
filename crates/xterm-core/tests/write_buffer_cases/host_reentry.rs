// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::Rc,
};
use xterm_core::input::write_buffer::{
    WeakWriteBuffer, WriteAction, WriteBuffer, WriteData, WriteError, WriteScheduler,
};
type Timer = (WeakWriteBuffer<ReentrantHost>, f64, bool);
#[derive(Default)]
struct State {
    entered: Cell<bool>,
    next: Cell<usize>,
    timers: RefCell<BTreeMap<usize, Timer>>,
}
#[derive(Clone, Default)]
struct ReentrantHost(Rc<State>);
impl WriteScheduler for ReentrantHost {
    type Handle = usize;
    fn request(
        &self,
        queue: WeakWriteBuffer<Self>,
        time: f64,
        result: bool,
    ) -> Result<usize, WriteError> {
        if !self.0.entered.replace(true) {
            queue
                .upgrade()
                .unwrap()
                .write(WriteData::text("nested"), None)?;
        }
        let id = self.0.next.get();
        self.0.next.set(id + 1);
        self.0.timers.borrow_mut().insert(id, (queue, time, result));
        Ok(id)
    }
    fn cancel(&self, id: usize) -> Result<(), WriteError> {
        self.0.timers.borrow_mut().remove(&id);
        Ok(())
    }
    fn now(&self) -> f64 {
        100.0
    }
    fn spawn(&self, _: Pin<Box<dyn Future<Output = ()>>>) {
        panic!("this source scenario has no pending action");
    }
    fn report_error(&self, error: WriteError) {
        panic!("unexpected source scenario failure: {error}");
    }
}
#[test]
fn request_host_reentry_keeps_both_source_timers_and_two_parsed_events() {
    let host = ReentrantHost::default();
    let log = Rc::new(RefCell::new(vec![]));
    let actions = log.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, data, _| {
        let WriteData::Text(text) = data else {
            panic!("source scenario only writes text")
        };
        actions
            .borrow_mut()
            .push(format!("action:{}", String::from_utf16(text).unwrap()));
        Ok(WriteAction::Complete)
    });
    let events = log.clone();
    queue
        .on_write_parsed()
        .subscribe(move |_| events.borrow_mut().push("parsed".into()));
    queue.write(WriteData::text("outer"), None).unwrap();
    assert_eq!(host.0.timers.borrow().len(), 2);
    for _ in 0..2 {
        let (_, (target, time, result)) = host.0.timers.borrow_mut().pop_first().unwrap();
        target.upgrade().unwrap().process(time, result).unwrap();
    }
    assert_eq!(
        *log.borrow(),
        ["action:nested", "action:outer", "parsed", "parsed"]
    );
    assert!(host.0.timers.borrow().is_empty());
}
