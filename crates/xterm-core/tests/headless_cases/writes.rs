use super::support::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    headless::{HeadlessError, Terminal},
    options::{OptionValue, OptionsService},
    public_api::addon_manager::TerminalAddon,
};

#[test]
fn default_options() {
    let term = terminal(80, 24);
    assert_eq!(term.cols().unwrap(), 80);
    assert_eq!(term.rows().unwrap(), 24);
}
#[test]
fn proposed_api_check() {
    let term = Terminal::new(80, 24, 1000).unwrap();
    assert_eq!(
        term.unicode().err().unwrap().to_string(),
        "You must set the allowProposedApi option to true to use proposed API"
    );
}
#[test]
fn write() {
    let term = terminal(80, 24);
    for text in ["foo", "bar", "文"] {
        term.write(text).unwrap();
    }
    line_equals(&term, 0, "foobar文");
}
#[test]
fn write_with_callback() {
    let term = terminal(80, 24);
    let result = RefCell::new(String::new());
    for (text, letter) in [("foo", "a"), ("bar", "b"), ("文", "c")] {
        term.write_with_callback(text, |_| {
            result.borrow_mut().push_str(letter);
            Ok(())
        })
        .unwrap();
    }
    line_equals(&term, 0, "foobar文");
    assert_eq!(*result.borrow(), "abc");
}
#[test]
fn write_bytes_utf8() {
    let term = terminal(80, 24);
    for bytes in [&[102, 111, 111][..], &[98, 97, 114], &[230, 150, 135]] {
        term.write_bytes(bytes).unwrap();
    }
    line_equals(&term, 0, "foobar文");
}
#[test]
fn write_bytes_utf8_with_callback() {
    let term = terminal(80, 24);
    let result = RefCell::new(String::new());
    for (bytes, letter) in [
        (&[102, 111, 111][..], "A"),
        (&[98, 97, 114][..], "B"),
        (&[230, 150, 135][..], "C"),
    ] {
        term.write_bytes_with_callback(bytes, |_| {
            result.borrow_mut().push_str(letter);
            Ok(())
        })
        .unwrap();
    }
    line_equals(&term, 0, "foobar文");
    assert_eq!(*result.borrow(), "ABC");
}
#[test]
fn writeln() {
    let term = terminal(80, 24);
    for text in ["foo", "bar", "文"] {
        term.writeln(text).unwrap();
    }
    for (index, text) in ["foo", "bar", "文"].iter().enumerate() {
        line_equals(&term, index, text);
    }
}
#[test]
fn writeln_with_callback() {
    let term = terminal(80, 24);
    let result = RefCell::new(String::new());
    for (text, letter) in [("foo", "1"), ("bar", "2"), ("文", "3")] {
        term.writeln_with_callback(text, |_| {
            result.borrow_mut().push_str(letter);
            Ok(())
        })
        .unwrap();
    }
    for (index, text) in ["foo", "bar", "文"].iter().enumerate() {
        line_equals(&term, index, text);
    }
    assert_eq!(*result.borrow(), "123");
}
#[test]
fn writeln_bytes_utf8() {
    let term = terminal(80, 24);
    for bytes in [&[102, 111, 111][..], &[98, 97, 114], &[230, 150, 135]] {
        term.writeln_bytes(bytes).unwrap();
    }
    for (index, text) in ["foo", "bar", "文"].iter().enumerate() {
        line_equals(&term, index, text);
    }
}
#[test]
fn clear() {
    let term = terminal(80, 5);
    for index in 0..10 {
        term.write(&format!("\n\rtest{index}")).unwrap();
    }
    term.clear().unwrap();
    assert_eq!(term.buffer().unwrap().active().unwrap().len().unwrap(), 5);
    line_equals(&term, 0, "test9");
    for index in 1..5 {
        line_equals(&term, index, "");
    }
}
#[test]
fn clear_disposes_markers() {
    let term = terminal(80, 5);
    for index in 0..10 {
        term.write(&format!("\n\rtest{index}")).unwrap();
    }
    let markers = (1..=4)
        .map(|offset| term.register_marker(offset).unwrap())
        .collect::<Vec<_>>();
    let count = Rc::new(Cell::new(0));
    for marker in &markers {
        let count = Rc::clone(&count);
        marker
            .on_dispose()
            .subscribe(move |_| count.set(count.get() + 1));
    }
    term.clear().unwrap();
    assert_eq!(count.get(), markers.len());
    for marker in &markers {
        assert!(marker.is_disposed());
    }
    assert!(term.markers().unwrap().is_empty());
}
#[test]
fn get_options() {
    let term = terminal(80, 24);
    assert_eq!(
        term.options().get("lineHeight").unwrap(),
        OptionValue::Number(1.0)
    );
    assert_eq!(
        term.options().get("cursorWidth").unwrap(),
        OptionValue::Number(1.0)
    );
}
#[test]
fn set_options() {
    let term = terminal(80, 24);
    term.options()
        .set("scrollback", OptionValue::Number(1.0))
        .unwrap();
    assert_eq!(
        term.options().get("scrollback").unwrap(),
        OptionValue::Number(1.0)
    );
    term.options()
        .assign([
            ("fontSize", OptionValue::Number(12.0)),
            ("fontFamily", OptionValue::from("Arial")),
        ])
        .unwrap();
    assert_eq!(
        term.options().get("fontSize").unwrap(),
        OptionValue::Number(12.0)
    );
    assert_eq!(
        term.options().get("fontFamily").unwrap(),
        OptionValue::from("Arial")
    );
}
struct Addon {
    cols: Rc<Cell<usize>>,
    disposed: Rc<Cell<bool>>,
}
impl TerminalAddon<Terminal> for Addon {
    type Error = HeadlessError;
    fn activate(&self, term: &Terminal) -> Result<(), Self::Error> {
        self.cols.set(term.cols()?);
        Ok(())
    }
    fn dispose(&self) -> Result<(), Self::Error> {
        self.disposed.set(true);
        Ok(())
    }
}
fn addon() -> Rc<Addon> {
    Rc::new(Addon {
        cols: Rc::new(Cell::new(0)),
        disposed: Rc::new(Cell::new(false)),
    })
}
#[test]
fn load_addon_constructor() {
    let term = terminal(5, 24);
    let addon = addon();
    term.load_addon(addon.clone()).unwrap();
    assert_eq!(addon.cols.get(), 5);
}
#[test]
fn load_addon_dispose_addon() {
    let term = terminal(80, 24);
    let addon = addon();
    let handle = term.load_addon(addon.clone()).unwrap();
    assert!(!addon.disposed.get());
    handle.dispose().unwrap();
    assert!(addon.disposed.get());
}
#[test]
fn load_addon_dispose_terminal() {
    let term = terminal(80, 24);
    let addon = addon();
    term.load_addon(addon.clone()).unwrap();
    assert!(!addon.disposed.get());
    term.dispose().unwrap();
    assert!(addon.disposed.get());
}
#[test]
fn dispose() {
    let term = Terminal::with_options(Rc::new(OptionsService::default())).unwrap();
    term.dispose().unwrap();
    assert!(term.is_disposed().unwrap());
}
