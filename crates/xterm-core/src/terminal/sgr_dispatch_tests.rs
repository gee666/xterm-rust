use crate::terminal::{CoreTerminal, TerminalState};
use crate::{
    buffer::{
        constants::{bg_flags, fg_flags},
        AttributeData,
    },
    log_service::{CustomLogger, LogError, LogRecord, LogSink, LogValue},
    options::OptionValue,
    parser::params::{ParamValue, Params},
};
use std::{cell::RefCell, rc::Rc};

type Callback = dyn Fn(LogRecord) -> Result<(), LogError>;
struct CallbackSink(Box<Callback>);
impl LogSink for CallbackSink {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.0(record)
    }
}

fn terminal(
    callback: impl Fn(&Rc<RefCell<TerminalState>>, LogRecord) -> Result<(), LogError> + 'static,
) -> CoreTerminal {
    let terminal = CoreTerminal::new(5, 3, 0).unwrap();
    let state = terminal.state_handle();
    let options = Rc::clone(&state.borrow().options);
    let weak = Rc::downgrade(&state);
    options.set("logLevel", OptionValue::from("debug")).unwrap();
    let sink = Rc::new(CallbackSink(Box::new(move |record| {
        callback(&weak.upgrade().unwrap(), record)
    })));
    options
        .set("logger", OptionValue::object(CustomLogger(sink)))
        .unwrap();
    terminal
}

// InputHandler.parse logs before dispatch. Targeted failure injections below
// act only on the SGR diagnostic, while still asserting the exact parse record.
fn is_parse_log(record: &LogRecord) -> bool {
    let parse: Vec<_> = "parsing data  \"\x1b[1;999;2m\"".encode_utf16().collect();
    if record.message == parse {
        assert!(record.parameters.is_empty());
        true
    } else {
        assert_eq!(
            record.message,
            "Unknown SGR attribute: %d."
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            record.parameters,
            [LogValue::Value(OptionValue::Number(999.0))]
        );
        false
    }
}

#[test]
fn logger_error_observes_prefix_and_stops_later_mutations() {
    let identity: Rc<dyn std::error::Error> =
        Rc::new(std::io::Error::other("injected SGR logger error"));
    let original = Rc::clone(&identity);
    let mut terminal = terminal(move |state, record| {
        if is_parse_log(&record) {
            assert_eq!(
                [state.borrow().attributes.fg, state.borrow().attributes.bg],
                [0, 0]
            );
            return Ok(());
        }
        assert_eq!(state.borrow().attributes.fg, fg_flags::BOLD);
        assert_eq!(state.borrow().attributes.bg, 0);
        assert_eq!(
            record.message,
            "Unknown SGR attribute: %d."
                .encode_utf16()
                .collect::<Vec<_>>()
        );
        assert_eq!(
            record.parameters,
            [LogValue::Value(OptionValue::Number(999.0))]
        );
        Err(LogError::Host(Rc::clone(&original)))
    });
    match terminal.write("\x1b[1;999;2m").unwrap_err() {
        crate::terminal::TerminalError::Log(LogError::Host(actual)) => {
            assert!(Rc::ptr_eq(&actual, &identity))
        }
        error => panic!("original logger error identity was lost: {error:?}"),
    }
    assert_eq!(terminal.state().attributes.fg, fg_flags::BOLD);
    assert_eq!(terminal.state().attributes.bg, 0);
}

#[test]
fn logger_reentrant_attribute_mutation_precedes_outer_tail() {
    let mut terminal = terminal(|state, record| {
        if is_parse_log(&record) {
            return Ok(());
        }
        let params = Params::from_array(&[ParamValue::Param(0), ParamValue::Param(3)]).unwrap();
        crate::terminal::sgr::apply(&mut state.borrow_mut().attributes, &params);
        Ok(())
    });
    terminal.write("\x1b[1;999;2m").unwrap();
    assert_eq!(terminal.state().attributes.fg, 0);
    assert_eq!(
        terminal.state().attributes.bg,
        bg_flags::ITALIC | bg_flags::DIM
    );
}

#[test]
fn nested_mutation_then_reset_preserves_the_original_detached_target() {
    let mut terminal = terminal(|state, record| {
        if is_parse_log(&record) {
            return Ok(());
        }
        assert_eq!(state.borrow().attributes.fg, fg_flags::BOLD);
        let params = Params::from_array(&[ParamValue::Param(3), ParamValue::Param(7)]).unwrap();
        crate::terminal::sgr::apply(&mut state.borrow_mut().attributes, &params);
        crate::terminal::reset_state_handle(state).map_err(|error| LogError::Host(Rc::new(error)))
    });
    let original = Rc::clone(&terminal.state().attributes_identity);
    terminal.write("\x1b[1;999;2m").unwrap();
    let current = terminal.state();
    assert_eq!(current.attributes.fg, 0);
    assert_eq!(current.attributes.bg, 0);
    assert!(!Rc::ptr_eq(&original, &current.attributes_identity));
    let detached = original.borrow();
    let detached = detached.as_ref().unwrap();
    assert_eq!(detached.fg, fg_flags::BOLD | fg_flags::INVERSE);
    assert_eq!(detached.bg, bg_flags::ITALIC | bg_flags::DIM);
}

#[test]
fn parse_debug_reset_happens_before_sgr_captures_its_target() {
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&seen);
    let mut terminal = terminal(move |state, record| {
        let parsing = is_parse_log(&record);
        observed.borrow_mut().push((
            parsing,
            state.borrow().attributes.fg,
            state.borrow().attributes.bg,
        ));
        if parsing {
            let params = Params::from_array(&[ParamValue::Param(3), ParamValue::Param(7)]).unwrap();
            crate::terminal::sgr::apply(&mut state.borrow_mut().attributes, &params);
            crate::terminal::reset_state_handle(state)
                .map_err(|error| LogError::Host(Rc::new(error)))?;
        }
        Ok(())
    });
    let original = Rc::clone(&terminal.state().attributes_identity);
    terminal.write("\x1b[1;999;2m").unwrap();
    assert_eq!(*seen.borrow(), [(true, 0, 0), (false, fg_flags::BOLD, 0)]);
    assert_eq!(
        [
            terminal.state().attributes.fg,
            terminal.state().attributes.bg
        ],
        [fg_flags::BOLD, bg_flags::DIM]
    );
    let old = original.borrow();
    let old = old.as_ref().unwrap();
    assert_eq!([old.fg, old.bg], [fg_flags::INVERSE, bg_flags::ITALIC]);
}

#[test]
fn replacing_target_moves_extended_identity_instead_of_cloning_payload() {
    let mut terminal = CoreTerminal::new(5, 3, 0).unwrap();
    terminal.write("\x1b[4m").unwrap();
    let state = terminal.state_handle();
    let mut state = state.borrow_mut();
    let payload: Rc<dyn std::any::Any> = Rc::new(7_u8);
    state
        .attributes
        .extended
        .set_payload(Some(Rc::clone(&payload)));
    let identity = Rc::clone(&state.attributes_identity);
    let extended = state.attributes.extended.share();
    state.replace_attributes(AttributeData::default());
    let detached = identity.borrow();
    let detached = detached.as_ref().unwrap();
    assert!(detached.extended.same_identity(&extended));
    assert!(Rc::ptr_eq(&detached.extended.payload().unwrap(), &payload));
}
