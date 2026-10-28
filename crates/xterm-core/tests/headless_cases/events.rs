use super::support::*;
use std::{cell::RefCell, rc::Rc};
use xterm_core::headless::{RenderEvent, ResizeEvent};
#[test]
fn on_cursor_move() {
    let term = terminal(80, 24);
    let count = Rc::new(RefCell::new(0));
    let output = Rc::clone(&count);
    term.on_cursor_move()
        .subscribe(move |_| *output.borrow_mut() += 1);
    term.write("foo").unwrap();
    assert_eq!(*count.borrow(), 1);
    term.write("bar").unwrap();
    assert_eq!(*count.borrow(), 2);
}
#[test]
fn on_data() {
    let term = terminal(80, 24);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.on_data()
        .subscribe(move |data| output.borrow_mut().push(data.clone()));
    term.write("\x1b[5n").unwrap();
    assert_eq!(
        *calls.borrow(),
        vec!["\x1b[0n".encode_utf16().collect::<Vec<_>>()]
    );
}
#[test]
fn on_line_feed() {
    let term = terminal(80, 24);
    let count = Rc::new(RefCell::new(0));
    let output = Rc::clone(&count);
    term.on_line_feed()
        .subscribe(move |_| *output.borrow_mut() += 1);
    term.writeln("foo").unwrap();
    assert_eq!(*count.borrow(), 1);
    term.writeln("bar").unwrap();
    assert_eq!(*count.borrow(), 2);
}
#[test]
fn on_render() {
    let term = terminal(80, 24);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.on_render()
        .subscribe(move |event| output.borrow_mut().push(*event));
    term.write("foo").unwrap();
    assert_eq!(*calls.borrow(), [RenderEvent { start: 0, end: 0 }]);
    term.write("\n\nbar").unwrap();
    assert_eq!(
        *calls.borrow(),
        [
            RenderEvent { start: 0, end: 0 },
            RenderEvent { start: 0, end: 2 }
        ]
    );
}
#[test]
fn on_scroll() {
    let term = terminal(80, 5);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.on_scroll()
        .subscribe(move |position| output.borrow_mut().push(*position));
    for _ in 0..4 {
        term.writeln("foo").unwrap();
    }
    assert!(calls.borrow().is_empty());
    term.writeln("bar").unwrap();
    assert_eq!(*calls.borrow(), [1]);
    term.writeln("baz").unwrap();
    assert_eq!(*calls.borrow(), [1, 2]);
}
#[test]
fn on_resize() {
    let term = terminal(80, 24);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.on_resize()
        .subscribe(move |event| output.borrow_mut().push(*event));
    assert!(calls.borrow().is_empty());
    term.resize(10, 5).unwrap();
    assert_eq!(*calls.borrow(), [ResizeEvent { cols: 10, rows: 5 }]);
    term.resize(20, 15).unwrap();
    assert_eq!(
        *calls.borrow(),
        [
            ResizeEvent { cols: 10, rows: 5 },
            ResizeEvent { cols: 20, rows: 15 }
        ]
    );
}
#[test]
fn on_title_change() {
    let term = terminal(80, 24);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.on_title_change()
        .subscribe(move |title| output.borrow_mut().push(title.clone()));
    assert!(calls.borrow().is_empty());
    term.write("\x1b]2;foo\u{9c}").unwrap();
    assert_eq!(
        *calls.borrow(),
        vec!["foo".encode_utf16().collect::<Vec<_>>()]
    );
}
#[test]
fn on_bell() {
    let term = terminal(80, 24);
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.on_bell()
        .subscribe(move |_| output.borrow_mut().push(true));
    assert!(calls.borrow().is_empty());
    term.write("\x07").unwrap();
    assert_eq!(*calls.borrow(), [true]);
}
