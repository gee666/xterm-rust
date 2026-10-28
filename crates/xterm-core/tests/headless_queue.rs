use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};
use xterm_core::{
    headless::{NativeWritePump, QueuedTerminal, Terminal},
    input::write_buffer::{WriteAction, WriteBuffer, WriteData, WriteError},
    parser::handler::{DispatchError, HandlerResult},
};
fn line(term: &Terminal) -> String {
    let row = term
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    let result =
        String::from_utf16(&row.borrow().translate_to_string(true, None, None, None)).unwrap();
    result
}
fn host() -> QueuedTerminal {
    QueuedTerminal::with_pump(
        Rc::new(Terminal::new(20, 3, 10).unwrap()),
        NativeWritePump::with_clock(|| 100.0),
    )
    .unwrap()
}
#[test]
fn fifo_callbacks_precede_one_batched_parsed_notification() {
    let host = host();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    host.terminal()
        .on_write_parsed()
        .subscribe(move |_| output.borrow_mut().push("parsed"));
    for (text, label) in [("foo", "a"), ("bar", "b"), ("文", "c")] {
        let output = Rc::clone(&calls);
        host.write(
            WriteData::text(text),
            Some(Rc::new(move |_| {
                output.borrow_mut().push(label);
                Ok(())
            })),
        )
        .unwrap();
    }
    assert_eq!(line(host.terminal()), "");
    assert!(calls.borrow().is_empty());
    assert!(host.pump().poll().unwrap() > 0);
    assert_eq!(line(host.terminal()), "foobar文");
    assert_eq!(*calls.borrow(), ["a", "b", "c", "parsed"]);
    assert_eq!(host.pump().pending_tasks(), 0);
}
#[test]
fn reentrant_callback_appends_to_the_same_fifo_without_losing_input() {
    let host = host();
    host.write(
        WriteData::text("a"),
        Some(Rc::new(|queue| queue.write(WriteData::text("b"), None))),
    )
    .unwrap();
    host.pump().poll().unwrap();
    assert_eq!(line(host.terminal()), "ab");
}
#[test]
fn writeln_bytes_and_crlf_callback_observe_completed_state() {
    let host = host();
    let weak = Rc::downgrade(host.terminal());
    let observed = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observed);
    host.writeln(
        WriteData::Bytes("文".as_bytes().to_vec()),
        Some(Rc::new(move |_| {
            let terminal = weak.upgrade().unwrap();
            output.borrow_mut().push(
                terminal
                    .buffer()
                    .unwrap()
                    .active()
                    .unwrap()
                    .cursor_y()
                    .unwrap(),
            );
            Ok(())
        })),
    )
    .unwrap();
    host.pump().poll().unwrap();
    assert_eq!(line(host.terminal()), "文");
    assert_eq!(*observed.borrow(), [1]);
}
#[test]
fn pending_parser_resumes_once_before_callback_and_later_chunks() {
    let host = host();
    let ready = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None));
    let polls = Rc::new(Cell::new(0));
    let (r, w, p) = (Rc::clone(&ready), Rc::clone(&waker), Rc::clone(&polls));
    host.terminal()
        .with_parser(|parser| {
            parser.register_osc_handler(999.0, move |_| {
                let wait = WaitForWake {
                    ready: Rc::clone(&r),
                    waker: Rc::clone(&w),
                    polls: Rc::clone(&p),
                };
                HandlerResult::PendingFallible(Box::pin(async move {
                    wait.await.map_err(|_| DispatchError::CallbackFailed)
                }))
            })
        })
        .unwrap()
        .unwrap();
    let called = Rc::new(Cell::new(false));
    let output = Rc::clone(&called);
    host.write(
        WriteData::text("A\x1b]999;x\x07B"),
        Some(Rc::new(move |_| {
            output.set(true);
            Ok(())
        })),
    )
    .unwrap();
    host.write(WriteData::text("C"), None).unwrap();
    host.pump().poll().unwrap();
    assert_eq!(line(host.terminal()), "A");
    assert!(!called.get());
    ready.set(true);
    waker.borrow_mut().take().unwrap().wake();
    host.pump().poll().unwrap();
    host.pump().poll().unwrap();
    assert_eq!(line(host.terminal()), "ABC");
    assert!(called.get());
}
#[test]
fn rejected_handler_retains_typed_error_and_resumes_false_into_older_handler() {
    let host = host();
    let older = Rc::new(Cell::new(false));
    let output = Rc::clone(&older);
    host.terminal()
        .with_parser(|parser| {
            parser
                .register_osc_handler(999.0, move |_| {
                    output.set(true);
                    HandlerResult::Ready(true)
                })
                .unwrap();
            parser
                .register_osc_handler(999.0, |_| {
                    HandlerResult::PendingFallible(Box::pin(async {
                        Err(DispatchError::CallbackFailed)
                    }))
                })
                .unwrap();
        })
        .unwrap();
    host.write(WriteData::text("\x1b]999;x\x07B"), None)
        .unwrap();
    let errors = host.pump().poll().unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        errors[0].external_ref::<DispatchError>(),
        Some(&DispatchError::CallbackFailed)
    );
    host.pump().poll().unwrap();
    assert!(older.get());
    assert_eq!(line(host.terminal()), "B");
}
#[test]
fn callback_error_is_returned_without_a_false_success_or_later_callback() {
    let host = host();
    let called = Rc::new(Cell::new(false));
    let output = Rc::clone(&called);
    host.write(
        WriteData::text("a"),
        Some(Rc::new(|_| {
            Err(WriteError::Callback("expected callback error".into()))
        })),
    )
    .unwrap();
    host.write(
        WriteData::text("b"),
        Some(Rc::new(move |_| {
            output.set(true);
            Ok(())
        })),
    )
    .unwrap();
    let errors = host.pump().poll().unwrap_err();
    assert_eq!(errors[0].to_string(), "expected callback error");
    assert_eq!(line(host.terminal()), "a");
    assert!(!called.get());
}
#[test]
fn dispose_cancels_queued_work_and_releases_host_timer() {
    let host = host();
    host.write(WriteData::text("lost"), None).unwrap();
    assert_eq!(host.pump().pending_tasks(), 1);
    host.dispose().unwrap();
    assert_eq!(host.pump().pending_tasks(), 0);
    assert_eq!(host.pump().poll().unwrap(), 0);
}
#[test]
fn user_input_drives_the_source_low_latency_path_but_disabled_stdin_does_not() {
    let host = host();
    host.terminal().input(&[97], true).unwrap();
    host.write(WriteData::text("a"), None).unwrap();
    assert_eq!(line(host.terminal()), "a");
    assert_eq!(host.pump().pending_tasks(), 0);
    host.terminal()
        .options()
        .set("disableStdin", xterm_core::options::OptionValue::Bool(true))
        .unwrap();
    host.terminal().input(&[98], true).unwrap();
    host.write(WriteData::text("b"), None).unwrap();
    assert_eq!(line(host.terminal()), "a");
    host.pump().poll().unwrap();
    assert_eq!(line(host.terminal()), "ab");
}
struct WaitForWake {
    ready: Rc<Cell<bool>>,
    waker: Rc<RefCell<Option<Waker>>>,
    polls: Rc<Cell<usize>>,
}
impl Future for WaitForWake {
    type Output = Result<bool, WriteError>;
    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        self.polls.set(self.polls.get() + 1);
        if self.ready.get() {
            Poll::Ready(Ok(true))
        } else {
            *self.waker.borrow_mut() = Some(context.waker().clone());
            Poll::Pending
        }
    }
}
#[test]
fn pending_local_future_is_not_busy_polled_and_wakes_on_demand() {
    let pump = NativeWritePump::new();
    let ready = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None));
    let polls = Rc::new(Cell::new(0));
    let (r, w, p) = (Rc::clone(&ready), Rc::clone(&waker), Rc::clone(&polls));
    let first = Cell::new(true);
    let queue = WriteBuffer::new(pump.scheduler(), move |_, _, resume| {
        if !first.replace(false) {
            assert_eq!(resume, Some(true));
            return Ok(WriteAction::Complete);
        }
        Ok(WriteAction::Pending(Box::pin(WaitForWake {
            ready: Rc::clone(&r),
            waker: Rc::clone(&w),
            polls: Rc::clone(&p),
        })))
    });
    queue.write(WriteData::text("a"), None).unwrap();
    pump.poll().unwrap();
    assert_eq!(polls.get(), 1);
    assert_eq!(pump.poll().unwrap(), 0);
    assert_eq!(polls.get(), 1);
    ready.set(true);
    waker.borrow_mut().take().unwrap().wake();
    pump.poll().unwrap();
    assert_eq!(polls.get(), 2);
    assert_eq!(queue.pending_units(), 0.0);
    assert_eq!(pump.pending_tasks(), 0);
    queue.dispose();
}
#[test]
fn dropping_pump_rejects_new_timers_without_leaking_queue_state() {
    let pump = NativeWritePump::new();
    let queue = WriteBuffer::new(pump.scheduler(), |_, _, _| Ok(WriteAction::Complete));
    drop(pump);
    let error = queue.write(WriteData::text("a"), None).unwrap_err();
    assert_eq!(error.to_string(), "native write pump is closed");
}
