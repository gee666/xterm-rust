use std::{cell::RefCell, convert::Infallible, rc::Rc};
use xterm_core::task_queue::{CallbackKind, Scheduler, TaskQueue};
type Hook = Rc<RefCell<Option<Box<dyn FnOnce()>>>>;
struct Host(Hook);
impl Scheduler for Host {
    type Handle = u32;
    type Error = Infallible;
    fn request(&self, _: CallbackKind) -> Result<u32, Infallible> { Ok(1) }
    fn cancel(&self, _: u32) {}
    fn now(&self) -> f64 {
        let hook = self.0.borrow_mut().take();
        if let Some(hook) = hook { hook(); }
        0.0
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), Infallible> { Ok(()) }
}
fn main() {
    let hook: Hook = Rc::default();
    let queue = TaskQueue::new(Host(hook.clone()), CallbackKind::Idle);
    let trace = Rc::new(RefCell::new(Vec::new()));
    let old = trace.clone();
    queue.enqueue(move |_| { old.borrow_mut().push("old"); false }).unwrap();
    let weak = queue.downgrade();
    let new = trace.clone();
    *hook.borrow_mut() = Some(Box::new(move || {
        let queue = weak.upgrade().unwrap();
        queue.clear();
        queue.enqueue(move |_| { new.borrow_mut().push("new"); false }).unwrap();
    }));
    queue.process(|| 100.0).unwrap();
    println!("{:?}", trace.borrow());
}
