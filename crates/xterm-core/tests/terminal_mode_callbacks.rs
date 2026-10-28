// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Observable callbacks must see source-ordered live state without held borrows.
use std::{cell::RefCell, error::Error, fmt, rc::Rc};
use xterm_core::{
    headless::Terminal,
    log_service::{CustomLogger, LogError, LogRecord, LogSink},
    options::{CoercionError, OptionObject, OptionValue as Value, OptionsService},
};

fn terminal() -> Rc<Terminal> {
    Rc::new(Terminal::new(80, 30, 1000).unwrap())
}
fn options(t: &Terminal) -> Rc<OptionsService> {
    t.with_state(|s| Rc::clone(&s.options)).unwrap()
}
fn replies(t: &Terminal) -> Rc<RefCell<Vec<String>>> {
    let output = Rc::new(RefCell::new(Vec::new()));
    let target = Rc::clone(&output);
    t.on_data()
        .subscribe(move |data| target.borrow_mut().push(String::from_utf16(data).unwrap()));
    output
}

#[test]
fn ansi_option_observer_reads_state_before_the_next_parameter() {
    let t = terminal();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&seen);
    let weak = Rc::downgrade(&t);
    options(&t)
        .on_specific_option_change("convertEol", move |_| {
            let terminal = weak.upgrade().unwrap();
            output.borrow_mut().push(
                terminal
                    .with_state(|s| s.core.modes.borrow().insert_mode)
                    .unwrap(),
            );
        })
        .unwrap();
    t.write("\x1b[20;4h").unwrap();
    assert_eq!(*seen.borrow(), [false]);
    assert!(t.with_state(|s| s.core.modes.borrow().insert_mode).unwrap());
}
#[test]
fn mouse_protocol_observer_reads_state_before_following_cursor_mode() {
    let t = terminal();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&seen);
    let weak = Rc::downgrade(&t);
    let mouse = t.with_state(|s| Rc::clone(&s.mouse)).unwrap();
    mouse.on_protocol_change().subscribe(move |_| {
        let terminal = weak.upgrade().unwrap();
        output.borrow_mut().push(
            terminal
                .with_state(|s| s.core.dec_private_modes.borrow().application_cursor_keys)
                .unwrap(),
        );
    });
    t.write("\x1b[?9;1h").unwrap();
    assert_eq!(*seen.borrow(), [false]);
    assert!(t
        .with_state(|s| s.core.dec_private_modes.borrow().application_cursor_keys)
        .unwrap());
}
#[test]
fn private_query_keeps_the_old_mode_object_when_getter_resets_core() {
    let t = terminal();
    let output = replies(&t);
    let core = t.with_state(|s| Rc::clone(&s.core)).unwrap();
    core.dec_private_modes.borrow_mut().win32_input_mode = true;
    options(&t)
        .set(
            "vtExtensions",
            Value::Object(OptionObject::with_properties(
                Rc::downgrade(&core),
                |_| Ok(Value::from("[object Object]")),
                |core, key| {
                    if key == "win32InputMode" {
                        let core = core
                            .upgrade()
                            .expect("terminal retains core while querying options");
                        core.reset();
                        Ok(Value::Bool(true))
                    } else {
                        Ok(Value::Undefined)
                    }
                },
            )),
        )
        .unwrap();
    t.write("\x1b[?9001$p").unwrap();
    assert_eq!(*output.borrow(), ["\x1b[?9001;1$y"]);
    assert!(!core.dec_private_modes.borrow().win32_input_mode);
}
#[test]
fn private_query_sees_in_place_mode_mutation_by_the_getter() {
    let t = terminal();
    let output = replies(&t);
    let core = t.with_state(|s| Rc::clone(&s.core)).unwrap();
    options(&t)
        .set(
            "vtExtensions",
            Value::Object(OptionObject::with_properties(
                Rc::downgrade(&core),
                |_| Ok(Value::from("[object Object]")),
                |core, key| {
                    if key == "win32InputMode" {
                        let core = core
                            .upgrade()
                            .expect("terminal retains core while querying options");
                        core.dec_private_modes.borrow_mut().win32_input_mode = true;
                        Ok(Value::Bool(true))
                    } else {
                        Ok(Value::Undefined)
                    }
                },
            )),
        )
        .unwrap();
    t.write("\x1b[?9001$p").unwrap();
    assert_eq!(*output.borrow(), ["\x1b[?9001;1$y"]);
    assert!(core.dec_private_modes.borrow().win32_input_mode);
}
#[test]
fn private_column_query_keeps_numeric_columns_captured_before_getter() {
    let t = terminal();
    let output = replies(&t);
    let service = t.with_state(|s| Rc::clone(&s.buffer_service)).unwrap();
    options(&t)
        .set(
            "windowOptions",
            Value::Object(OptionObject::with_properties(
                Rc::downgrade(&service),
                |_| Ok(Value::from("[object Object]")),
                |service, key| {
                    if key == "setWinLines" {
                        let service = service
                            .upgrade()
                            .expect("terminal retains buffer service during query");
                        service
                            .resize(132, service.rows())
                            .map_err(|error| CoercionError::new(&error.to_string()))?;
                        Ok(Value::Bool(true))
                    } else {
                        Ok(Value::Undefined)
                    }
                },
            )),
        )
        .unwrap();
    t.write("\x1b[?3$p").unwrap();
    assert_eq!(*output.borrow(), ["\x1b[?3;2$y"]);
    assert_eq!(t.cols().unwrap(), 132);
}
#[test]
fn throwing_gate_keeps_prior_parameters_and_stops_later_parameters() {
    let t = terminal();
    let core = t.with_state(|s| Rc::clone(&s.core)).unwrap();
    core.dec_private_modes.borrow_mut().wraparound = false;
    options(&t)
        .set(
            "vtExtensions",
            Value::Object(OptionObject::with_properties(
                (),
                |_| Ok(Value::from("[object Object]")),
                |_, key| {
                    if key == "win32InputMode" {
                        Err(CoercionError::new("denied gate"))
                    } else {
                        Ok(Value::Undefined)
                    }
                },
            )),
        )
        .unwrap();
    assert!(t
        .write("\x1b[?1;9001;7h")
        .unwrap_err()
        .to_string()
        .contains("denied gate"));
    let modes = core.dec_private_modes.borrow();
    assert!(modes.application_cursor_keys);
    assert!(!modes.win32_input_mode);
    assert!(!modes.wraparound);
}

#[test]
fn disabled_stdin_suppresses_mode_replies_but_keeps_incoming_mutations() {
    let t = terminal();
    let output = replies(&t);
    options(&t).set("disableStdin", Value::Bool(true)).unwrap();
    t.write("A\x1b[?1h\x1b[?1$pB\x1b[4h\x1b[4$pC").unwrap();
    assert!(output.borrow().is_empty());
    assert!(t
        .with_state(|s| s.core.dec_private_modes.borrow().application_cursor_keys)
        .unwrap());
    assert!(t.with_state(|s| s.core.modes.borrow().insert_mode).unwrap());
    let text = t
        .with_state(|s| {
            s.active_buffer()
                .borrow()
                .translate_buffer_line_to_string(0, true, None, None)
                .unwrap()
        })
        .unwrap();
    assert_eq!(String::from_utf16(&text).unwrap(), "ABC");
    options(&t).set("disableStdin", Value::Bool(false)).unwrap();
    t.write("\x1b[?1$p\x1b[4$p").unwrap();
    assert_eq!(*output.borrow(), ["\x1b[?1;1$y", "\x1b[4;1$y"]);
}

#[derive(Debug)]
struct LoggerFailure;
impl fmt::Display for LoggerFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("keypad logger rejected")
    }
}
impl Error for LoggerFailure {}
type Observations = Rc<RefCell<Vec<(bool, bool)>>>;
struct InspectLogger {
    terminal: std::rc::Weak<Terminal>,
    observations: Observations,
    fail: bool,
}
impl LogSink for InspectLogger {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        if String::from_utf16(&record.message).unwrap()
            != "Serial port requested application keypad."
        {
            return Ok(());
        }
        let t = self.terminal.upgrade().unwrap();
        self.observations.borrow_mut().push(
            t.with_state(|s| {
                let modes = s.core.dec_private_modes.borrow();
                (modes.application_keypad, modes.application_cursor_keys)
            })
            .unwrap(),
        );
        if self.fail {
            Err(LogError::Host(Rc::new(LoggerFailure)))
        } else {
            Ok(())
        }
    }
}
fn logger_case(fail: bool) -> (Rc<Terminal>, Observations) {
    let t = terminal();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let logger = CustomLogger(Rc::new(InspectLogger {
        terminal: Rc::downgrade(&t),
        observations: Rc::clone(&observations),
        fail,
    }));
    options(&t).set("logger", Value::object(logger)).unwrap();
    options(&t).set("logLevel", Value::from("debug")).unwrap();
    (t, observations)
}
#[test]
fn logger_runs_before_keypad_mutation_and_before_following_parameters() {
    let (t, observations) = logger_case(false);
    t.write("\x1b[?66;1h").unwrap();
    assert_eq!(*observations.borrow(), [(false, false)]);
    assert!(t
        .with_state(|s| s.core.dec_private_modes.borrow().application_keypad)
        .unwrap());
    assert!(t
        .with_state(|s| s.core.dec_private_modes.borrow().application_cursor_keys)
        .unwrap());
}
#[test]
fn logger_failure_leaves_keypad_and_following_parameters_unmodified() {
    let (t, observations) = logger_case(true);
    let error = t.write("\x1b[?66;1h").unwrap_err();
    assert_eq!(error.to_string(), "keypad logger rejected");
    assert_eq!(*observations.borrow(), [(false, false)]);
    assert!(!t
        .with_state(|s| s.core.dec_private_modes.borrow().application_keypad)
        .unwrap());
    assert!(!t
        .with_state(|s| s.core.dec_private_modes.borrow().application_cursor_keys)
        .unwrap());
    let mut cause: &(dyn Error + 'static) = &error;
    while let Some(next) = cause.source() {
        cause = next;
    }
    assert!(cause.is::<LoggerFailure>());
}
