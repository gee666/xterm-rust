use std::{cell::RefCell, rc::Rc};
use xterm_core::{headless::Terminal, terminal::TerminalEvent};
fn text(term: &Terminal) -> String {
    let row = term
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    let units = row.borrow().translate_to_string(true, None, None, None);
    String::from_utf16(&units).unwrap()
}
#[test]
fn host_reset_runs_before_notification_without_a_second_default_reset() {
    let term = Rc::new(Terminal::new(8, 3, 10).unwrap());
    term.write("old").unwrap();
    let log = Rc::new(RefCell::new(Vec::new()));
    let target = Rc::downgrade(&term);
    let output = Rc::clone(&log);
    let token = Rc::new(());
    let retained = Rc::clone(&token);
    let weak_token = Rc::downgrade(&token);
    term.set_reset_request_handler(Some(Rc::new(move || {
        let _keep = &retained;
        let term = target.upgrade().unwrap();
        output.borrow_mut().push(format!("host:{}", text(&term)));
        term.reset()
            .map_err(xterm_core::event::EmitError::callback)?;
        term.with_state(|state| state.core.modes.borrow_mut().insert_mode = true)
            .map_err(xterm_core::event::EmitError::callback)?;
        Ok(())
    })))
    .unwrap();
    drop(token);
    let output = Rc::clone(&log);
    term.on_event().subscribe(move |event| {
        if *event == TerminalEvent::RequestReset {
            output.borrow_mut().push("notice".into());
        }
    });
    term.write("\x1bcX").unwrap();
    assert_eq!(*log.borrow(), ["host:old", "notice"]);
    assert_eq!(text(&term), "X");
    assert!(term.modes().unwrap().insert_mode);
    term.dispose().unwrap();
    assert!(weak_token.upgrade().is_none());
}
#[test]
fn default_headless_reset_remains_functional_without_a_host_override() {
    let term = Terminal::new(8, 3, 10).unwrap();
    term.write("old\x1bcX").unwrap();
    assert_eq!(text(&term), "X");
}
