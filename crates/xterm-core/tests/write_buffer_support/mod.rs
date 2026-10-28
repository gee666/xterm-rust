// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};
use xterm_core::input::write_buffer::{
    WeakWriteBuffer, WriteBuffer, WriteCallback, WriteData, WriteError, WriteScheduler,
};
type Scheduled = (WeakWriteBuffer<Host>, f64, bool);
type LocalFuture = Pin<Box<dyn Future<Output = ()>>>;
#[derive(Default)]
pub struct HostState {
    pub time: Cell<f64>,
    pub fail_requests: Cell<bool>,
    next: Cell<usize>,
    tasks: RefCell<BTreeMap<usize, Scheduled>>,
    futures: RefCell<VecDeque<LocalFuture>>,
    pub errors: RefCell<Vec<WriteError>>,
}
#[derive(Clone, Default)]
pub struct Host(pub Rc<HostState>);
impl WriteScheduler for Host {
    type Handle = usize;
    fn request(
        &mut self,
        queue: WeakWriteBuffer<Self>,
        last_time: f64,
        result: bool,
    ) -> Result<usize, WriteError> {
        if self.0.fail_requests.get() {
            return Err(WriteError::Schedule("host request failed".into()));
        }
        let id = self.0.next.get();
        self.0.next.set(id + 1);
        self.0
            .tasks
            .borrow_mut()
            .insert(id, (queue, last_time, result));
        Ok(id)
    }
    fn cancel(&mut self, handle: usize) {
        self.0.tasks.borrow_mut().remove(&handle);
    }
    fn now(&self) -> f64 {
        self.0.time.get()
    }
    fn spawn(&mut self, future: LocalFuture) {
        self.0.futures.borrow_mut().push_back(future);
    }
    fn report_error(&mut self, error: WriteError) {
        self.0.errors.borrow_mut().push(error);
    }
}
impl Host {
    pub fn pending_tasks(&self) -> usize {
        self.0.tasks.borrow().len()
    }
    pub fn advance(&self, ms: f64) {
        self.0.time.set(self.0.time.get() + ms);
    }
    pub fn task(&self) -> Result<bool, WriteError> {
        let next = self.0.tasks.borrow_mut().pop_first();
        let Some((_, (queue, time, result))) = next else {
            return Ok(false);
        };
        if let Some(queue) = queue.upgrade() {
            queue.process(time, result)?;
        }
        Ok(true)
    }
    pub fn tasks(&self) {
        for _ in 0..100_000 {
            if !self.task().unwrap() {
                return;
            }
        }
        panic!("test host exceeded bounded task count");
    }
    pub fn microtasks(&self) {
        struct NoWake;
        impl Wake for NoWake {
            fn wake(self: Arc<Self>) {}
        }
        let waker = Waker::from(Arc::new(NoWake));
        let mut context = Context::from_waker(&waker);
        let count = self.0.futures.borrow().len();
        for _ in 0..count {
            let next = self.0.futures.borrow_mut().pop_front();
            if let Some(mut future) = next {
                if future.as_mut().poll(&mut context) == Poll::Pending {
                    self.0.futures.borrow_mut().push_back(future);
                }
            }
        }
    }
}
pub type Stack = Rc<RefCell<Vec<WriteData>>>;
pub type Calls = Rc<RefCell<Vec<String>>>;
pub fn queue() -> (WriteBuffer<Host>, Host, Stack, Calls) {
    let host = Host::default();
    let stack = Stack::default();
    let calls = Calls::default();
    let output = stack.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, data, _| {
        output.borrow_mut().push(data.clone());
        Ok(xterm_core::input::write_buffer::WriteAction::Complete)
    });
    (queue, host, stack, calls)
}
pub fn callback(calls: &Calls, label: &str) -> Option<WriteCallback<Host>> {
    let calls = calls.clone();
    let label = label.to_owned();
    Some(Rc::new(move |_| {
        calls.borrow_mut().push(label.clone());
        Ok(())
    }))
}
