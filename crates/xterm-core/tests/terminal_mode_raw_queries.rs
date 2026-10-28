//! Scalar RawOptionsStore getters may reenter the same terminal during DECRQM.
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::{Rc, Weak},
};
use xterm_core::{
    headless::Terminal,
    options::{CoercionError, OptionError, OptionValue, OptionsService, RawOptionsStore},
};

struct ReentrantStore {
    values: RefCell<BTreeMap<String, OptionValue>>,
    terminal: RefCell<Weak<Terminal>>,
    observed_columns: RefCell<Vec<usize>>,
    key: &'static str,
    armed: Cell<bool>,
}
impl RawOptionsStore for ReentrantStore {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        if key == self.key && self.armed.get() {
            let terminal = self
                .terminal
                .borrow()
                .upgrade()
                .expect("test retains terminal");
            let cols = terminal
                .cols()
                .map_err(|error| CoercionError::new(&error.to_string()))?;
            self.observed_columns.borrow_mut().push(cols);
        }
        Ok(self
            .values
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or(OptionValue::Undefined))
    }
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        self.values.borrow_mut().insert(key.to_owned(), value);
        Ok(())
    }
}

#[test]
fn ansi20_and_dec12_raw_getters_read_live_terminal_without_state_borrow() {
    for (key, input, expected) in [
        ("convertEol", "\x1b[20$p", "\x1b[20;1$y"),
        ("cursorBlink", "\x1b[?12$p", "\x1b[?12;1$y"),
    ] {
        let defaults = OptionsService::default();
        let layout = defaults.property_layout();
        let values = defaults
            .keys()
            .into_iter()
            .map(|key| (key.to_owned(), defaults.get(key).unwrap()))
            .collect();
        let store = Rc::new(ReentrantStore {
            values: RefCell::new(values),
            terminal: RefCell::new(Weak::new()),
            observed_columns: RefCell::new(Vec::new()),
            key,
            armed: Cell::new(false),
        });
        let options = Rc::new(defaults.with_raw_store(store.clone(), layout));
        let terminal = Rc::new(Terminal::with_options(options).unwrap());
        *store.terminal.borrow_mut() = Rc::downgrade(&terminal);
        store.set(key, OptionValue::Bool(true)).unwrap();
        store.armed.set(true);
        let replies = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&replies);
        terminal
            .on_data()
            .subscribe(move |text| output.borrow_mut().push(String::from_utf16(text).unwrap()));
        terminal.write(input).unwrap();
        assert_eq!(*replies.borrow(), [expected]);
        assert_eq!(*store.observed_columns.borrow(), [80]);
        terminal.dispose().unwrap();
    }
}
