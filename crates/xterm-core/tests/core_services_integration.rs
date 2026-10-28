//! Core/log integration through the real parser, active buffers and terminal API.
use std::cell::{Cell, RefCell};
use std::error::Error as _;
use std::rc::Rc;
use xterm_core::core_service::{CoreError, KittyKeyboardState};
use xterm_core::log_service::{CustomLogger, LogError, LogLevel, LogRecord, LogSink};
use xterm_core::options::{OptionValue, OptionsService};
use xterm_core::terminal::{CoreTerminal, TerminalError, TerminalEvent};

struct Sink(Box<dyn Fn(LogRecord) -> Result<(), LogError>>);
impl LogSink for Sink {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        (self.0)(record)
    }
}
fn text(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}
fn options() -> Rc<OptionsService> {
    Rc::new(OptionsService::new(
        false,
        [
            ("cols", 8.0.into()),
            ("rows", 3.0.into()),
            ("scrollback", 20.0.into()),
            ("logLevel", "trace".into()),
            ("showCursorImmediately", true.into()),
        ],
    ))
}

#[test]
fn actual_scrollback_input_orders_scroll_user_and_data_with_live_options() {
    let options = options();
    let mut terminal = CoreTerminal::with_options(Rc::clone(&options)).unwrap();
    terminal.write("0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6").unwrap();
    let active = terminal.state().active_buffer();
    let core = Rc::clone(&terminal.state().core);
    let base = active.borrow().ybase;
    assert!(base > 0);
    terminal.scroll_to_top().unwrap();
    assert_eq!(active.borrow().ydisp, 0);
    terminal.drain_events();
    terminal.drain_logs();
    let observed = Rc::new(RefCell::new(vec![]));
    let output = Rc::clone(&observed);
    let buffer = Rc::clone(&active);
    core.on_user_input()
        .subscribe(move |_| output.borrow_mut().push(buffer.borrow().ydisp));
    let units = vec![0, 0xd800, 0xd83d, 0xde00];
    terminal.input(&units, true).unwrap();
    assert_eq!(*observed.borrow(), [base]);
    assert_eq!(
        terminal.drain_events(),
        [
            TerminalEvent::Scroll(base),
            TerminalEvent::Data(units.clone())
        ]
    );
    assert_eq!(active.borrow().ydisp, base);
    let logs = terminal.drain_logs();
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].level, LogLevel::Debug);
    assert_eq!(logs[1].level, LogLevel::Trace);
    options.set("scrollOnUserInput", false.into()).unwrap();
    terminal.scroll_to_top().unwrap();
    terminal.drain_events();
    terminal.input(&units, true).unwrap();
    assert_eq!(active.borrow().ydisp, 0);
    assert_eq!(terminal.drain_events(), [TerminalEvent::Data(units)]);
    assert_eq!(*observed.borrow(), [base, 0]);
}

#[test]
fn actual_input_and_binary_forwarding_obey_disable_stdin_and_user_origin() {
    let options = options();
    let mut terminal = CoreTerminal::with_options(Rc::clone(&options)).unwrap();
    terminal.write("\r\n\r\n\r\n\r\n").unwrap();
    terminal.scroll_to_top().unwrap();
    terminal.drain_events();
    terminal.drain_logs();
    let core = Rc::clone(&terminal.state().core);
    let observed = Rc::new(Cell::new(0));
    let output = Rc::clone(&observed);
    core.on_user_input()
        .subscribe(move |_| output.set(output.get() + 1));
    terminal.input(&[0xdfff], false).unwrap();
    core.trigger_binary_event(&[0, 255, 0xd800]).unwrap();
    assert_eq!(
        terminal.drain_events(),
        [
            TerminalEvent::Data(vec![0xdfff]),
            TerminalEvent::Binary(vec![0, 255, 0xd800])
        ]
    );
    assert_eq!(observed.get(), 0);
    assert_eq!(terminal.state().active_buffer().borrow().ydisp, 0);
    terminal.drain_logs();
    options.set("disableStdin", true.into()).unwrap();
    terminal.input(&[], true).unwrap();
    core.trigger_binary_event(&[42]).unwrap();
    assert!(terminal.drain_events().is_empty());
    assert!(terminal.drain_logs().is_empty());
    assert_eq!(observed.get(), 0);
    assert_eq!(terminal.state().active_buffer().borrow().ydisp, 0);
    options.set("disableStdin", false.into()).unwrap();
    terminal.input(&[], false).unwrap();
    assert_eq!(terminal.drain_events(), [TerminalEvent::Data(vec![])]);
}

#[test]
fn parser_modes_and_terminal_reset_use_the_same_shared_core_state() {
    let mut terminal = CoreTerminal::with_options(options()).unwrap();
    let core = Rc::clone(&terminal.state().core);
    assert!(core.is_cursor_initialized.get());
    terminal
        .write("\x1b[4h\x1b[?1h\x1b[?2004h\x1b[?25l")
        .unwrap();
    assert!(core.modes.borrow().insert_mode);
    assert!(core.dec_private_modes.borrow().application_cursor_keys);
    assert!(core.dec_private_modes.borrow().bracketed_paste_mode);
    assert!(core.is_cursor_hidden.get());
    *core.kitty_keyboard.borrow_mut() = KittyKeyboardState {
        flags: 7,
        main_flags: 8,
        alt_flags: 9,
        main_stack: vec![10],
        alt_stack: vec![11],
    };
    terminal.drain_events();
    terminal.reset().unwrap();
    assert!(!core.modes.borrow().insert_mode);
    assert!(!core.dec_private_modes.borrow().application_cursor_keys);
    assert!(!core.dec_private_modes.borrow().bracketed_paste_mode);
    assert!(core.dec_private_modes.borrow().wraparound);
    assert_eq!(*core.kitty_keyboard.borrow(), KittyKeyboardState::default());
    assert!(core.is_cursor_initialized.get());
    assert!(core.is_cursor_hidden.get());
    // Public CoreTerminal.reset performs the reset itself. Only the parser's
    // full-reset escape requests a reset from its terminal owner.
    assert!(!terminal
        .drain_events()
        .contains(&TerminalEvent::RequestReset));
    terminal.input(&text("after reset"), false).unwrap();
    assert_eq!(
        terminal.drain_events(),
        [TerminalEvent::Data(text("after reset"))]
    );
}

#[test]
fn parser_replies_and_logger_callbacks_can_observe_engine_without_borrow_panics() {
    let options = options();
    let mut terminal = CoreTerminal::with_options(Rc::clone(&options)).unwrap();
    let state = terminal.state_handle();
    let core = Rc::clone(&terminal.state().core);
    let observed = Rc::new(RefCell::new(vec![]));
    let output = Rc::clone(&observed);
    let weak = Rc::downgrade(&state);
    core.on_data().subscribe(move |data| {
        let state = weak.upgrade().unwrap();
        let state = state
            .try_borrow()
            .expect("no state borrow across data callbacks");
        output
            .borrow_mut()
            .push((data.clone(), state.active_buffer().borrow().x));
    });
    let records = Rc::new(RefCell::new(vec![]));
    let capture = Rc::clone(&records);
    let weak = Rc::downgrade(&state);
    let sink: Rc<dyn LogSink> = Rc::new(Sink(Box::new(move |record| {
        let state = weak.upgrade().unwrap();
        let state = state
            .try_borrow()
            .expect("no state borrow across logger callbacks");
        capture
            .borrow_mut()
            .push((record, state.active_buffer().borrow().x));
        Ok(())
    })));
    options
        .set("logger", OptionValue::object(CustomLogger(sink)))
        .unwrap();
    terminal.write("ab\x1b[6n").unwrap();
    assert_eq!(observed.borrow()[0], (text("\x1b[1;3R"), 2));
    assert!(!records.borrow().is_empty());
    assert!(records.borrow().iter().all(|(_, x)| *x == 2));
    assert!(terminal.drain_logs().is_empty());
    options.set("logLevel", "off".into()).unwrap();
    let count = records.borrow().len();
    terminal.input(&[0xd800], false).unwrap();
    assert_eq!(records.borrow().len(), count);
}

#[test]
fn opaque_logger_errors_reach_the_terminal_caller_without_data_dispatch() {
    let options = options();
    let mut terminal = CoreTerminal::with_options(Rc::clone(&options)).unwrap();
    let original: Rc<dyn std::error::Error> = Rc::new(std::io::Error::other("host failure"));
    let retained = Rc::clone(&original);
    let sink: Rc<dyn LogSink> = Rc::new(Sink(Box::new(move |_| {
        Err(LogError::Host(Rc::clone(&retained)))
    })));
    options
        .set("logger", OptionValue::object(CustomLogger(sink)))
        .unwrap();
    let error = terminal.input(&[], false).unwrap_err();
    let root = error.source().unwrap().source().unwrap().source().unwrap();
    assert!(std::ptr::eq(root, original.as_ref()));
    let TerminalError::Core(CoreError::Log(LogError::Host(actual))) = error else {
        panic!("original failure lost");
    };
    assert!(Rc::ptr_eq(&actual, &original));
    assert!(terminal.drain_events().is_empty());
}

#[test]
fn disposal_disconnects_retained_services_and_drop_releases_engine_state() {
    let options = options();
    let mut terminal = CoreTerminal::with_options(Rc::clone(&options)).unwrap();
    let core = Rc::clone(&terminal.state().core);
    let logger = Rc::clone(&terminal.state().logger);
    let state = Rc::downgrade(&terminal.state_handle());
    let called = Rc::new(Cell::new(0));
    let output = Rc::clone(&called);
    core.on_data()
        .subscribe(move |_| output.set(output.get() + 1));
    terminal.dispose().unwrap();
    terminal.dispose().unwrap();
    assert!(terminal.is_disposed());
    assert!(matches!(
        terminal.input(&[], true),
        Err(TerminalError::Disposed)
    ));
    core.trigger_data_event(&[42], true).unwrap();
    core.trigger_binary_event(&[255]).unwrap();
    assert_eq!(called.get(), 0);
    assert!(terminal.drain_events().is_empty());
    options.set("logLevel", "off".into()).unwrap();
    assert_eq!(logger.level(), Some(LogLevel::Trace));
    drop(terminal);
    assert!(state.upgrade().is_none());
}
