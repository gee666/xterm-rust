// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    error::Error,
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    options::{default_options, OptionError, OptionValue, OptionsService, RawOptionsStore},
    terminal::{reset_state_handle, CoreTerminal, TerminalError, TerminalEvent},
};

type Getter = Rc<dyn Fn() -> Result<(), OptionError>>;
struct Store {
    values: RefCell<BTreeMap<String, OptionValue>>,
    getter: RefCell<Option<Getter>>,
    calls: Cell<usize>,
    armed: Cell<bool>,
    enabled: bool,
}
impl RawOptionsStore for Store {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        if key == "screenReaderMode" {
            let callback = self.getter.borrow().clone();
            if let Some(callback) = callback {
                self.calls.set(self.calls.get() + 1);
                if self.armed.replace(false) {
                    callback()?;
                }
                return Ok(OptionValue::Bool(self.enabled));
            }
        }
        Ok(self.values.borrow().get(key).cloned().unwrap())
    }
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        self.values.borrow_mut().insert(key.into(), value);
        Ok(())
    }
}
fn option_error(error: TerminalError) -> OptionError {
    OptionError::Dispatch(EmitError::callback(error))
}
fn has_original(error: &(dyn Error + 'static), original: &EmitError) -> bool {
    let mut current = error;
    loop {
        if current.downcast_ref::<EmitError>() == Some(original) {
            return true;
        }
        match current.source() {
            Some(source) => current = source,
            None => return false,
        }
    }
}

#[test]
fn tab_screen_reader_getter_matches_source_reentry_signed_distance_and_errors() {
    let mut failures = Vec::new();
    for (index, row) in include_str!("fixtures/tab-option.tsv").lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 8);
        let enabled = fields[1] == "1";
        let column: usize = fields[2].parse().unwrap();
        let mut values: BTreeMap<String, OptionValue> = default_options(false)
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect();
        for (key, value) in [("cols", 12.0), ("rows", 3.0), ("scrollback", 2.0)] {
            values.insert(key.into(), OptionValue::Number(value));
        }
        let store = Rc::new(Store {
            values: RefCell::new(values),
            getter: RefCell::new(None),
            calls: Cell::new(0),
            armed: Cell::new(true),
            enabled,
        });
        let layout = default_options(false)
            .into_iter()
            .map(|(key, _)| (key.into(), true))
            .collect();
        let options = Rc::new(OptionsService::default().with_raw_store(store.clone(), layout));
        let mut terminal = CoreTerminal::with_options(options).unwrap();
        if fields[0] == "1" {
            terminal.write("\x1b[?1049h").unwrap();
        }
        terminal.write("AAAA\r\nBBBB\r\nCCCC").unwrap();
        terminal.state().active_buffer().borrow_mut().x = column;
        assert!(!projection::events(&mut terminal).is_empty());
        let records = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&records);
        let weak = Rc::downgrade(&terminal.state_handle());
        let event_state = weak.clone();
        let action = fields[3].to_owned();
        let original = EmitError::callback(std::io::Error::other("tab option sentinel"));
        let captured = original.clone();
        *store.getter.borrow_mut() = Some(Rc::new(move || {
            let shared = weak.upgrade().unwrap();
            let snapshot = || {
                shared
                    .try_borrow()
                    .map(|state| projection::snapshot_state(&state))
                    .map_err(|_| option_error(TerminalError::Borrowed))
            };
            output.borrow_mut().push(snapshot()?);
            match action.as_str() {
                "error" => return Err(OptionError::Dispatch(captured.clone())),
                "reset" => reset_state_handle(&shared).map_err(option_error)?,
                "alt" | "normal" => {
                    let buffers = Rc::clone(&shared.borrow().buffers);
                    if action == "alt" {
                        buffers.activate_alt_buffer(None)
                    } else {
                        buffers.activate_normal_buffer()
                    }
                    .map_err(TerminalError::from)
                    .map_err(option_error)?;
                }
                "resize" => {
                    let service = Rc::clone(&shared.borrow().buffer_service);
                    service
                        .resize(7, 5)
                        .map_err(TerminalError::from)
                        .map_err(option_error)?;
                }
                "cursor" | "mutate-error" => {
                    let active = shared.borrow().active_buffer();
                    let mut buffer = active.borrow_mut();
                    buffer.x = 1;
                    buffer.y = 0;
                }
                "inspect" | "event-cursor" | "event-error" => {}
                other => panic!("unknown source action {other}"),
            }
            output.borrow_mut().push(snapshot()?);
            if action == "mutate-error" {
                return Err(OptionError::Dispatch(captured.clone()));
            }
            Ok(())
        }));
        let events = Rc::new(RefCell::new(Vec::new()));
        let event_output = Rc::clone(&events);
        let callback_records = Rc::clone(&records);
        let action = fields[3].to_owned();
        let captured = original.clone();
        terminal.set_event_handler(Some(Rc::new(move |event| {
            let text = match &event {
                TerminalEvent::Resize { cols, rows } => Some(format!("Z{cols}:{rows}")),
                TerminalEvent::AccessibilityTab(count) => Some(format!("T{count}")),
                event => projection::event(event),
            };
            if let Some(text) = text {
                event_output.borrow_mut().push(text);
            }
            if matches!(event, TerminalEvent::AccessibilityTab(_)) {
                let shared = event_state.upgrade().unwrap();
                callback_records
                    .borrow_mut()
                    .push(projection::snapshot_state(&shared.borrow()));
                if action == "event-cursor" {
                    let active = shared.borrow().active_buffer();
                    let mut buffer = active.borrow_mut();
                    buffer.x = 1;
                    buffer.y = 0;
                }
                if action == "event-error" {
                    return Err(captured.clone());
                }
            }
            Ok(())
        })));
        let units = projection::parse_hex("9");
        assert_eq!(projection::hex(&units), "9");
        let result = terminal.write(&String::from_utf16(&units).unwrap());
        let throws = column < 12
            && (fields[3] == "error"
                || fields[3] == "mutate-error"
                || (enabled && fields[3] == "event-error"));
        let valid_result = match &result {
            Ok(_) => !throws,
            Err(error) => throws && has_original(error, &original),
        };
        let mut mismatches = Vec::new();
        if !valid_result {
            mismatches.push(format!("result={result:?}"));
        }
        if store.calls.get().to_string() != fields[4] {
            mismatches.push("getter count".into());
        }
        if records.borrow().join("~") != fields[5] {
            mismatches.push("callback states".into());
        }
        if projection::snapshot(&terminal) != fields[6] {
            mismatches.push("final state".into());
        }
        if events.borrow().join("/") != fields[7] {
            mismatches.push(format!(
                "events actual={:?} expected={:?}",
                events.borrow().join("/"),
                fields[7]
            ));
        }
        if !mismatches.is_empty() {
            failures.push(format!(
                "case {index} {}: {}",
                fields[..4].join(" "),
                mismatches.join(", ")
            ));
        }
        terminal.dispose().unwrap();
    }
    assert!(
        failures.is_empty(),
        "{} mismatching cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
