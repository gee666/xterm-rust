//! Source-derived print/provider/REP callback checkpoints, PR02/03/05/08/09.
use crate::{
    buffer::{constants::attributes, AttributeData, CellData},
    event::EmitError,
    options::OptionValue,
    terminal::{CoreTerminal, TerminalError, TerminalEvent, TerminalState},
    unicode::{UnicodeCharProperties, UnicodeCharWidth, UnicodeVersionProvider},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

fn terminal(cols: usize) -> CoreTerminal {
    let mut terminal = CoreTerminal::new(cols, 3, 10).unwrap();
    terminal
        .state()
        .options
        .set("screenReaderMode", OptionValue::Bool(true))
        .unwrap();
    terminal.drain_events();
    terminal
}
fn cell(terminal: &CoreTerminal, row: usize, column: usize) -> CellData {
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    let line = buffer.lines.get(buffer.ybase + row).unwrap();
    let value = line.borrow().get_cell(column).unwrap();
    value
}
struct Provider {
    state: Option<Weak<RefCell<TerminalState>>>,
}
impl UnicodeVersionProvider for Provider {
    fn version(&self) -> &str {
        "print-test"
    }
    fn wcwidth(&self, _: u32) -> UnicodeCharWidth {
        UnicodeCharWidth::Narrow
    }
    fn char_properties(&self, code: u32, _: UnicodeCharProperties) -> UnicodeCharProperties {
        if let Some(state) = &self.state {
            state
                .upgrade()
                .unwrap()
                .try_borrow_mut()
                .expect("provider must reenter terminal state")
                .attributes
                .fg = attributes::CM_RGB | code;
        }
        UnicodeCharProperties::new(code, 1, false)
    }
}

#[test]
fn unicode_provider_reentry_updates_the_live_captured_attribute_object() {
    let mut terminal = terminal(8);
    let state = terminal.state_handle();
    let unicode = Rc::clone(&state.borrow().unicode);
    unicode
        .register(Rc::new(Provider {
            state: Some(Rc::downgrade(&state)),
        }))
        .unwrap();
    unicode.set_active_version("print-test").unwrap();
    terminal.write("AB").unwrap();
    assert_eq!(
        cell(&terminal, 0, 0).fg,
        attributes::CM_RGB | u32::from(b'A')
    );
    assert_eq!(
        cell(&terminal, 0, 1).fg,
        attributes::CM_RGB | u32::from(b'B')
    );
}

#[test]
fn a11y_and_wrap_modes_are_snapshots_until_the_next_print_run() {
    let mut terminal = terminal(2);
    let state = Rc::downgrade(&terminal.state_handle());
    let calls = Rc::new(Cell::new(0));
    let output = Rc::clone(&calls);
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::AccessibilityChar(_) = event {
            output.set(output.get() + 1);
            if output.get() == 1 {
                let state = state.upgrade().unwrap();
                let (core, options) = {
                    let state = state.borrow();
                    (Rc::clone(&state.core), Rc::clone(&state.options))
                };
                core.dec_private_modes.borrow_mut().wraparound = false;
                options
                    .set("screenReaderMode", OptionValue::Bool(false))
                    .map_err(EmitError::callback)?;
            }
        }
        Ok(())
    })));
    terminal.write("ABC").unwrap();
    assert_eq!(calls.get(), 3);
    assert_eq!(cell(&terminal, 0, 0).get_chars(), [u16::from(b'A')]);
    assert_eq!(cell(&terminal, 0, 1).get_chars(), [u16::from(b'B')]);
    assert_eq!(cell(&terminal, 1, 0).get_chars(), [u16::from(b'C')]);
    terminal.write("DE").unwrap();
    assert_eq!(calls.get(), 3);
    assert_eq!(cell(&terminal, 1, 1).get_chars(), [u16::from(b'E')]);
}

#[test]
fn replacement_does_not_retarget_the_original_attribute_object() {
    let mut terminal = terminal(8);
    let state = terminal.state_handle();
    state.borrow_mut().attributes.fg = attributes::CM_RGB | 0xff0000;
    let target = Rc::downgrade(&state);
    let first = Cell::new(true);
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::AccessibilityChar(_) = event {
            let state = target.upgrade().unwrap();
            let mut state = state.try_borrow_mut().expect("a11y must reenter state");
            if first.replace(false) {
                state.attributes.fg = attributes::CM_RGB | 0x00ff00;
                state.replace_attributes(AttributeData::default());
                state.attributes.fg = attributes::CM_RGB | 0x0000ff;
            } else {
                state.attributes.fg = attributes::CM_RGB | 0xff00ff;
            }
        }
        Ok(())
    })));
    terminal.write("AB").unwrap();
    assert_eq!(cell(&terminal, 0, 0).fg, attributes::CM_RGB | 0x00ff00);
    assert_eq!(cell(&terminal, 0, 1).fg, attributes::CM_RGB | 0x00ff00);
    assert_eq!(state.borrow().attributes.fg, attributes::CM_RGB | 0xff00ff);
}

#[test]
fn callback_error_stops_before_cell_write_and_keeps_preceding_state() {
    let mut terminal = terminal(8);
    let unicode = Rc::clone(&terminal.state().unicode);
    unicode.register(Rc::new(Provider { state: None })).unwrap();
    unicode.set_active_version("print-test").unwrap();
    terminal.write("Z").unwrap();
    terminal.drain_events();
    let preceding = terminal.parser_mut().preceding_join_state.get();
    let failure = EmitError::callback(std::io::Error::other("a11y listener"));
    let original = failure.clone();
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::AccessibilityChar(_) = event {
            return Err(failure.clone());
        }
        Ok(())
    })));
    let error = terminal.write("AB").unwrap_err();
    let TerminalError::Event(actual) = error else {
        panic!("lost callback error");
    };
    assert_eq!(actual, original);
    assert_eq!(terminal.parser_mut().preceding_join_state.get(), preceding);
    assert_eq!(terminal.state().active_buffer().borrow().x, 1);
    assert_eq!(cell(&terminal, 0, 0).get_chars(), [u16::from(b'Z')]);
    assert!(cell(&terminal, 0, 1).get_chars().is_empty());
}

#[test]
fn rep_delivers_each_callback_before_its_repeated_cell() {
    let mut terminal = terminal(8);
    terminal.write("A").unwrap();
    terminal.drain_events();
    let state = Rc::downgrade(&terminal.state_handle());
    let observations = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observations);
    terminal.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::AccessibilityChar(value) = event {
            assert_eq!(value, [u16::from(b'A')]);
            let state = state.upgrade().unwrap();
            let state = state.try_borrow().expect("REP must release state");
            let active = state.active_buffer();
            let buffer = active.borrow();
            let line = buffer.lines.get(buffer.ybase + buffer.y).unwrap();
            assert!(line.borrow().get_string(buffer.x).unwrap().is_empty());
            output.borrow_mut().push(buffer.x);
        }
        Ok(())
    })));
    terminal.write("\x1b[3b").unwrap();
    assert_eq!(*observations.borrow(), [1, 2, 3]);
    for column in 0..4 {
        assert_eq!(cell(&terminal, 0, column).get_chars(), [u16::from(b'A')]);
    }
}
