use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use xterm_core::{
    headless::NativeWritePump,
    input::write_buffer::{WriteAction, WriteBuffer, WriteData, WriteScheduler},
};

#[test]
fn ready_continuation_from_timer_a_runs_before_timer_b() {
    let pump = NativeWritePump::with_clock(|| 100.0);
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&log);
    let resumed = Cell::new(false);
    let first = WriteBuffer::new(pump.scheduler(), move |_, _, _| {
        if resumed.replace(true) {
            output.borrow_mut().push("a-resume");
            Ok(WriteAction::Complete)
        } else {
            output.borrow_mut().push("a-start");
            Ok(WriteAction::Pending(Box::pin(async { Ok(true) })))
        }
    });
    let output = Rc::clone(&log);
    first
        .write(
            WriteData::text("a"),
            Some(Rc::new(move |_| {
                output.borrow_mut().push("a-callback");
                Ok(())
            })),
        )
        .unwrap();
    let output = Rc::clone(&log);
    let second = WriteBuffer::new(pump.scheduler(), move |_, _, _| {
        output.borrow_mut().push("b-start");
        Ok(WriteAction::Complete)
    });
    let output = Rc::clone(&log);
    second
        .write(
            WriteData::text("b"),
            Some(Rc::new(move |_| {
                output.borrow_mut().push("b-callback");
                Ok(())
            })),
        )
        .unwrap();
    pump.poll().unwrap();
    assert_eq!(
        *log.borrow(),
        ["a-start", "a-resume", "a-callback", "b-start", "b-callback"]
    );
}
#[test]
fn rejected_a_is_observed_before_false_continuation_and_timer_b() {
    let pump = NativeWritePump::with_clock(|| 100.0);
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&log);
    let resumed = Cell::new(false);
    let first = WriteBuffer::new(pump.scheduler(), move |_, _, result| {
        if resumed.replace(true) {
            assert_eq!(result, Some(false));
            output.borrow_mut().push("a-resume");
            Ok(WriteAction::Complete)
        } else {
            output.borrow_mut().push("a-start");
            Ok(WriteAction::Pending(Box::pin(async {
                Err(xterm_core::input::write_buffer::WriteError::Callback(
                    "a-error".into(),
                ))
            })))
        }
    });
    let output = Rc::clone(&log);
    first
        .write(
            WriteData::text("a"),
            Some(Rc::new(move |_| {
                output.borrow_mut().push("a-callback");
                Ok(())
            })),
        )
        .unwrap();
    let output = Rc::clone(&log);
    let second = WriteBuffer::new(pump.scheduler(), move |_, _, _| {
        output.borrow_mut().push("b-start");
        Ok(WriteAction::Complete)
    });
    let output = Rc::clone(&log);
    second
        .write(
            WriteData::text("b"),
            Some(Rc::new(move |_| {
                output.borrow_mut().push("b-callback");
                Ok(())
            })),
        )
        .unwrap();
    let errors = pump.poll().unwrap_err();
    assert_eq!(errors[0].to_string(), "a-error");
    log.borrow_mut().push("a-error");
    assert_eq!(*log.borrow(), ["a-start", "a-error"]);
    pump.poll().unwrap();
    assert_eq!(
        *log.borrow(),
        [
            "a-start",
            "a-error",
            "a-resume",
            "a-callback",
            "b-start",
            "b-callback"
        ]
    );
}
#[test]
fn already_ready_local_future_precedes_the_next_timer() {
    let pump = NativeWritePump::with_clock(|| 100.0);
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&log);
    pump.scheduler().spawn(Box::pin(async move {
        output.borrow_mut().push("continuation");
    }));
    let output = Rc::clone(&log);
    let queue = WriteBuffer::new(pump.scheduler(), move |_, _, _| {
        output.borrow_mut().push("timer");
        Ok(WriteAction::Complete)
    });
    queue.write(WriteData::text("b"), None).unwrap();
    pump.poll().unwrap();
    assert_eq!(*log.borrow(), ["continuation", "timer"]);
}
struct Deferred {
    name: &'static str,
    ready: Rc<Cell<bool>>,
    waker: Rc<RefCell<Option<Waker>>>,
    log: Rc<RefCell<Vec<&'static str>>>,
}
impl Future for Deferred {
    type Output = ();
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<()> {
        if self.ready.get() {
            self.log.borrow_mut().push(self.name);
            Poll::Ready(())
        } else {
            *self.waker.borrow_mut() = Some(context.waker().clone());
            Poll::Pending
        }
    }
}
#[test]
fn wake_order_is_fifo_not_future_registration_order() {
    let pump = NativeWritePump::with_clock(|| 100.0);
    let log = Rc::new(RefCell::new(Vec::new()));
    let a = Rc::new(Cell::new(false));
    let b = Rc::new(Cell::new(false));
    let aw = Rc::new(RefCell::new(None));
    let bw = Rc::new(RefCell::new(None));
    for (name, ready, waker) in [
        ("a", Rc::clone(&a), Rc::clone(&aw)),
        ("b", Rc::clone(&b), Rc::clone(&bw)),
    ] {
        pump.scheduler().spawn(Box::pin(Deferred {
            name,
            ready,
            waker,
            log: Rc::clone(&log),
        }));
    }
    pump.poll().unwrap();
    assert!(log.borrow().is_empty());
    b.set(true);
    bw.borrow_mut().take().unwrap().wake();
    a.set(true);
    aw.borrow_mut().take().unwrap().wake();
    pump.poll().unwrap();
    assert_eq!(*log.borrow(), ["b", "a"]);
    assert_eq!(pump.pending_tasks(), 0);
}
