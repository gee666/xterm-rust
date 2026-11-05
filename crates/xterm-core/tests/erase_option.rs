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

const PRELUDE: &str = "\x1b[31;44mAAAAA\r\nBBBBB\r\n\x1b[1\"qCCCCC\x1b[2\"q\x1b[3;5H";
type Getter = Rc<dyn Fn() -> Result<(), OptionError>>;
struct Store {
    values: RefCell<BTreeMap<String, OptionValue>>,
    getter: RefCell<Option<Getter>>,
    calls: Cell<usize>,
    armed: Cell<bool>,
    value: OptionValue,
}
impl RawOptionsStore for Store {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        if key == "scrollOnEraseInDisplay" {
            let callback = self.getter.borrow().clone();
            if let Some(callback) = callback {
                self.calls.set(self.calls.get() + 1);
                if self.armed.replace(false) {
                    callback()?;
                }
                return Ok(self.value.clone());
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
fn raw_value(name: &str) -> OptionValue {
    match name {
        "false" => OptionValue::Bool(false),
        "true" => OptionValue::Bool(true),
        "zero" => OptionValue::Number(0.0),
        "nan" => OptionValue::Number(f64::NAN),
        "empty" => OptionValue::from(""),
        "string" => OptionValue::from("false"),
        "object" => OptionValue::object(()),
        "null" => OptionValue::Null,
        "undefined" => OptionValue::Undefined,
        other => panic!("unknown source value {other}"),
    }
}

#[test]
fn ed2_getter_matches_source_reentry_truthiness_and_error_phases() {
    let mut failures = Vec::new();
    let mut cases = 0;
    for (index, row) in include_str!("fixtures/erase-option.tsv")
        .lines()
        .enumerate()
    {
        cases += 1;
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 9);
        let mut values: BTreeMap<String, OptionValue> = default_options(false)
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect();
        for (key, value) in [("cols", 5.0), ("rows", 4.0), ("scrollback", 2.0)] {
            values.insert(key.into(), OptionValue::Number(value));
        }
        let store = Rc::new(Store {
            values: RefCell::new(values),
            getter: RefCell::new(None),
            calls: Cell::new(0),
            armed: Cell::new(true),
            value: raw_value(fields[2]),
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
        terminal.write(PRELUDE).unwrap();
        terminal.state().active_buffer().borrow_mut().x = 7;
        assert!(!projection::events(&mut terminal).is_empty());
        let records = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&records);
        let weak = Rc::downgrade(&terminal.state_handle());
        let event_state = weak.clone();
        let action = fields[3].to_owned();
        let original = EmitError::callback(std::io::Error::other("erase option sentinel"));
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
                "attributes" => {
                    shared
                        .try_borrow_mut()
                        .map_err(|_| option_error(TerminalError::Borrowed))?
                        .attributes
                        .bg = 0x1000002
                }
                "inspect" | "event-reset" | "event-error" => {}
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
        let event_armed = Cell::new(true);
        terminal.set_event_handler(Some(Rc::new(move |event| {
            let text = match &event {
                TerminalEvent::Resize { cols, rows } => Some(format!("Z{cols}:{rows}")),
                event => projection::event(event),
            };
            if let Some(text) = text {
                event_output.borrow_mut().push(text);
            }
            if matches!(event, TerminalEvent::Scroll(_)) {
                let shared = event_state.upgrade().unwrap();
                callback_records
                    .borrow_mut()
                    .push(projection::snapshot_state(&shared.borrow()));
                if event_armed.replace(false) {
                    if action == "event-reset" {
                        reset_state_handle(&shared).map_err(EmitError::callback)?;
                    }
                    if action == "event-error" {
                        return Err(captured.clone());
                    }
                }
            }
            Ok(())
        })));
        let units = projection::parse_hex(if fields[1] == "1" {
            "1b,5b,3f,32,4a"
        } else {
            "1b,5b,32,4a"
        });
        assert_eq!(
            projection::hex(&units),
            if fields[1] == "1" {
                "1b,5b,3f,32,4a"
            } else {
                "1b,5b,32,4a"
            }
        );
        let result = terminal.write(&String::from_utf16(&units).unwrap());
        let valid_result = match &result {
            Ok(_) => fields[5] == "0",
            Err(error) => fields[5] == "1" && has_original(error, &original),
        };
        let mut mismatches = Vec::new();
        if !valid_result {
            mismatches.push(format!("result={result:?}"));
        }
        if store.calls.get().to_string() != fields[4] {
            mismatches.push("getter count".into());
        }
        if records.borrow().join("~") != fields[6] {
            mismatches.push("callback states".into());
        }
        if projection::snapshot(&terminal) != fields[7] {
            mismatches.push("final state".into());
        }
        let event_text = events.borrow().join("/");
        let expected_events = if fields[8] == "-" { "" } else { fields[8] };
        if event_text != expected_events {
            mismatches.push(format!(
                "events actual={:?} expected={:?}",
                events.borrow().join("/"),
                fields[8]
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
    assert_eq!(cases, 396);
    assert!(
        failures.is_empty(),
        "{} mismatching cases:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
