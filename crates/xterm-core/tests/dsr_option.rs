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
    options::{
        default_options, OptionError, OptionObject, OptionValue, OptionsService, RawOptionsStore,
    },
    terminal::{reset_state_handle, CoreTerminal, TerminalError, TerminalEvent},
};

const PRELUDE: &str = "\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\x1b[3;5H";
type Getter = Rc<dyn Fn() -> Result<(), OptionError>>;
struct Store {
    values: RefCell<BTreeMap<String, OptionValue>>,
    getter: RefCell<Option<Getter>>,
    special: RefCell<Option<OptionValue>>,
    bag_phase: bool,
    calls: Cell<usize>,
    armed: Cell<bool>,
}
impl Store {
    fn trigger(&self) -> Result<(), OptionError> {
        self.calls.set(self.calls.get() + 1);
        if self.armed.replace(false) {
            let callback = self.getter.borrow().clone().unwrap();
            callback()?;
        }
        Ok(())
    }
}
impl RawOptionsStore for Store {
    fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        if key == "vtExtensions" {
            let special = self.special.borrow().clone();
            if let Some(value) = special {
                if self.bag_phase {
                    self.trigger()?;
                }
                return Ok(value);
            }
        }
        Ok(self.values.borrow().get(key).cloned().unwrap())
    }
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        if key == "vtExtensions" {
            self.special.borrow_mut().take();
        }
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
        "undefined" => OptionValue::Undefined,
        "null" => OptionValue::Null,
        "false" => OptionValue::Bool(false),
        "true" => OptionValue::Bool(true),
        "zero" => OptionValue::Number(0.0),
        "nan" => OptionValue::Number(f64::NAN),
        "empty" => OptionValue::from(""),
        "text" => OptionValue::from("false"),
        "object" => OptionValue::Object(OptionObject::new((), |_| panic!("must not coerce"))),
        other => panic!("unknown source value {other}"),
    }
}

#[test]
fn private_dsr_raw_option_getters_match_source_states_events_and_original_errors() {
    let fixture = include_str!("fixtures/dsr-option.tsv");
    assert_eq!(fixture.lines().count(), 468);
    let mut failures = Vec::new();
    for (index, row) in fixture.lines().enumerate() {
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
            special: RefCell::new(None),
            bag_phase: fields[0] == "bag",
            calls: Cell::new(0),
            armed: Cell::new(true),
        });
        let layout = default_options(false)
            .into_iter()
            .map(|(key, _)| (key.into(), true))
            .collect();
        let options = Rc::new(OptionsService::default().with_raw_store(store.clone(), layout));
        let mut terminal = CoreTerminal::with_options(options).unwrap();
        if fields[1] == "1" {
            terminal.write("\x1b[?1049h").unwrap();
        }
        terminal.write(PRELUDE).unwrap();
        assert!(!projection::events(&mut terminal).is_empty());
        let records = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&records);
        let weak = Rc::downgrade(&terminal.state_handle());
        let event_state = weak.clone();
        let action = fields[3].to_owned();
        let original = EmitError::callback(std::io::Error::other("DSR option sentinel"));
        let captured = original.clone();
        *store.getter.borrow_mut() = Some(Rc::new(move || {
            let shared = weak.upgrade().unwrap();
            let snapshot = || {
                shared
                    .try_borrow()
                    .map(|state| format!("get:{}", projection::snapshot_state(&state)))
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
                    buffer.x = 3;
                    buffer.y = 0;
                }
                "modes" => {
                    let core = Rc::clone(&shared.borrow().core);
                    core.modes.borrow_mut().insert_mode = true;
                    core.dec_private_modes.borrow_mut().application_cursor_keys = true;
                }
                "disable" | "replace" => {
                    let options = Rc::clone(&shared.borrow().options);
                    if action == "disable" {
                        options.set_raw("disableStdin", OptionValue::Bool(true))?;
                    } else {
                        options.set_raw(
                            "vtExtensions",
                            OptionValue::property_map([(
                                "colorSchemeQuery",
                                OptionValue::Bool(false),
                            )]),
                        )?;
                    }
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
        let value = raw_value(fields[2]);
        *store.special.borrow_mut() = Some(if store.bag_phase {
            value
        } else {
            let weak_store = Rc::downgrade(&store);
            OptionValue::Object(OptionObject::with_fallible_properties(
                value,
                |_| panic!("bag must not coerce"),
                move |value, key| {
                    assert_eq!(key, "colorSchemeQuery");
                    weak_store.upgrade().unwrap().trigger()?;
                    Ok(value.clone())
                },
            ))
        });
        let events = Rc::new(RefCell::new(Vec::new()));
        let event_output = Rc::clone(&events);
        let output = Rc::clone(&records);
        let action = fields[3].to_owned();
        let captured = original.clone();
        let first_event = Cell::new(true);
        terminal.set_event_handler(Some(Rc::new(move |event| {
            let text = match &event {
                TerminalEvent::Resize { cols, rows } => Some(format!("Z{cols}:{rows}")),
                TerminalEvent::ColorSchemeQuery => Some("Q".into()),
                event => projection::event(event),
            };
            if let Some(text) = text {
                event_output.borrow_mut().push(text);
            }
            if matches!(event, TerminalEvent::ColorSchemeQuery) {
                let shared = event_state.upgrade().unwrap();
                let text = shared
                    .try_borrow()
                    .map(|state| format!("event:{}", projection::snapshot_state(&state)))
                    .map_err(|_| EmitError::callback(TerminalError::Borrowed))?;
                output.borrow_mut().push(text);
                if first_event.replace(false) {
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
        let units = projection::parse_hex("1b,5b,3f,39,39,36,3b,36,6e,1b,5b,3f,36,6e,5a");
        let result = terminal.write(&String::from_utf16(&units).unwrap());
        let valid_result = match &result {
            Ok(_) => fields[5] == "0",
            Err(error) => fields[5] == "1" && has_original(error, &original),
        };
        let event_text = events.borrow().join("/");
        let event_text = if event_text.is_empty() {
            "-"
        } else {
            &event_text
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
        if event_text != fields[8] {
            mismatches.push(format!(
                "ordered events actual={event_text:?} expected={:?}",
                fields[8]
            ));
        }
        if !mismatches.is_empty() {
            failures.push(format!(
                "case {index} {} alt={} {} {}: {}",
                fields[0],
                fields[1],
                fields[2],
                fields[3],
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

#[test]
fn unchanged_headless_on_data_dsr_assertion() {
    let mut terminal = CoreTerminal::new(80, 24, 1000).unwrap();
    terminal.write("\x1b[5n").unwrap();
    let calls: Vec<_> = terminal
        .drain_events()
        .into_iter()
        .filter_map(|event| {
            if let TerminalEvent::Data(data) = event {
                Some(String::from_utf16(&data).unwrap())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(calls, ["\x1b[0n"]);
}
