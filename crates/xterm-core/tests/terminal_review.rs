//! Independent current-engine callback-borrow checks for task149.
use std::{cell::Cell, rc::Rc};
use xterm_core::terminal::CoreTerminal;

#[test]
fn resize_observer_can_read_live_terminal_state() {
    let mut terminal = CoreTerminal::new(10, 3, 10).unwrap();
    let shared = terminal.state_handle();
    let service = Rc::clone(&shared.borrow().buffer_service);
    let weak = Rc::downgrade(&shared);
    let observed = Rc::new(Cell::new(None));
    let result = Rc::clone(&observed);
    service.on_resize().subscribe(move |_| {
        let shared = weak.upgrade().unwrap();
        result.set(Some(shared.try_borrow().is_ok()));
    });
    terminal.resize(12, 4).unwrap();
    assert_eq!(
        observed.get(),
        Some(true),
        "resize held terminal state across a public service callback"
    );
}

#[test]
fn dispose_marker_observer_can_mutate_live_terminal_state() {
    let mut terminal = CoreTerminal::new(10, 3, 10).unwrap();
    let shared = terminal.state_handle();
    let active = shared.borrow().active_buffer();
    let marker = active.borrow_mut().add_marker(0);
    let weak = Rc::downgrade(&shared);
    let observed = Rc::new(Cell::new(None));
    let result = Rc::clone(&observed);
    marker.on_dispose().subscribe(move |_| {
        let shared = weak.upgrade().unwrap();
        result.set(Some(shared.try_borrow_mut().is_ok()));
    });
    terminal.dispose().unwrap();
    assert_eq!(
        observed.get(),
        Some(true),
        "dispose held terminal state across marker disposal"
    );
}

#[test]
fn clear_marker_observer_can_read_live_terminal_state_and_buffer() {
    let mut terminal = CoreTerminal::new(10, 3, 10).unwrap();
    let shared = terminal.state_handle();
    let active = shared.borrow().active_buffer();
    let marker = active.borrow_mut().add_marker(0);
    let weak_state = Rc::downgrade(&shared);
    let weak_buffer = Rc::downgrade(&active);
    let observed = Rc::new(Cell::new(None));
    let result = Rc::clone(&observed);
    marker.on_dispose().subscribe(move |_| {
        let shared = weak_state.upgrade().unwrap();
        let active = weak_buffer.upgrade().unwrap();
        result.set(Some(
            shared.try_borrow().is_ok() && active.try_borrow().is_ok(),
        ));
    });
    terminal.clear().unwrap();
    assert_eq!(
        observed.get(),
        Some(true),
        "clear held state/buffer across marker disposal"
    );
}
