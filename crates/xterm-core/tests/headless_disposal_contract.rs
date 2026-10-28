// Expected behavior comes from scripts/headless-disposal-oracle.mjs. A disposed
// public terminal is not a universal error guard in the source implementation.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    headless::{QueuedTerminal, Terminal},
    options::{OptionValue, OptionsService},
    parser::handler::HandlerResult,
};
#[test]
fn marker_disposal_observes_source_service_phases() {
    let options = Rc::new(OptionsService::new(
        false,
        [
            ("cols", OptionValue::Number(8.0)),
            ("rows", OptionValue::Number(3.0)),
            ("allowProposedApi", OptionValue::Bool(true)),
            ("linkHandler", OptionValue::object(())),
            ("documentOverride", OptionValue::object(())),
        ],
    ));
    let term = Rc::new(Terminal::with_options(options).unwrap());
    let host = QueuedTerminal::new(Rc::clone(&term)).unwrap();
    let public = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&public);
    term.on_data()
        .subscribe(move |data| output.borrow_mut().push(data.clone()));
    let core = term.with_state(|state| Rc::clone(&state.core)).unwrap();
    let internal = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&internal);
    core.on_data()
        .subscribe(move |data| output.borrow_mut().push(data.clone()));
    let unicode = term.unicode().unwrap();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&changes);
    unicode
        .on_change()
        .subscribe(move |version| output.borrow_mut().push(version.clone()));
    let refs_cleared = Rc::new(Cell::new(false));
    let output = Rc::clone(&refs_cleared);
    let weak = Rc::downgrade(&term);
    term.register_marker(0)
        .unwrap()
        .on_dispose()
        .subscribe(move |_| {
            let term = weak.upgrade().unwrap();
            output.set(
                term.options().get("linkHandler").unwrap() == OptionValue::Null
                    && term.options().get("documentOverride").unwrap() == OptionValue::Null,
            );
            term.input(&[77], true).unwrap();
            term.unicode().unwrap().set_active_version("6").unwrap();
        });
    let weak = Rc::downgrade(&term);
    term.on_bell()
        .subscribe(move |_| weak.upgrade().unwrap().dispose().unwrap());
    host.write(
        xterm_core::input::write_buffer::WriteData::text("A\x07B"),
        None,
    )
    .unwrap();
    host.queue().flush_sync().unwrap();
    assert!(refs_cleared.get());
    assert!(public.borrow().is_empty());
    assert_eq!(*internal.borrow(), vec![vec![77]]);
    assert_eq!(*changes.borrow(), ["6"]);
    assert!(term.is_disposed().unwrap());
    assert_eq!(text(&term), "AB");
}
#[test]
fn reentrant_disposal_clears_commands_immediately_but_preserves_print_and_later_registrations() {
    for register in [false, true] {
        let term = Rc::new(Terminal::new(8, 3, 10).unwrap());
        let host = QueuedTerminal::new(Rc::clone(&term)).unwrap();
        let calls = Rc::new(Cell::new(0));
        let output = Rc::clone(&calls);
        let target = Rc::downgrade(&term);
        term.on_bell().subscribe(move |_| {
            let term = target.upgrade().unwrap();
            term.dispose().unwrap();
            if register {
                let output = Rc::clone(&output);
                term.with_parser(|api| {
                    api.register_csi_handler(
                        xterm_core::parser::escape::FunctionIdentifier::new("z"),
                        move |_| {
                            output.set(output.get() + 1);
                            HandlerResult::Ready(true)
                        },
                    )
                })
                .unwrap()
                .unwrap();
            }
        });
        let input = if register {
            "A\x07\x1b[zC"
        } else {
            "A\x07B\n\x1bcC"
        };
        host.write(
            xterm_core::input::write_buffer::WriteData::text(input),
            None,
        )
        .unwrap();
        host.queue().flush_sync().unwrap();
        assert_eq!(text(&term), if register { "AC" } else { "ABC" });
        assert_eq!(
            term.buffer().unwrap().active().unwrap().cursor_y().unwrap(),
            0
        );
        assert_eq!(calls.get(), usize::from(register));
    }
}
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
fn nine_public_post_disposal_outcomes_follow_source_instead_of_blanket_rejection() {
    for action in [
        "resize", "clear", "reset", "scroll", "input", "write", "marker", "parser", "unicode",
    ] {
        let term = Terminal::new(8, 3, 10).unwrap();
        term.options()
            .set("allowProposedApi", OptionValue::Bool(true))
            .unwrap();
        term.write("A").unwrap();
        term.dispose().unwrap();
        let callback = Cell::new(false);
        match action {
            "resize" => term.resize(6, 4).unwrap(),
            "clear" => term.clear().unwrap(),
            "reset" => term.reset().unwrap(),
            "scroll" => term.scroll_lines(-1).unwrap(),
            "input" => term.input(&[77], true).unwrap(),
            "write" => term
                .write_with_callback("B", |_| {
                    callback.set(true);
                    Ok(())
                })
                .unwrap(),
            "marker" => {
                term.register_marker(1).unwrap();
            }
            "parser" => {
                term.with_parser(|api| {
                    api.register_osc_handler(999.0, |_| HandlerResult::Ready(true))
                })
                .unwrap()
                .unwrap();
            }
            "unicode" => term.unicode().unwrap().set_active_version("6").unwrap(),
            _ => unreachable!(),
        }
        assert!(!callback.get(), "{action}");
        assert_eq!(
            (term.cols().unwrap(), term.rows().unwrap()),
            if action == "resize" { (6, 4) } else { (8, 3) },
            "{action}"
        );
        assert_eq!(
            text(&term),
            if action == "reset" { "" } else { "A" },
            "{action}"
        );
        assert_eq!(
            term.markers().unwrap().len(),
            usize::from(action == "marker"),
            "{action}"
        );
    }
}
