//! Actual Rust terminal parser integration, separate from registration fixtures.
use std::{cell::RefCell, rc::Rc};
use xterm_addons::{
    native_status::NativeStatusTerminal,
    program_status::ProgramStatusAddon,
    progress::{ProgressAddon, ProgressState},
};
use xterm_core::{
    event::EmitError,
    lifecycle::Disposable,
    options::OptionValue,
    parser::{escape::ParseError, handler::DispatchError},
    terminal::{CoreTerminal, TerminalError, TerminalEvent},
};

type Engine = Rc<RefCell<CoreTerminal>>;
fn setup() -> (Engine, Rc<NativeStatusTerminal>, ProgramStatusAddon) {
    let engine = Rc::new(RefCell::new(CoreTerminal::new(80, 24, 1000).unwrap()));
    let adapter = Rc::new(NativeStatusTerminal::new(engine.clone()));
    let addon = ProgramStatusAddon::new();
    addon.activate(adapter.clone()).unwrap();
    (engine, adapter, addon)
}
fn replies(engine: &Engine) -> Vec<String> {
    engine
        .borrow_mut()
        .drain_events()
        .into_iter()
        .filter_map(|e| match e {
            TerminalEvent::Data(data) => Some(String::from_utf16(&data).unwrap()),
            _ => None,
        })
        .collect()
}
fn text(engine: &Engine) -> String {
    let engine = engine.borrow();
    let state = engine.state();
    let buffer = state.active_buffer();
    let buffer = buffer.borrow();
    let line = buffer.lines.get(buffer.ybase).unwrap();
    let text = line.borrow().translate_to_string(true, None, None, None);
    String::from_utf16(&text).unwrap()
}
#[test]
fn queries_use_st_and_precede_following_da1_without_printing() {
    for terminator in ["\x07", "\x1b\\"] {
        let (engine, _adapter, addon) = setup();
        engine
            .borrow_mut()
            .write(&format!("\x1b]7501;?{terminator}\x1b[c"))
            .unwrap();
        assert_eq!(replies(&engine), ["\x1b]7501;?\x1b\\", "\x1b[?1;2c"]);
        assert!(addon.records().is_empty());
        assert_eq!(text(&engine), "");
    }
}
#[test]
fn report_chunking_bel_st_and_abort_follow_actual_parser() {
    let input = "\x1b]7501;state=working:app=cargo\x1b\\\x1b]7501;state=blocked:id=a\x07";
    for split in 0..=input.len() {
        let (engine, _adapter, addon) = setup();
        engine
            .borrow_mut()
            .write_bytes(&input.as_bytes()[..split])
            .unwrap();
        engine
            .borrow_mut()
            .write_bytes(&input.as_bytes()[split..])
            .unwrap();
        let records = addon.records();
        assert_eq!(records.len(), 2, "split {split}");
        assert_eq!(records[1].effective_app.as_deref(), Some("cargo"));
        assert_eq!(text(&engine), "");
    }
    let (engine, _adapter, addon) = setup();
    engine
        .borrow_mut()
        .write("\x1b]7501;state=working\x18")
        .unwrap();
    assert!(addon.records().is_empty());
}
#[test]
fn disabled_stdin_suppresses_feature_reply() {
    let (engine, _adapter, _addon) = setup();
    engine
        .borrow()
        .state()
        .options
        .set("disableStdin", OptionValue::Bool(true))
        .unwrap();
    engine.borrow_mut().write("\x1b]7501;?\x07").unwrap();
    assert!(replies(&engine).is_empty());
}
#[test]
fn ris_resets_terminal_and_addon_api_reset_retains_records() {
    let (engine, _adapter, addon) = setup();
    engine
        .borrow_mut()
        .write("\x1b]7501;state=working\x07hello")
        .unwrap();
    assert_eq!(text(&engine), "hello");
    engine.borrow_mut().reset().unwrap();
    assert_eq!(addon.records().len(), 1);
    engine.borrow_mut().write("hello\x1bc").unwrap();
    assert!(addon.records().is_empty());
    assert_eq!(text(&engine), "");
    let active = engine.borrow().state().active_buffer();
    assert_eq!(active.borrow().x, 0);
    engine
        .borrow_mut()
        .write("\x1b]7501;state=done\x07")
        .unwrap();
    assert_eq!(addon.records().len(), 1);
}
#[test]
fn soft_reset_and_alternate_screen_retain_records() {
    let (engine, _adapter, addon) = setup();
    engine.borrow_mut().write("\x1b]7501;state=working\x07\x1b[!p\x1b[?1049h\x1b]7501;state=blocked:id=pager\x07\x1b[?1049l").unwrap();
    assert_eq!(
        addon
            .records()
            .iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>(),
        ["", "pager"]
    );
}
#[test]
fn disposal_unregisters_live_handlers() {
    let (engine, _adapter, addon) = setup();
    engine
        .borrow_mut()
        .write("\x1b]7501;state=working\x07")
        .unwrap();
    addon.dispose();
    engine
        .borrow_mut()
        .write("\x1b]7501;state=done\x07\x1b]7501;?\x07")
        .unwrap();
    assert!(addon.records().is_empty());
    assert!(replies(&engine).is_empty());
}
#[test]
fn progress_and_status_cooperate_on_live_stream() {
    let (engine, adapter, addon) = setup();
    let progress = ProgressAddon::new();
    progress.activate(adapter);
    engine
        .borrow_mut()
        .write("\x1b]9;4;1;40\x1b\\\x1b]7501;state=blocked:progress=40\x07")
        .unwrap();
    assert_eq!(
        progress.progress(),
        ProgressState {
            state: 1.,
            value: 40.
        }
    );
    assert_eq!(addon.records()[0].progress, Some(40));
    assert_eq!(text(&engine), "");
}
#[test]
fn native_public_load_addon_uses_live_engine_and_releases_terminal_owner() {
    let terminal = Rc::new(xterm_core::headless::Terminal::new(80, 24, 1000).unwrap());
    let weak_terminal = Rc::downgrade(&terminal);
    let status = Rc::new(ProgramStatusAddon::new());
    let progress = Rc::new(ProgressAddon::new());
    terminal.load_addon(status.clone()).unwrap();
    terminal.load_addon(progress.clone()).unwrap();
    let replies = Rc::new(RefCell::new(Vec::new()));
    let capture = replies.clone();
    terminal
        .on_data()
        .subscribe(move |data| capture.borrow_mut().push(data.clone()));
    terminal
        .write("\x1b]7501;state=done\x07\x1b]9;4;1;40\x07\x1b]7501;?\x07")
        .unwrap();
    assert_eq!(status.records().len(), 1);
    assert_eq!(
        progress.progress(),
        ProgressState {
            state: 1.,
            value: 40.
        }
    );
    assert_eq!(
        *replies.borrow(),
        vec!["\x1b]7501;?\x1b\\".encode_utf16().collect::<Vec<_>>()]
    );
    terminal.dispose().unwrap();
    assert!(status.records().is_empty());
    drop(terminal);
    assert!(weak_terminal.upgrade().is_none());
}

#[test]
fn public_load_addon_during_callback_uses_cached_registrars() {
    let terminal = Rc::new(xterm_core::headless::Terminal::new(80, 24, 1000).unwrap());
    let first = Rc::new(ProgramStatusAddon::new());
    let second = Rc::new(ProgramStatusAddon::new());
    terminal.load_addon(first.clone()).unwrap();
    let weak = Rc::downgrade(&terminal);
    let capture = second.clone();
    first.on_change().try_subscribe(move |_| {
        weak.upgrade()
            .unwrap()
            .load_addon(capture.clone())
            .map_err(EmitError::callback)?;
        Ok(())
    });
    terminal
        .write("\x1b]7501;state=working\x07\x1b]7501;state=done:id=second\x07")
        .unwrap();
    assert_eq!(first.records().len(), 1);
    assert_eq!(second.records()[0].id, "second");
}

#[test]
fn activation_during_callback_registers_without_borrowing_engine() {
    let (engine, adapter, addon) = setup();
    let second = Rc::new(ProgramStatusAddon::new());
    let captured = second.clone();
    addon
        .on_change()
        .subscribe(move |_| captured.activate(adapter.clone()).unwrap());
    engine
        .borrow_mut()
        .write("\x1b]7501;state=working\x07\x1b]7501;state=done:id=second\x07")
        .unwrap();
    assert_eq!(addon.records().len(), 1);
    assert_eq!(second.records()[0].id, "second");
}

#[test]
fn reply_failure_preserves_original_error_and_stops_dispatch() {
    let (engine, adapter, _addon) = setup();
    engine
        .borrow()
        .state()
        .core
        .on_data()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    let error = engine
        .borrow_mut()
        .write("\x1b]7501;?\x07must-not-print")
        .unwrap_err();
    assert!(matches!(
        error,
        TerminalError::Parse(ParseError::Dispatch(DispatchError::Callback(_)))
    ));
    let input_error = adapter.take_input_error().unwrap();
    assert!(matches!(
        input_error.callback_ref::<xterm_core::core_service::CoreError>(),
        Some(xterm_core::core_service::CoreError::Dispatch(
            EmitError::ListenerArrayCleared
        ))
    ));
    assert_eq!(text(&engine), "");
}

#[test]
fn listener_failure_stops_before_following_text_without_rolling_back_record() {
    let (engine, _adapter, addon) = setup();
    addon
        .on_change()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    let error = engine
        .borrow_mut()
        .write("\x1b]7501;state=working\x07must-not-print")
        .unwrap_err();
    assert!(matches!(
        error,
        TerminalError::Parse(ParseError::Dispatch(DispatchError::Callback(
            EmitError::ListenerArrayCleared
        )))
    ));
    assert_eq!(addon.records().len(), 1);
    assert_eq!(text(&engine), "");
}
