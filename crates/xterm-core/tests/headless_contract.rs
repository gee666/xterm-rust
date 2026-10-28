use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    headless::Terminal,
    options::OptionValue,
    parser::handler::HandlerResult,
    unicode_addons::{Unicode11Addon, UnicodeGraphemesAddon},
};
fn line(term: &Terminal) -> Vec<u16> {
    let row = term
        .buffer()
        .unwrap()
        .active()
        .unwrap()
        .get_line(0)
        .unwrap()
        .unwrap();
    let text = row.borrow().translate_to_string(true, None, None, None);
    text
}
#[test]
fn pa02_live_options_and_cached_namespace_keep_identity_across_reset() {
    let term = Terminal::new(8, 3, 10).unwrap();
    assert!(std::ptr::eq(term.options(), term.options()));
    let namespace = term.buffer().unwrap();
    let normal = namespace.normal();
    term.write("old").unwrap();
    term.reset().unwrap();
    assert!(Rc::ptr_eq(&namespace, &term.buffer().unwrap()));
    assert!(Rc::ptr_eq(&normal, &namespace.normal()));
    assert!(line(&term).is_empty());
    term.options()
        .set("fontSize", OptionValue::Number(12.0))
        .unwrap();
    assert_eq!(
        term.with_state(|state| state.options.get("fontSize").unwrap())
            .unwrap(),
        OptionValue::Number(12.0)
    );
    term.resize(12, 4).unwrap();
    assert_eq!((term.cols().unwrap(), term.rows().unwrap()), (12, 4));
    for key in ["cols", "rows"] {
        assert!(term.options().set(key, OptionValue::Number(12.0)).is_err());
    }
}
#[test]
fn pa03_direct_reset_keeps_partial_parser_sequence_and_registration() {
    let term = Terminal::new(8, 3, 10).unwrap();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    term.with_parser(|parser| {
        parser.register_osc_handler(999.0, move |text| {
            output.borrow_mut().push(text);
            HandlerResult::Ready(true)
        })
    })
    .unwrap();
    term.write("\x1b]999;A").unwrap();
    term.reset().unwrap();
    term.write("B\x07C").unwrap();
    assert_eq!(
        *calls.borrow(),
        vec!["AB".encode_utf16().collect::<Vec<_>>()]
    );
    assert_eq!(line(&term), "C".encode_utf16().collect::<Vec<_>>());
}
#[test]
fn pa03_byte_and_surrogate_splits_reach_real_cells_without_replacement() {
    let term = Terminal::new(8, 3, 10).unwrap();
    for byte in "文".as_bytes() {
        term.write_bytes(&[*byte]).unwrap();
    }
    term.write_utf16(&[0xd83d]).unwrap();
    term.write_utf16(&[0xde00]).unwrap();
    assert_eq!(line(&term), "文😀".encode_utf16().collect::<Vec<_>>());
}
#[test]
fn pa04_public_input_allows_nested_data_callbacks_without_engine_borrow_panics() {
    let term = Rc::new(Terminal::new(8, 3, 10).unwrap());
    let calls = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&term);
    let output = Rc::clone(&calls);
    term.on_data().subscribe(move |text| {
        output.borrow_mut().push(text.clone());
        if text == &[97] {
            weak.upgrade().unwrap().input(&[98], false).unwrap();
        }
    });
    term.input(&[97], true).unwrap();
    assert_eq!(*calls.borrow(), vec![vec![97], vec![98]]);
    term.options()
        .set("disableStdin", OptionValue::Bool(true))
        .unwrap();
    term.input(&[99], true).unwrap();
    term.input_binary(&[255]).unwrap();
    assert_eq!(calls.borrow().len(), 2);
    assert!(line(&term).is_empty());
}
#[test]
fn pa02_unicode_addons_gate_at_actual_terminal_activation_and_restore_retained_service() {
    let term = Terminal::new(8, 3, 10).unwrap();
    assert!(term.load_addon(Rc::new(Unicode11Addon::new())).is_err());
    assert!(term
        .load_addon(Rc::new(UnicodeGraphemesAddon::new()))
        .is_err());
    term.options()
        .set("allowProposedApi", OptionValue::Bool(true))
        .unwrap();
    let unicode = term.unicode().unwrap();
    assert_eq!(unicode.active_version(), "6");
    let plain = term.load_addon(Rc::new(Unicode11Addon::new())).unwrap();
    assert!(unicode.versions().contains(&"11".into()));
    assert_eq!(unicode.active_version(), "6");
    plain.dispose().unwrap();
    assert!(unicode.versions().contains(&"11".into()));
    let graphemes = term
        .load_addon(Rc::new(UnicodeGraphemesAddon::new()))
        .unwrap();
    assert_eq!(unicode.active_version(), "15-graphemes");
    term.options()
        .set("allowProposedApi", OptionValue::Bool(false))
        .unwrap();
    assert!(term.unicode().is_err());
    graphemes.dispose().unwrap();
    assert_eq!(unicode.active_version(), "6");
}
