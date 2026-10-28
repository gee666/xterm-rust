use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    headless::{NativeWritePump, QueuedTerminal, Terminal},
    input::write_buffer::{WriteAction, WriteBuffer, WriteData, WriteScheduler},
    parser::{escape::ParseError, handler::HandlerResult},
    terminal::TerminalError,
};
#[test]
fn earlier_timer_can_cancel_a_later_timer_in_the_same_host_poll() {
    let pump = NativeWritePump::new();
    let scheduler = Rc::new(RefCell::new(pump.scheduler()));
    let cancelled = Rc::new(Cell::new(None));
    let target = Rc::clone(&cancelled);
    let host = Rc::clone(&scheduler);
    let first = WriteBuffer::new(pump.scheduler(), move |_, _, _| {
        host.borrow_mut().cancel(target.get().unwrap())?;
        Ok(WriteAction::Complete)
    });
    first.write(WriteData::text("a"), None).unwrap();
    let second = WriteBuffer::new(pump.scheduler(), |_, _, _| Ok(WriteAction::Complete));
    second.write(WriteData::text("b"), None).unwrap();
    cancelled.set(Some(
        scheduler
            .borrow_mut()
            .request(second.downgrade(), 0.0, true)
            .unwrap(),
    ));
    assert_eq!(pump.poll().unwrap(), 2);
}
#[test]
fn sync_flush_during_paused_handler_preserves_improper_continuation_error() {
    let terminal = Rc::new(Terminal::new(10, 3, 10).unwrap());
    let host = QueuedTerminal::new(terminal).unwrap();
    host.terminal()
        .with_parser(|parser| {
            parser.register_osc_handler(999.0, |_| {
                HandlerResult::Pending(Box::pin(std::future::pending()))
            })
        })
        .unwrap()
        .unwrap();
    host.write(WriteData::text("\x1b]999;payload\x07B"), None)
        .unwrap();
    host.pump().poll().unwrap();
    let error = host.queue().flush_sync().unwrap_err();
    assert!(matches!(
        error.external_ref::<TerminalError>(),
        Some(TerminalError::Parse(ParseError::ImproperContinuation))
    ));
}
#[test]
fn resize_flushes_queued_text_and_callback_before_resizing_real_buffer() {
    let host = QueuedTerminal::new(Rc::new(Terminal::new(10, 3, 10).unwrap())).unwrap();
    let order = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&order);
    host.terminal()
        .on_resize()
        .subscribe(move |_| output.borrow_mut().push("resize"));
    let output = Rc::clone(&order);
    host.write(
        WriteData::text("abc"),
        Some(Rc::new(move |_| {
            output.borrow_mut().push("callback");
            Ok(())
        })),
    )
    .unwrap();
    host.resize(5, 3).unwrap();
    assert_eq!(*order.borrow(), ["callback", "resize"]);
    let row = host
        .terminal()
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf16(&row.borrow().translate_to_string(true, None, None, None)).unwrap(),
        "abc"
    );
    assert_eq!(host.terminal().cols().unwrap(), 5);
}
#[test]
fn unchanged_headless_resize_keeps_queued_text_deferred() {
    let host = QueuedTerminal::new(Rc::new(Terminal::new(10, 3, 10).unwrap())).unwrap();
    host.write(WriteData::text("abc"), None).unwrap();
    host.resize(10, 3).unwrap();
    assert_eq!(host.queue().pending_units(), 3.0);
    host.pump().poll().unwrap();
    assert_eq!(host.queue().pending_units(), 0.0);
}
