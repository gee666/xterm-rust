// Retained Antigone assertion. Source async WriteBuffer delivery enters a
// recursive flush while its active chunk is still present in the queue.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    headless::{NativeWritePump, QueuedTerminal, Terminal},
    input::write_buffer::WriteData,
};

#[test]
fn async_bell_resize_preserves_source_flush_reentry() {
    let term = Rc::new(Terminal::new(8, 3, 10).unwrap());
    let host =
        QueuedTerminal::with_pump(Rc::clone(&term), NativeWritePump::with_clock(|| 100.0)).unwrap();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&log);
    term.on_resize()
        .subscribe(move |event| output.borrow_mut().push(format!("resize:{}", event.cols)));
    let target = Rc::downgrade(&term);
    let output = Rc::clone(&log);
    term.on_bell().subscribe(move |_| {
        output.borrow_mut().push("bell".into());
        target.upgrade().unwrap().resize(6, 3).unwrap();
    });
    let output = Rc::clone(&log);
    host.write(
        WriteData::text("A\x07B"),
        Some(Rc::new(move |_| {
            output.borrow_mut().push("callback".into());
            Ok(())
        })),
    )
    .unwrap();
    host.pump().poll().unwrap();
    let row = term
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    assert_eq!(
        String::from_utf16(&row.borrow().translate_to_string(true, None, None, None)).unwrap(),
        "AABB"
    );
    assert_eq!(
        *log.borrow(),
        ["bell", "bell", "resize:6", "callback", "resize:6"]
    );
}
