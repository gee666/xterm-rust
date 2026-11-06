use std::{cell::RefCell, rc::Rc};
use xterm_core::parser::{escape::FunctionIdentifier, handler::HandlerResult};
use xterm_core::terminal::mark_dirty_state;
use xterm_core::{
    headless::Terminal,
    terminal::{AttributeView, TerminalError},
};

#[test]
fn retained_attributes_follow_sgr_then_freeze_on_hard_and_soft_reset() {
    let terminal = Terminal::new(10, 4, 100).unwrap();
    let state = terminal.state_handle_for_addon();
    let original = AttributeView::capture(state.clone()).unwrap();
    terminal.write("\x1b[31;44;7m").unwrap();
    let same = AttributeView::capture(state.clone()).unwrap();
    assert!(original.same_object(&same));
    let old = original.read().unwrap();
    assert_eq!(old.get_fg_color(), 1);
    assert_eq!(old.get_bg_color(), 4);
    assert_ne!(old.is_inverse(), 0);
    terminal.reset().unwrap();
    let reset = AttributeView::capture(state.clone()).unwrap();
    assert!(!original.same_object(&reset));
    assert_eq!(original.read().unwrap().fg, old.fg);
    assert_eq!(original.read().unwrap().bg, old.bg);
    assert!(reset.read().unwrap().is_attribute_default());
    terminal.write("\x1b[38;2;18;52;86m").unwrap();
    assert_eq!(reset.read().unwrap().get_fg_color(), 0x123456);
    terminal.write("\x1b[!p").unwrap();
    let soft = AttributeView::capture(state).unwrap();
    assert!(!reset.same_object(&soft));
    assert_eq!(reset.read().unwrap().get_fg_color(), 0x123456);
    assert!(soft.read().unwrap().is_attribute_default());
}

#[test]
fn internal_dirty_marks_change_the_real_parser_render_range() {
    let terminal = Terminal::new(10, 4, 100).unwrap();
    terminal.write("\x1b[2;1H").unwrap();
    let renders = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&renders);
    let _listener = terminal
        .on_render()
        .subscribe(move |value| output.borrow_mut().push(*value));
    let state = terminal.state_handle_for_addon();
    terminal
        .with_parser(|api| {
            api.register_dcs_handler(
                FunctionIdentifier {
                    prefix: None,
                    intermediates: None,
                    final_byte: "z",
                },
                move |_, _| {
                    mark_dirty_state(&state, 3).unwrap();
                    assert_eq!(state.borrow().pending_dirty_range().unwrap(), (1, 3));
                    HandlerResult::Ready(true)
                },
            )
        })
        .unwrap()
        .unwrap();
    terminal.write("\x1bPz\x1b\\").unwrap();
    assert_eq!(
        *renders.borrow(),
        vec![xterm_core::headless::RenderEvent { start: 1, end: 3 }]
    );
}

#[test]
fn attribute_views_report_borrows_and_replaced_views_need_no_state_borrow() {
    let terminal = Terminal::new(10, 4, 100).unwrap();
    let state = terminal.state_handle_for_addon();
    let original = AttributeView::capture(state.clone()).unwrap();
    {
        let _borrow = state.borrow_mut();
        assert!(matches!(original.read(), Err(TerminalError::Borrowed)));
        assert!(matches!(
            AttributeView::capture(state.clone()),
            Err(TerminalError::Borrowed)
        ));
    }
    terminal.reset().unwrap();
    let _borrow = state.borrow_mut();
    assert!(original.read().unwrap().is_attribute_default());
}
